#!/usr/bin/env python3
"""Regenerate the generated files of examples/gas-140x29-coastal-lng/ from the engine (issue #822).

    python3 tools/refresh_gas_example.py <path to ovd>             # results.json, ga/, trace.md, README table
    python3 tools/refresh_gas_example.py --from-results FILE ...   # trace.md and the table only, from a results file

Nothing is typed: `results.json` is `ovd run --json`, `ga/` is `ovd draw`, `trace.md` and the README
Results table come from `results.json`. The README rows that name a quantity are replaced from the
results; the prose around them is the author's. It fails when a table row has no source, and checks that
every Option's `hash_status` is "matches".
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "examples" / "gas-140x29-coastal-lng"
DESIGN = DIR / "gas-140x29-coastal-lng.vdesign"


def cell(x) -> str:
    if x is None:
        return ""
    return str(x) if isinstance(x, bool) else f"{x:.4g}"


def trace_md(option: dict) -> str:
    rt = option["results"]["rule_trace"]
    lines = [
        f"# Rule trace: Option {option['name']}",
        "",
        f"Scope: {rt['scope']}",
        "",
        'One row per Rule and object. Status is the Rule\'s verification status, then the Measure\'s method status. "Finding" is the register ticket.',
        "",
        "| Rule | Object | Outcome | Value | Limit | Unit | Consequence | Status | Finding | Citation |",
        "|---|---|---|---:|---:|---|---|---|---|---|",
    ]
    for r in rt["rows"]:
        lines.append(
            f"| {r['rule']} | {r['object'] or ''} | {r['outcome']} | {cell(r['value'])} | {cell(r['limit'])} | "
            f"{r['unit']} | {r['consequence']} | {r['rule_status']} / {r['method_status']} | {r['finding'] or ''} | {r['citation']} |"
        )
    seen: list[tuple[str, str]] = []
    for r in rt["rows"]:
        if r["gap"] and r["rule"] not in [s[0] for s in seen]:
            seen.append((r["rule"], r["gap"]))
    lines += ["", "## Gaps the trace reports", ""]
    lines += [f"- `{rule}`: {gap}" for rule, gap in seen]
    return "\n".join(lines)


def n(x, d=0):
    return f"{x:,.{d}f}"


def readme_rows(options: list[dict]) -> dict[str, list[str]]:
    """Row label -> the cell of each of the first three Options, from the results."""
    rows: dict[str, list[str]] = {}
    for o in options:
        r = o["results"]
        res, arr, geo, st = r["results"], r["arrangement"], r["geometry"], r["stability"]
        cond = {c["name"]: c for c in st["conditions"]}
        env = r["powering"]["environments"][0]
        put = lambda k, v: rows.setdefault(k, []).append(v)  # noqa: E731
        put("Moulded depth (m)", n(arr["depth_m"], 2))
        put("Tanks across x along", f"{arr['n_wide']} x {arr['n_long']}")
        put("Tank outer diameter, to insulation (m)", n(arr["tank_od_m"], 2))
        put("Side clearance per side (m)", n(arr["side_clearance_m"], 2))
        put("Cargo zone (m)", n(geo["cargo_zone_length_m"], 1))
        put("Bow void (m)", n(geo["bow_void_length_m"], 1))
        put("Stern void (m)", n(geo["stern_void_length_m"], 1))
        put("Cargo volume at 0.98 fill (m3)", n(res["cargo_volume_m3"]))
        put("Cargo (t)", n(res["cargo_t"]))
        put("Lightship (t)", n(res["lightship_t"]))
        put("Displacement, laden (t)", n(res["displacement_t"]))
        put("Draught, laden (m)", n(res["draught_m"], 2))
        put("Draught cap (depth less freeboard) (m)", n(res["draught_max_m"], 2))
        put("Freeboard, laden (m)", n(res["freeboard_m"], 2))
        put("Air draft, laden (m)", n(res["air_draft_loaded_m"], 2))
        put("Air draft, light (m)", n(res["air_draft_light_m"], 2))
        put("GM fluid, full load departure (m)", n(cond["Full load, departure"]["gm_fluid_m"], 2))
        put("Max GZ, full load departure (m)", n(cond["Full load, departure"]["max_gz_m"], 2))
        put("GM fluid, ballast departure (m)", n(cond["Ballast, departure"]["gm_fluid_m"], 2))
        pw = r["powering"]
        put(
            f"Installed power (design speed {pw['design_speed_kn']:.1f} kn, margin {pw['margin']:g}) (kW)",
            n(pw["installed_power_kw"]),
        )
        put("Service speed, deep water (kn)", n(env["service_speed_kn"], 1))
        put("Power at the service speed (kW)", n(env["power_at_service_kw"]))
        bo = r["boil_off"]
        put("Boil-off (IGC 7.2 ambient) (kg/h)", n(bo["boil_off_kg_per_h"]))
        put("Gas use at the service speed (kg/h)", n(env["fuel_gas_kg_per_h"]))
        failed = [c["name"] for c in r["checks"] if not c["ok"]]
        put("Checks", ", ".join(failed) if failed else "all pass")
        put("Results hash", o["results_hash"][:12])
    return rows


def update_sensitivities(options: list[dict]) -> None:
    """Replace the Tank OD, Cargo and Draught cells of the sensitivities table, by Option name."""
    path = DIR / "README.md"
    by_name = {o["name"]: o["results"] for o in options}
    out = []
    for line in path.read_text().split("\n"):
        cells = [c.strip() for c in line.strip("|").split("|")] if line.startswith("| ") else []
        if len(cells) == 6 and cells[0] in by_name:
            r = by_name[cells[0]]
            res = r["results"]
            cells[2:] = [
                n(r["arrangement"]["tank_od_m"], 2),
                n(res["cargo_volume_m3"]),
                n(res["cargo_t"]),
                n(res["draught_m"], 2),
            ]
            line = "| " + " | ".join(cells) + " |"
        out.append(line)
    path.write_text("\n".join(out))


def update_readme(options: list[dict]) -> list[str]:
    """Replace the cells of the Results table that the results give. Returns the labels with no source."""
    path = DIR / "README.md"
    text = path.read_text()
    rows = readme_rows(options[:3])
    out, missing = [], []
    start = text.index("## Results")
    end = text.index("\n\n", text.index("| Quantity |", start))
    for line in text[start:end].split("\n"):
        m = re.match(r"\| (.+?) \| (.*) \|$", line)
        label = m.group(1) if m else None
        if label in rows:
            line = f"| {label} | " + " | ".join(rows[label]) + " |"
        elif label and label != "Quantity" and not set(label) <= set("-:"):
            missing.append(label)
        out.append(line)
    path.write_text(text[:start] + "\n".join(out) + text[end:])
    return missing


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("ovd", nargs="?")
    ap.add_argument("--from-results")
    a = ap.parse_args()
    if a.from_results:
        data = json.loads(Path(a.from_results).read_text())
    else:
        if not a.ovd:
            ap.error("the path to ovd is needed")
        run = subprocess.run([a.ovd, "run", "--json", str(DESIGN)], capture_output=True, text=True)
        if run.returncode != 0:
            print(f"ovd run failed with exit code {run.returncode}:\n{run.stderr}", file=sys.stderr)
            return 1
        data = json.loads(run.stdout)
        (DIR / "results.json").write_text(run.stdout)
        subprocess.run([a.ovd, "draw", "--out", str(DIR / "ga"), str(DESIGN)], check=True, capture_output=True)
        for stray in (DIR / "ga").glob("*"):
            if stray.suffix not in (".pdf", ".svg"):
                stray.unlink()
    bad = [o["name"] for o in data["options"] if o.get("hash_status") != "matches"]
    if bad:
        print("hash_status is not 'matches' for:", ", ".join(bad), file=sys.stderr)
        return 1
    (DIR / "trace.md").write_text(trace_md(data["options"][0]))
    if a.ovd:
        sens = subprocess.run(
            [a.ovd, "run", "--json", str(DIR / "gas-140x29-coastal-lng-sensitivities.vdesign")],
            capture_output=True,
            text=True,
        )
        if sens.returncode != 0:
            print(f"ovd run (sensitivities) failed with exit code {sens.returncode}:\n{sens.stderr}", file=sys.stderr)
            return 1
        sdata = json.loads(sens.stdout)
        if any(o.get("hash_status") != "matches" for o in sdata["options"]):
            print("sensitivities: hash_status is not 'matches'", file=sys.stderr)
            return 1
        update_sensitivities(sdata["options"])
    missing = update_readme(data["options"])
    for label in missing:
        print("no source for README row:", label, file=sys.stderr)
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
