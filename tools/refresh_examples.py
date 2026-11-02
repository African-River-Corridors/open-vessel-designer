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
# Options of a sensitivities file that the README does not tabulate, by example folder: a name pattern.
# Any other Option with no README row fails. The block-coefficient sweeps (N1 to N9, P1 to P9) are run, not tabled.
UNTABULATED = {"gas-140x29-coastal-lng": r"[NP][1-9] Cb \d\.\d+"}
REFRESH = "python3 tools/refresh_examples.py <path to ovd>"


def unhandled_examples(root: Path | None = None) -> list[str]:
    """Folders under examples/ with a Design that EXAMPLES does not name, and Designs in a listed folder it does not name."""
    root = root or ROOT
    known = {name: {design, sens} for name, design, sens in EXAMPLES}
    problems = []
    for folder in sorted(p for p in (root / "examples").glob("*") if p.is_dir()):
        designs = {f.name for f in folder.glob("*.vdesign")}
        if not designs:
            continue
        if folder.name not in known:
            problems.append(f"examples/{folder.name}/ holds a Design but is not in EXAMPLES in tools/refresh_examples.py")
        else:
            problems += [f"examples/{folder.name}/{d} is not named in EXAMPLES" for d in sorted(designs - known[folder.name])]
    return problems


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


def update_sensitivities(DIR: Path, options: list[dict], untabulated: str | None = None) -> list[str]:
    """Replace the Tank OD, Cargo and Draught cells of the sensitivities table, by Option name.

    Returns the problems: a table row that no Option matches or that has the wrong number of cells, and
    an Option with no row. Nothing is skipped silently."""
    path = DIR / "README.md"
    by_name = {o["name"]: o["results"] for o in options}
    text = path.read_text()
    header = "| Option | Change from A |"
    if header not in text:
        return [f"{path.name}: no sensitivities table (a header row starting '{header}')"]
    start = text.index(header)
    end = text.find("\n\n", start)
    end = len(text) if end < 0 else end
    problems, seen, out = [], set(), []
    for i, line in enumerate(text[start:end].split("\n")):
        if i >= 2:      # after the header and the separator
            cells = [c.strip() for c in line.strip("|").split("|")]
            if len(cells) != 6:
                problems.append(f"sensitivities row has {len(cells)} cells, not 6: {line}")
            elif cells[0] not in by_name:
                problems.append(f"sensitivities row '{cells[0]}' matches no Option of the sensitivities file")
            else:
                r = by_name[cells[0]]
                res = r["results"]
                seen.add(cells[0])
                cells[2:] = [
                    n(r["arrangement"]["tank_od_m"], 2),
                    n(res["cargo_volume_m3"]),
                    n(res["cargo_t"]),
                    n(res["draught_m"], 2),
                ]
                line = "| " + " | ".join(cells) + " |"
        out.append(line)
    problems += [f"sensitivities Option '{name}' has no row in the README table" for name in by_name if name not in seen and not (untabulated and re.fullmatch(untabulated, name))]
    path.write_text(text[:start] + "\n".join(out) + text[end:])
    return problems


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
        problems = update_sensitivities(directory, sdata["options"], UNTABULATED.get(directory.name))
        for problem in problems:
            print(problem, file=sys.stderr)
        if problems:
            return 1
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
    if a.check and a.from_results:
        ap.error("--check and --from-results cannot be used together (--from-results rewrites files)")
    if a.from_results:
        name, design, sens = EXAMPLES[0]
        return refresh(None, ROOT / "examples" / name, design, sens, a.from_results)
    if not a.ovd:
        ap.error("the path to ovd is needed")
    if unhandled := unhandled_examples():
        print("\n".join(unhandled), file=sys.stderr)
        return 1
    if a.check:
        return check(a.ovd)
    rc = 0
    for name, design, sens in EXAMPLES:
        rc |= refresh(a.ovd, ROOT / "examples" / name, design, sens)
    return rc


if __name__ == "__main__":
    sys.exit(main())
