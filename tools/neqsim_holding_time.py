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

Only the `content_sha256` is a reproducibility claim; the `environment` block is outside it.
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evaluation/holding_time/neqsim_cases.json"
NIST = ROOT / "catalogue/methane_saturation/nist_methane.tsv"
ATM_BAR = 1.01325
PINNED_NEQSIM = "3.27.0"
EOS = "SystemGERG2008Eos"
STEP_S = 3600.0
VOLUME_M3 = 100.0

# (id, fill fraction of the volume, heat leak kW, withdrawal kg/s of vapour, relief setting MPa gauge)
CASES = [
    ("A_fill090_q10", 0.90, 10.0, 0.0, 0.50),
    ("B_fill080_q10", 0.80, 10.0, 0.0, 0.50),
    ("C_fill090_q20", 0.90, 20.0, 0.0, 0.50),
    ("D_fill090_q10_withdrawal", 0.90, 10.0, 0.002, 0.50),
]


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
def state_from(u, v, sat, lo, hi):
    """(T, x) whose specific internal energy is u (kJ/kg) and specific volume v (m3/kg)."""
    def g(t):
        s = sat(t)
        x = (v - 1 / s["rho_l"]) / (1 / s["rho_v"] - 1 / s["rho_l"])
        return (1 - x) * s["u_l"] + x * s["u_v"] - u, x, s
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        if g(mid)[0] < 0:
            lo = mid
        else:
            hi = mid
    t = 0.5 * (lo + hi)
    _, x, s = g(t)
    return t, x, s


def run_case(case, sat, offset_u, step_s=STEP_S, keep_every=6):
    cid, fill, q_kw, mdot, relief_mpa = case
    p_relief = relief_mpa * 10 + ATM_BAR
    # start: saturated at atmospheric pressure, found by bisection on the saturation pressure
    lo, hi = -170.0, -150.0
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        lo, hi = (mid, hi) if sat(mid)["p"] < ATM_BAR else (lo, mid)
    t0 = 0.5 * (lo + hi)
    s0 = sat(t0)
    m_l, v_tot = fill * VOLUME_M3, VOLUME_M3
    m = fill * VOLUME_M3 * s0["rho_l"] + (1 - fill) * VOLUME_M3 * s0["rho_v"]
    u_tot = (fill * VOLUME_M3 * s0["rho_l"] * s0["u_l"] + (1 - fill) * VOLUME_M3 * s0["rho_v"] * s0["u_v"]) + 0.0
    t, x, s = t0, (1 - fill) * VOLUME_M3 * s0["rho_v"] / m, s0
    traj = [(0.0, s["p"], t)]
    capped = False
    time_s, n = 0.0, 0
    p_prev = s["p"]
    while s["p"] < p_relief and n < 100000:
        vap_avail = x * m
        take = min(mdot * step_s, vap_avail) if (mdot > 0 and s["p"] > ATM_BAR + 1e-6) else 0.0
        capped |= mdot > 0 and take < mdot * step_s - 1e-12 and s["p"] > ATM_BAR + 1e-6
        u_tot += q_kw * step_s - take * s["h_v"]
        m -= take
        t, x, s = state_from(u_tot / m, v_tot / m, sat, t - 1.0, t + 6.0)
        n += 1
        p_prev_t = time_s
        time_s += step_s
        if n % keep_every == 0:
            traj.append((time_s, s["p"], t))
        if s["p"] >= p_relief:
            # straight line in time between the last two steps, for the crossing
            w = (p_relief - p_prev) / (s["p"] - p_prev)
            time_s = p_prev_t + w * step_s
            traj.append((time_s, p_relief, t))
            break
        p_prev = s["p"]
    return {
        "id": cid, "fill_fraction": fill, "heat_leak_kw": q_kw, "withdrawal_kg_s_vapour": mdot,
        "relief_mpa_gauge": relief_mpa, "volume_m3": VOLUME_M3, "step_s": step_s,
        "start_temperature_c": sig(t0), "start_mass_kg": sig(m if mdot == 0 else
            fill * VOLUME_M3 * s0["rho_l"] + (1 - fill) * VOLUME_M3 * s0["rho_v"]),
        "holding_time_h": sig(time_s / 3600.0, 6), "end_temperature_c": sig(t),
        "withdrawal_capped_by_vapour": bool(capped),
        "trajectory_h_bar_abs_c": [[sig(a / 3600, 6), sig(b), sig(c)] for a, b, c in traj],
    }


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
            cache[k] = neqsim_sat(t, jneqsim)
        return cache[k]

    rows = nist_table()
    # NeqSim's energy zero differs from NIST's: one offset, fixed at the saturated-liquid row of -160 C
    off_u = rows[-160]["u_l"] - sat(-160)["u_l"]
    off_h = rows[-160]["h_l"] - sat(-160)["h_l"]

    results = [run_case(c, sat, off_u) for c in CASES]
    # the same stepping on the NIST table (straight line between rows), same states and inputs
    nist_cache = {}
    def sat_nist(t):
        return nist_sat(t, rows)
    nist_runs = {c[0]: run_case(c, sat_nist, 0.0) for c in CASES}
    half = {c[0]: run_case(c, sat, off_u, step_s=STEP_S / 2) for c in CASES}
    for r in results:
        r["nist_table_holding_time_h"] = nist_runs[r["id"]]["holding_time_h"]
        r["holding_time_diff_vs_nist_table_percent"] = sig(
            100 * (r["holding_time_h"] - nist_runs[r["id"]]["holding_time_h"]) / nist_runs[r["id"]]["holding_time_h"], 4)
        r["half_step_holding_time_h"] = half[r["id"]]["holding_time_h"]
        r["halving_the_step_changes_holding_time_percent"] = sig(
            100 * (half[r["id"]]["holding_time_h"] - r["holding_time_h"]) / r["holding_time_h"], 4)

    t_lo = int(min(r["start_temperature_c"] for r in results) // 1)
    t_hi = int(-(-max(r["end_temperature_c"] for r in results) // 1))
    spread = {"rows_from_c": t_lo, "rows_to_c": t_hi,
              "energy_offset_kj_kg": {"internal": sig(off_u), "enthalpy": sig(off_h),
                                      "anchor": "saturated liquid at -160 C"}}
    cols = {"p": "pressure_percent", "rho_l": "liquid_density_percent", "rho_v": "vapour_density_percent",
            "u_l": "liquid_internal_energy_kj_kg", "u_v": "vapour_internal_energy_kj_kg",
            "h_l": "liquid_enthalpy_kj_kg", "h_v": "vapour_enthalpy_kj_kg"}
    for k, name in cols.items():
        worst = 0.0
        for t in range(t_lo, t_hi + 1):
            nq, ni = sat(float(t))[k], rows[t][k]
            if k.startswith("u"):
                nq += off_u
            if k.startswith("h"):
                nq += off_h
            d = abs(nq - ni) / (abs(ni) if k in ("p", "rho_l", "rho_v") else 1.0)
            worst = max(worst, d * (100 if k in ("p", "rho_l", "rho_v") else 1))
        spread["max_abs_difference_" + name] = sig(worst, 4)

    return {
        "purpose": ("Reference cases for the Type C holding time. They check saturation properties and "
                    "the energy-balance stepping only. Every case shares the homogeneous assumption "
                    "(one saturation state, no stratification) and the heat leak with the model under test."),
        "neqsim_version": ver, "equation_of_state": "GERG-2008 (NeqSim SystemGERG2008Eos), pure methane",
        "model": "closed tank, homogeneous, start saturated at 101.325 kPa absolute, vapour withdrawn at saturated vapour enthalpy",
        "step_s": STEP_S,
        "spread_neqsim_vs_nist_table": spread,
        "cases": results,
        "environment": {"java": java_version(), "python": sys.version.split()[0]},
    }


def java_version():
    import subprocess
    r = subprocess.run(["java", "-version"], capture_output=True, text=True)
    return r.stderr.splitlines()[0] if r.stderr else "unknown"


def content_hash(doc):
    body = {k: v for k, v in doc.items() if k not in ("environment", "content_sha256")}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    doc = build()
    doc["content_sha256"] = content_hash(doc)
    if a.check:
        stored = json.loads(OUT.read_text(encoding="utf-8"))
        ok = stored["content_sha256"] == doc["content_sha256"]
        print(("same" if ok else "DIFFERENT"), doc["content_sha256"])
        sys.exit(0 if ok else 1)
    OUT.write_text(json.dumps(doc, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(doc["content_sha256"])


if __name__ == "__main__":
    main()
