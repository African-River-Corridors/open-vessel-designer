#!/usr/bin/env python3
"""An independent oracle for the damage stability verdict (issue #763, spec 0012).

It never calls the engine and shares no code with it. The hull is a wall-sided box, L x B x D = 60 x 6 x 12 m,
at 1,200 t (density 1.0), with one flooded compartment across the whole beam (10 m long, 1.5 m high, permeability
0.95) at mid-length, so the damaged hull has no trim and no heel of its own. A transverse offset `tcg` of the
centre of gravity gives a heel. For a wall-sided box the lever of the damaged hull has a closed form (the same one
`crates/ovd-core/tests/damage_stability.rs` works out):

    T      = (M + lost) / (L B),  lost = p l B h
    GZ(a)  = (L B^3 tan(a) / 12 / M) cos(a) + (Mz / M - KG) sin(a) - tcg cos(a)
    Mz     = L (B T^2 / 2 + B^3 tan(a)^2 / 24) - lost h / 2

valid while neither the deck edge nor the bottom edge is out of the water (here to about 49 degrees).

The criteria are typed here from the texts, not read from the engine:

    ADN 2025 9.3.1.15.2 / 9.3.2.15.2, ES-TRIN 28.03(6): heel <= 12, GZ >= 0.05 m and area >= 0.0065 m.rad to 27
    ES-TRIN 28.03(5): intermediate heel <= 15, GZ >= 0.02 m before 27 degrees
    ADN 9.3.1.15.2: intermediate GZ >= 0.03 m over a positive range of 5 degrees (no heel limit)

The curve is read in steps of 2 degrees from the equilibrium to the limit, the trapezoid area, as the engine does
(`STEP_DEG`); a case whose lever is lost inside the range is only compared on its verdict.

    python3 evaluation/oracle/damage_oracle.py           write evaluation/damage/cases.json
    python3 evaluation/oracle/damage_oracle.py --check   fail if the file differs
"""
import json
import math
import sys
from pathlib import Path

L, B, D, M = 60.0, 6.0, 12.0, 1200.0
LEN, HEIGHT, PERM = 10.0, 1.5, 0.95
LOST = PERM * LEN * B * HEIGHT
T = (M + LOST) / (L * B)
STEP = 2.0

#: name -> (max heel, min GZ, min area, limit angle or None, range or None)
CRITERIA = {
    "adn_9_3_1_final": (12.0, 0.05, 0.0065, 27.0, None),
    "es_trin_28_03_final": (12.0, 0.05, 0.0065, 27.0, None),
    "es_trin_28_03_intermediate": (15.0, 0.02, 0.0, 27.0, None),
    "adn_9_3_1_intermediate": (90.0, 0.03, 0.0, None, 5.0),
}


def gz(deg, kg, tcg):
    a = math.radians(deg)
    tn = math.tan(a)
    my = L * B**3 * tn / 12.0
    mz = L * (B * T * T / 2.0 + B**3 * tn * tn / 24.0) - LOST * HEIGHT / 2.0
    return my / M * math.cos(a) + (mz / M - kg) * math.sin(a) - tcg * math.cos(a)


def equilibrium(kg, tcg):
    if tcg == 0.0:
        return 0.0
    deg = 0.0
    while gz(deg, kg, tcg) <= 0.0:
        deg += 0.001
        if deg > 45.0:
            return None
    lo, hi = deg - 0.001, deg
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        if gz(mid, kg, tcg) <= 0.0:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def assess(name, kg, tcg):
    max_heel, min_gz, min_area, limit, rng = CRITERIA[name]
    eq = equilibrium(kg, tcg)
    if eq is None:
        return dict(ok=False, heel=None, max_gz=0.0, area=0.0, lever_lost=True)
    top = eq + rng if rng is not None else limit
    max_gz, area, prev_gz, prev, end = 0.0, 0.0, 0.0, eq, eq
    lost = False
    deg = eq
    while deg < top:
        nxt = min(deg + STEP, top)
        g = gz(nxt, kg, tcg)
        if g <= 0.0:
            lost = True
            break
        area += 0.5 * (prev_gz + g) * (nxt - prev) * math.pi / 180.0
        max_gz = max(max_gz, g)
        prev_gz, prev, end, deg = g, nxt, nxt, nxt
    range_met = rng is None or (not lost and end >= top - 1e-9)
    ok = abs(eq) <= max_heel and range_met and max_gz >= min_gz and area >= min_area
    return dict(ok=ok, heel=eq, max_gz=max_gz, area=area, lever_lost=lost)


def solve_kg(name, tcg=0.0):
    """The KG at which the verdict flips, by bisection (a larger KG gives a smaller lever)."""
    lo, hi = 0.5, 6.0
    assert assess(name, lo, tcg)["ok"] and not assess(name, hi, tcg)["ok"], name
    for _ in range(80):
        mid = 0.5 * (lo + hi)
        if assess(name, mid, tcg)["ok"]:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def solve_tcg(name, kg, heel):
    """The offset of G that puts the equilibrium at `heel` degrees: GZ(heel) = tcg cos(heel)."""
    return gz(heel, kg, 0.0) / math.cos(math.radians(heel))


def which_binds(name, a):
    max_heel, min_gz, min_area, limit, rng = CRITERIA[name]
    out = []
    if a["heel"] is None or abs(a["heel"]) > max_heel:
        out.append("heel")
    if a["max_gz"] < min_gz:
        out.append("gz")
    if a["area"] < min_area:
        out.append("area")
    if a["lever_lost"] and rng is not None:
        out.append("range")
    return out


def cases():
    out = []

    def add(cid, name, kg, tcg, limit):
        a = assess(name, kg, tcg)
        out.append(
            dict(
                id=cid,
                criteria=name,
                limit=limit,
                kg_m=round(kg, 6),
                tcg_m=round(tcg, 6),
                expect=dict(
                    ok=a["ok"],
                    heel_deg=None if a["heel"] is None else round(a["heel"], 6),
                    max_gz_m=round(a["max_gz"], 6),
                    area_m_rad=round(a["area"], 6),
                    failing=which_binds(name, a),
                ),
            )
        )

    # A lever limit: the KG at which the verdict flips, a little each side. On this hull the lever never binds
    # before the heel (the wall-sided lever only rises), so a lever limit binds only upright: the area of the final
    # stage (the lever of 0.05 m is met there, 0.057 against 0.05) and the lever of the ADN intermediate stage.
    # The 0.05 m and 0.02 m levers do not bind on any wall-sided box; their constants are held by the literal test
    # in `crates/ovd-format/tests/damage_boundaries.rs`.
    for name, limit, binds in [
        ("adn_9_3_1_final", "area 0.0065 m.rad", "area"),
        ("es_trin_28_03_final", "area 0.0065 m.rad", "area"),
        ("adn_9_3_1_intermediate", "gz 0.03 m over 5 degrees", "gz"),
    ]:
        kg = solve_kg(name)
        for side, d in (("below", -0.004), ("above", 0.004)):
            if side == "above":
                assert which_binds(name, assess(name, kg + d, 0.0)) == [binds], name
            add(f"{name}-lever-{side}", name, kg + d, 0.0, limit)
    # a heel limit: the offset of G that puts the equilibrium at the limit, a little each side. The KG is low
    # enough that the lever beyond the equilibrium is not what fails.
    for name, heel in [("adn_9_3_1_final", 12.0), ("es_trin_28_03_final", 12.0), ("es_trin_28_03_intermediate", 15.0)]:
        kg = 2.0
        t = solve_tcg(name, kg, heel)
        for side, f in (("below", 0.97), ("above", 1.03)):
            add(f"{name}-heel-{heel:g}-{side}", name, kg, t * f, f"heel {heel:g} degrees")
    return out


def render():
    return json.dumps(
        dict(
            note="Generated by evaluation/oracle/damage_oracle.py. Do not edit by hand.",
            hull=dict(length_m=L, beam_m=B, depth_m=D, mass_t=M, flooded=dict(length_m=LEN, height_m=HEIGHT, permeability=PERM)),
            cases=cases(),
        ),
        indent=2,
    ) + "\n"


if __name__ == "__main__":
    path = Path(__file__).resolve().parents[1] / "damage" / "cases.json"
    text = render()
    if "--check" in sys.argv:
        if not path.exists() or path.read_text() != text:
            sys.exit(f"{path} differs from the oracle: run damage_oracle.py")
        print("damage_oracle: ok")
    else:
        path.write_text(text)
        for c in json.loads(text)["cases"]:
            print(c["id"], c["expect"])
