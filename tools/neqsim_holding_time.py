#!/usr/bin/env python3
"""NeqSim reference cases for the Type C holding time (issue #868, spec 0016 sections 4 and 7, D6).

Runs NeqSim offline, with the GERG-2008 equation of state, on closed-tank methane cases and writes
`evaluation/holding_time/neqsim_cases.json`: the inputs, the NeqSim version, the pressure-time
trajectory and a hash of all of it.

What the cases check: saturation properties and the energy-balance stepping. Every case shares the
homogeneous assumption (one saturation state, no thermal stratification) and the heat leak with the
model under test, so they do not check either of those.

Not run in CI (NeqSim is not a repo dependency; it needs Java 11 or newer and a Python venv):

    python3 -m venv ~/Dev/_venv-neqsim && ~/Dev/_venv-neqsim/bin/pip install neqsim==3.27.0
    ~/Dev/_venv-neqsim/bin/python tools/neqsim_holding_time.py            # write the file
    ~/Dev/_venv-neqsim/bin/python tools/neqsim_holding_time.py --check    # recompute, compare the hash

The hash covers every number in `neqsim_cases.json`. The Java and Python versions are in
`neqsim_environment.json`, outside the hash.
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evaluation/holding_time/neqsim_cases.json"
ENV = ROOT / "evaluation/holding_time/neqsim_environment.json"
NIST = ROOT / "catalogue/methane_saturation/nist_methane.tsv"
ATM_BAR = 1.01325
PINNED_NEQSIM = "3.27.0"
EOS = "SystemGERG2008Eos"
VOLUME_M3 = 100.0

# (id, fill: "LL" is the loading limit at the relief setting, or a fraction of the volume;
#  heat leak kW, withdrawal kg/s of vapour, relief setting MPa gauge)
CASES = [
    ("A_fillLL_q10", "LL", 10.0, 0.0, 0.50),
    ("B_fill080_q10", 0.80, 10.0, 0.0, 0.50),
    ("C_fillLL_q20", "LL", 20.0, 0.0, 0.50),
    ("D_fillLL_q10_withdrawal", "LL", 10.0, 0.002, 0.50),
]
FILLING_LIMIT = 0.98  # IGC 15.1.3.1: the filling limit FL
REF_STEP_S = 60.0     # the stored reference; run to step convergence (see build)
COARSE_STEP_S = 120.0


def sig(x, n=7):
    return float(f"{x:.{n}g}")


# ---- NeqSim saturation properties --------------------------------------------------------------
def neqsim_sat(t_c, jneqsim):
    """Pressure (bar abs), densities (kg/m3), internal energies and enthalpies (kJ/kg) of saturated
    methane at t_c (C), liquid and vapour. The energy zero is NeqSim's own, not NIST's."""
    f = getattr(jneqsim.thermo.system, EOS)(t_c + 273.15, 2.0)
    f.addComponent("methane", 1.0)
    jneqsim.thermodynamicoperations.ThermodynamicOperations(f).bubblePointPressureFlash(False)
    f.initProperties()
    out = {"p": f.getPressure()}
    for i in range(f.getNumberOfPhases()):
        ph = f.getPhase(i)
        k = "v" if ph.getType().toString() == "GAS" else "l"
        out["rho_" + k] = ph.getDensity("kg/m3")
        out["u_" + k] = ph.getInternalEnergy("kJ/kg")
        out["h_" + k] = ph.getEnthalpy("kJ/kg")
    assert {"rho_l", "rho_v"} <= set(out), t_c
    return out


def nist_table():
    rows = {}
    for ln in NIST.read_text(encoding="utf-8").splitlines()[1:]:
        c = ln.split("\t")
        rows[int(c[0])] = dict(zip(["p", "rho_l", "rho_v", "u_l", "u_v", "h_l", "h_v"], map(float, c[1:])))
    return rows


def nist_sat(t_c, rows):
    lo = int(t_c // 1)
    a, b = rows[lo], rows[min(lo + 1, -100)]
    w = t_c - lo
    return {k: a[k] + w * (b[k] - a[k]) for k in a}


# ---- the closed-tank stepping (spec 0016 section 4) --------------------------------------------
def state_from(u, v, sat, t_guess):
    """(T, x) whose specific internal energy is u (kJ/kg) and specific volume v (m3/kg). Fails if the
    state is not two-phase (0 <= x <= 1)."""
    def g(t):
        s = sat(t)
        x = (v - 1 / s["rho_l"]) / (1 / s["rho_v"] - 1 / s["rho_l"])
        return (1 - x) * s["u_l"] + x * s["u_v"] - u, x, s
    lo, hi = t_guess - 0.05, t_guess + 0.5
    flo, fhi = g(lo)[0], g(hi)[0]
    while flo > 0:
        lo -= 1.0; flo = g(lo)[0]
    while fhi < 0:
        hi += 2.0; fhi = g(hi)[0]
    side = 0
    for _ in range(100):  # Illinois regula falsi
        t = (lo * fhi - hi * flo) / (fhi - flo)
        ft = g(t)[0]
        if abs(ft) < 1e-9 or hi - lo < 1e-12:
            break
        if ft < 0:
            lo, flo = t, ft
            if side == -1: fhi /= 2
            side = -1
        else:
            hi, fhi = t, ft
            if side == 1: flo /= 2
            side = 1
    _, x, s = g(t)
    if not -1e-9 <= x <= 1 + 1e-9:
        raise SystemExit(f"quality {x} outside 0..1 at {t} C: the tank is not two-phase (liquid-full or dry)")
    return t, x, s


def start_state(sat, p_bar):
    lo, hi = -170.0, -100.0
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        lo, hi = (mid, hi) if sat(mid)["p"] < p_bar else (lo, mid)
    return 0.5 * (lo + hi)


def run_case(case, sat, step_s):
    cid, fill_in, q_kw, mdot, relief_mpa = case
    p_relief = relief_mpa * 10 + ATM_BAR
    t0 = start_state(sat, ATM_BAR)
    s0 = sat(t0)
    if fill_in == "LL":  # LL = FL * rho(relief) / rho(start), rounded down to 4 digits
        s_r = sat(start_state(sat, p_relief))
        fill = int(FILLING_LIMIT * s_r["rho_l"] / s0["rho_l"] * 1e4) / 1e4
    else:
        fill = fill_in
    v_tot = VOLUME_M3
    m = fill * v_tot * s0["rho_l"] + (1 - fill) * v_tot * s0["rho_v"]
    m_start = m
    u_tot = fill * v_tot * s0["rho_l"] * s0["u_l"] + (1 - fill) * v_tot * s0["rho_v"] * s0["u_v"]
    t, x, s = t0, (1 - fill) * v_tot * s0["rho_v"] / m, s0
    keep_every = int(round(3600 / step_s))
    traj = [(0.0, s["p"], t)]
    capped, time_s, n, p_prev = False, 0.0, 0, s["p"]
    p_floor = s["p"]  # withdrawal stops when the pressure falls to the start pressure
    while True:
        want = mdot * step_s if (mdot > 0 and s["p"] >= p_floor - 1e-12) else 0.0
        take = min(want, x * m)
        capped |= take < want - 1e-12
        u_tot += q_kw * step_s - take * s["h_v"]
        m -= take
        t, x, s = state_from(u_tot / m, v_tot / m, sat, t)
        n += 1
        t_prev = time_s
        time_s += step_s
        if s["p"] >= p_relief:
            w = (p_relief - p_prev) / (s["p"] - p_prev)  # straight line in time across the last step
            time_s = t_prev + w * step_s
            traj.append((time_s, p_relief, t))
            break
        if n % keep_every == 0:
            traj.append((time_s, s["p"], t))
        p_prev = s["p"]
        if n > 2_000_000:
            raise SystemExit("no relief pressure reached")
    return {
        "id": cid, "fill_fraction": fill, "heat_leak_kw": q_kw, "withdrawal_kg_s_vapour": mdot,
        "relief_mpa_gauge": relief_mpa, "volume_m3": v_tot, "step_s": step_s,
        "start_temperature_c": sig(t0), "start_mass_kg": sig(m_start),
        "holding_time_h": sig(time_s / 3600.0, 7), "end_temperature_c": sig(t),
        "end_quality": sig(x), "withdrawal_capped_by_vapour": bool(capped),
        "trajectory_h_bar_abs_c": [[sig(a / 3600, 7), sig(b), sig(c)] for a, b, c in traj],
    }


def pct(a, b):
    return 100 * (a - b) / b


def build():
    from neqsim import jneqsim
    import importlib.metadata as md
    ver = md.version("neqsim")
    if ver != PINNED_NEQSIM:
        sys.exit(f"NeqSim {ver} installed; this script pins {PINNED_NEQSIM}")
    cache = {}

    def sat(t):
        k = round(t, 9)
        if k not in cache:
            if len(cache) > 400000:
                cache.clear()
            cache[k] = neqsim_sat(t, jneqsim)
        return cache[k]

    rows = nist_table()

    def sat_nist(t):
        return nist_sat(t, rows)

    results = []
    for c in CASES:
        r = run_case(c, sat, REF_STEP_S)
        coarse = run_case(c, sat, COARSE_STEP_S)
        tab = run_case(c, sat_nist, REF_STEP_S)
        r["step_convergence_holding_time_percent_120s_vs_60s"] = sig(pct(coarse["holding_time_h"], r["holding_time_h"]), 4)
        r["nist_table_holding_time_h"] = tab["holding_time_h"]
        r["holding_time_diff_vs_nist_table_percent"] = sig(pct(r["holding_time_h"], tab["holding_time_h"]), 4)
        pa, pb = r["trajectory_h_bar_abs_c"][:-1], tab["trajectory_h_bar_abs_c"][:-1]
        r["max_pressure_diff_vs_nist_table_run_percent"] = sig(max(abs(pct(a[1], b[1])) for a, b in zip(pa, pb)), 4)
        results.append(r)

    t_lo = int(min(r["start_temperature_c"] for r in results) // 1)
    t_hi = int(-(-max(r["end_temperature_c"] for r in results) // 1))
    # NeqSim's energy zero differs from NIST's: one offset, fixed at the saturated liquid of -160 C
    off_u = rows[-160]["u_l"] - sat(-160)["u_l"]
    off_h = rows[-160]["h_l"] - sat(-160)["h_l"]
    spread = {"rows_from_c": t_lo, "rows_to_c": t_hi,
              "energy_offset_kj_kg": {"internal": sig(off_u), "enthalpy": sig(off_h),
                                      "anchor": "saturated liquid at -160 C"}}
    cols = {"p": "pressure_percent", "rho_l": "liquid_density_percent", "rho_v": "vapour_density_percent",
            "u_l": "liquid_internal_energy_kj_kg", "u_v": "vapour_internal_energy_kj_kg",
            "h_l": "liquid_enthalpy_kj_kg", "h_v": "vapour_enthalpy_kj_kg"}
    rel = ("p", "rho_l", "rho_v")
    for k, name in cols.items():
        at_rows, between = 0.0, 0.0
        for t in range(t_lo, t_hi + 1):
            off = off_u if k.startswith("u") else off_h if k.startswith("h") else 0.0
            def d(tc, nist_val):
                nq = sat(tc)[k] + off
                return abs(nq - nist_val) / (abs(nist_val) / 100 if k in rel else 1.0)
            at_rows = max(at_rows, d(float(t), rows[t][k]))
            if t < t_hi:  # half way between rows: the NIST value is the straight line OVD reads
                between = max(between, d(t + 0.5, 0.5 * (rows[t][k] + rows[t + 1][k])))
        spread["max_difference_at_rows_" + name] = sig(at_rows, 4)
        spread["max_difference_half_way_between_rows_" + name] = sig(between, 4)

    return {
        "purpose": ("Reference cases for the Type C holding time. They check saturation properties and "
                    "the energy-balance stepping only. Every case shares the homogeneous assumption "
                    "(one saturation state, no stratification) and the heat leak with the model under test."),
        "neqsim_version": ver, "equation_of_state": "GERG-2008 (NeqSim SystemGERG2008Eos), pure methane",
        "model": "closed tank, homogeneous, start saturated at 101.325 kPa absolute, vapour withdrawn at saturated vapour enthalpy; fills are at or below the loading limit",
        "reference_step_s": REF_STEP_S,
        "spread_neqsim_vs_nist_table": spread,
        "cases": results,
    }


def java_version():
    import subprocess
    r = subprocess.run(["java", "-version"], capture_output=True, text=True)
    return r.stderr.splitlines()[0] if r.stderr else "unknown"


def content_hash(doc):
    body = {k: v for k, v in doc.items() if k != "content_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    doc = build()
    doc["content_sha256"] = content_hash(doc)
    env = {"java": java_version(), "python": sys.version.split()[0], "note": "not hashed"}
    if a.check:
        stored = json.loads(OUT.read_text(encoding="utf-8"))
        ok = stored["content_sha256"] == doc["content_sha256"]
        print(("same" if ok else "DIFFERENT"), doc["content_sha256"])
        sys.exit(0 if ok else 1)
    OUT.write_text(json.dumps(doc, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    ENV.write_text(json.dumps(env, indent=1) + "\n", encoding="utf-8")
    print(doc["content_sha256"])


if __name__ == "__main__":
    main()
