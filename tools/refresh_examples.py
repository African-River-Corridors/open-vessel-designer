#!/usr/bin/env python3
"""Regenerate, or check, the generated files of every worked example under examples/ (issues #822, #857).

    python3 tools/refresh_examples.py <path to ovd>             # results.json, ga/, trace.md, README tables
    python3 tools/refresh_examples.py --check <path to ovd>     # fail, naming each stale file, and change nothing
    python3 tools/refresh_examples.py --from-results FILE ...   # trace.md and the tables only, from a results file

Nothing is typed: `results.json` is `ovd run --json`, `ga/` is `ovd draw`, `trace.md` and the README
Results table come from `results.json`. The README rows that name a quantity are replaced from the
results; the prose around them is the author's. It fails when a table row has no source, and checks that
every Option's `hash_status` is "matches".

`--check` copies each example to a temporary folder, regenerates there, and compares every generated
file (results.json, trace.md, README.md, ga/*) with the committed one. A worked example is listed in
EXAMPLES; add the next one there.
"""
from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Worked examples: folder name, Design file, optional sensitivities Design (a second run for the README).
EXAMPLES = [("gas-140x29-coastal-lng", "gas-140x29-coastal-lng.vdesign", "gas-140x29-coastal-lng-sensitivities.vdesign")]
REFRESH = "python3 tools/refresh_examples.py <path to ovd>"


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


def update_sensitivities(DIR: Path, options: list[dict]) -> None:
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


def update_readme(DIR: Path, options: list[dict]) -> list[str]:
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


def refresh(ovd: str | None, directory: Path, design: str, sens: str | None, from_results: str | None = None) -> int:
    """Regenerate the generated files of one example folder in place."""
    if from_results:
        data = json.loads(Path(from_results).read_text())
    else:
        run = subprocess.run([ovd, "run", "--json", str(directory / design)], capture_output=True, text=True)
        if run.returncode != 0:
            print(f"ovd run failed with exit code {run.returncode}:\n{run.stderr}", file=sys.stderr)
            return 1
        data = json.loads(run.stdout)
        (directory / "results.json").write_text(run.stdout)
        subprocess.run([ovd, "draw", "--out", str(directory / "ga"), str(directory / design)], check=True, capture_output=True)
        for extra in (directory / "ga").glob("*"):      # the drawings kept are the PDF and SVG sheets
            if extra.suffix not in (".pdf", ".svg"):
                extra.unlink()
    bad = [o["name"] for o in data["options"] if o.get("hash_status") != "matches"]
    if bad:
        print("hash_status is not 'matches' for:", ", ".join(bad), file=sys.stderr)
        return 1
    (directory / "trace.md").write_text(trace_md(data["options"][0]))
    if ovd and sens:
        run = subprocess.run([ovd, "run", "--json", str(directory / sens)], capture_output=True, text=True)
        if run.returncode != 0:
            print(f"ovd run (sensitivities) failed with exit code {run.returncode}:\n{run.stderr}", file=sys.stderr)
            return 1
        sdata = json.loads(run.stdout)
        if any(o.get("hash_status") != "matches" for o in sdata["options"]):
            print("sensitivities: hash_status is not 'matches'", file=sys.stderr)
            return 1
        update_sensitivities(directory, sdata["options"])
    missing = update_readme(directory, data["options"])
    for label in missing:
        print("no source for README row:", label, file=sys.stderr)
    return 1 if missing else 0


def generated(directory: Path) -> dict[str, bytes]:
    """The generated files of an example folder, by relative path."""
    names = ["results.json", "trace.md", "README.md"]
    names += [f"ga/{p.name}" for p in (directory / "ga").glob("*")] if (directory / "ga").is_dir() else []
    return {n: (directory / n).read_bytes() for n in sorted(names) if (directory / n).is_file()}


def check(ovd: str) -> int:
    """Regenerate each example in a temporary copy; name every file that differs from the committed one."""
    stale: list[str] = []
    for name, design, sens in EXAMPLES:
        src = ROOT / "examples" / name
        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / name
            shutil.copytree(src, copy)
            if (copy / "ga").is_dir():
                shutil.rmtree(copy / "ga")      # a file the engine no longer draws must show up as stale
            if refresh(ovd, copy, design, sens):
                return 1
            old, new = generated(src), generated(copy)
            for rel in sorted(set(old) | set(new)):
                if old.get(rel) != new.get(rel):
                    stale.append(f"examples/{name}/{rel}" + ("" if rel in old else " (missing)") + ("" if rel in new else " (no longer generated)"))
    if stale:
        print("These generated example files are not what the engine draws now:", file=sys.stderr)
        for f in stale:
            print(f"  {f}", file=sys.stderr)
        print(f"Refresh them and commit the result:\n  {REFRESH}", file=sys.stderr)
        return 1
    print(f"examples: {len(EXAMPLES)} worked example(s), every generated file matches the engine")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("ovd", nargs="?")
    ap.add_argument("--from-results")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    if a.from_results:
        name, design, sens = EXAMPLES[0]
        return refresh(None, ROOT / "examples" / name, design, sens, a.from_results)
    if not a.ovd:
        ap.error("the path to ovd is needed")
    if a.check:
        return check(a.ovd)
    rc = 0
    for name, design, sens in EXAMPLES:
        rc |= refresh(a.ovd, ROOT / "examples" / name, design, sens)
    return rc


if __name__ == "__main__":
    sys.exit(main())
