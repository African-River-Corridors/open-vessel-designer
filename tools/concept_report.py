#!/usr/bin/env python3
"""A concept design report from an `ovd run --json` Option, as an Open Report Designer (ORD) document (issue #727).

Every concept report of a gas barge has the same parts. This script builds each of them from the run, so
that no figure is typed:

  particulars(o, ref)       main particulars, with a reference Option beside them when given
  lightship(o)              lightship by item (the items add to the lightship; checked)
  steel(o, status)          the hull steel figures (the panel model since #722, the top-down rate) and the gap
  hydrostatics(o)           a hydrostatic table by draught, with Cb
  stability(o)              the loading conditions table; gz_chart(o, path) draws the GZ curves
  powering(o)               installed power, speeds and the gas balance; POWERING_METHOD names the method
  compliance(o)             the compliance matrix: one row per governing or warning Rule, its worst outcome
  rule_paragraphs(o, notes) a short paragraph on each governing Rule, built from its trace lines
  not_evaluated(o)          the Rules the run does not evaluate
  provenance(...)           engine version, commit and results hashes
  ga_figures(...)           the GA sheets of the Option, drawn by `ovd draw`, as cropped PNGs
  build(run, option, ...)   the whole document: the summary, nine body parts, Appendix A (14 chapters) and Appendix B (8 sections)
  render(ord, doc, stem)    `ord check` and `ord render` to PDF, HTML and Word

A project's own narrative and trade studies stay outside this repo: import this module, call the parts,
and add sections around them.

    python3 tools/concept_report.py run.json --option "A basis" [--ref "B profile"] -o report.json
    python3 tools/concept_report.py run.json --option "A basis" -o report.json --render out/ [--ord ord]

`run.json` is the output of `ovd run --json <file.vdesign>`. An Option is one element of its "options"
list ({"name", "results_hash", "hash_status", "results": {...}}). The script reads only the run; it does
not call the engine, except `ga_figures`, which runs `ovd draw`.
"""
import argparse
import json
import pathlib
import re
import subprocess
import sys

NOTICE = "Concept and preliminary design only. Not verified. Never for approval."
OUTCOME_ORDER = ["fail", "warn", "not evaluated", "note", "pass", "not applicable"]
RULE_SET_NAMES = {"igc": "IGC Code", "is2008": "IS Code 2008", "es-trin": "ES-TRIN 2025/1", "icll": "ICLL",
                  "solas": "SOLAS", "lr-inland": "LR inland", "abs-barges": "ABS barges", "rheinschpv": "RheinSchPV",
                  "ovd": "OVD"}
POWERING_METHOD = (
    "Calm-water resistance is the ITTC-1957 friction line times (1 + Cr/Cf). Cr/Cf is a bluff-barge fit that depends "
    "on the Froude number only: an uncalibrated OVD estimate, not a published method. So the method sees a change of "
    "hull form only through the wetted surface and the waterline length. It is not Holtrop–Mennen; that regression "
    "covers Cp 0.73–0.85, L/B 5.1–7.1 and B/T 2.4–3.2 for full forms (Ortigosa et al., 2009).")


# --- ORD blocks ---

NOT_EVALUATED = "not evaluated"


def n(x, dp=0):
    """A number with thousands separators; a missing value prints "not evaluated", never blank or zero (spec 0015 section 5)."""
    return NOT_EVALUATED if x is None else f"{x:,.{dp}f}"


def table(tid, caption, columns, rows, notes=None, aligns=None, widths=None):
    cols = [{"title": c} for c in columns]
    for i, a in enumerate(aligns or []):
        if a:
            cols[i]["align"] = a
    for i, w in enumerate(widths or []):
        if w:
            cols[i]["width"] = w
    rows = [[NOT_EVALUATED if c is None or c == "" else c for c in row] for row in rows]
    t = {"type": "table", "id": tid, "caption": caption, "columns": cols, "rows": rows}
    if notes:
        t["notes"] = notes
    return t


def prose(md):
    return {"type": "prose", "markdown": md}


def figure(fid, caption, alt, width=1.0):
    return {"type": "figure", "id": fid, "caption": caption, "asset": fid, "alt": alt, "width": width}


def section(sid, title, body):
    return {"type": "section", "id": sid, "title": title, "body": body}


# --- the parts ---

def R(o):
    return o["results"]


def cond(o, name):
    return next(c for c in R(o)["stability"]["conditions"] if c["name"] == name)


def _get(d, *keys):
    """A value of a nested run block, or None when any step is missing."""
    for k in keys:
        if isinstance(d, (dict, list)) and (k in d if isinstance(d, dict) else -len(d) <= k < len(d)):
            d = d[k]
        else:
            return None
    return d


def _u(x, dp=0, unit=""):
    """A number with its unit; a missing value is "not evaluated" alone, never "not evaluated m"."""
    return NOT_EVALUATED if x is None else f"{n(x, dp)}{' ' + unit if unit else ''}"


def _particular_rows(o):
    r = R(o)
    d, res, g, h, p = r["dims"], r["results"], r["geometry"], r["hydrostatics"], r.get("powering")
    tanks = (NOT_EVALUATED if _get(g, "n_tanks_total") is None or _get(g, "tank_outer_diameter_m") is None else
             f"{g['n_tanks_total']} × IMO Type C, {n(g['tank_outer_diameter_m'], 2)} m OD")
    air = (NOT_EVALUATED if _get(res, "air_draft_loaded_m") is None or _get(res, "air_draft_light_m") is None else
           f"{n(res['air_draft_loaded_m'], 2)} / {n(res['air_draft_light_m'], 2)} m")
    cargo = (NOT_EVALUATED if _get(res, "cargo_volume_m3") is None or _get(res, "cargo_t") is None else
             f"{n(res['cargo_volume_m3'])} m³ / {n(res['cargo_t'])} t")
    return [
        ["Length overall", _u(_get(d, "loa_m"), 1, "m"), "Input"],
        ["Beam, moulded", _u(_get(d, "beam_m"), 1, "m"), "Input"],
        ["Depth, moulded", _u(_get(d, "depth_m"), 2, "m"), "Input"],
        ["Draught, laden (98 % fill)", _u(_get(res, "draught_m"), 2, "m"), "Hydrostatics"],
        ["Draught, light", _u(_get(res, "light_draught_m"), 2, "m"), "Hydrostatics"],
        ["Block coefficient Cb, laden", _u(_get(h, "cb"), 3), f"On Lwl {_u(_get(h, 'l_wl_m'), 1, 'm')} (ES-TRIN 1.01)"],
        ["Cargo tanks", tanks, "Fit search, IGC 2.4"],
        ["Cargo at 98 % fill", cargo, "Tank volume × fill × density"],
        ["Lightship", _u(_get(res, "lightship_t"), 0, "t"), "Build-up, uncalibrated"],
        ["Displacement, laden", _u(_get(res, "displacement_t"), 0, "t"), "Hydrostatics"],
        ["Freeboard, laden", _u(_get(res, "freeboard_m"), 2, "m"), "Depth less draught"],
        ["Air draught, laden / light", air, "Top of the weather shield"],
        ["Installed power", _u(_get(p, "installed_power_kw"), 0, "kW"), "Sized to the design speed × margin"],
        ["Service speed, deep water", _u(_get(p, "environments", 0, "service_speed_kn"), 1, "kn"), "Resistance at installed power"],
    ]


def particulars(o, ref=None, extra_rows=()):
    """Main particulars of Option o. With `ref`, a second value column gives the reference Option."""
    rows = _particular_rows(o)
    if ref is None:
        rows += [list(x) for x in extra_rows]
        return table("tbl-particulars", f"Main particulars, {o['name']}", ["Item", "Value", "Basis"], rows, aligns=[None, "right", None])
    rref = _particular_rows(ref)
    rows = [[a[0], a[1], b[1], a[2]] for a, b in zip(rows, rref)] + [list(x) for x in extra_rows]
    return table("tbl-particulars", f"Main particulars, {o['name']} (reference: {ref['name']})",
                 ["Item", o["name"], f"Reference: {ref['name']}", "Basis"], rows, aligns=[None, "right", "right", None],
                 widths=[0.30, 0.22, 0.22, 0.26])


def steel(o, status=None):
    """The hull steel figures. `status` labels them (for example a review). Since issue #722 the panel
    model (structural members with class minimum scantlings) drives the lightship and the cost; the
    top-down rate is printed for comparison only. A results file from before has no `panel_t`."""
    st = R(o)["cost_masses"]["steel"]
    td, bu = st["top_down_t"], st["bottom_up_t"]
    if "panel_t" in st:
        pn = st["panel_t"]
        rows = [["Panel model: members with class minimum scantlings", n(pn), "Drives the lightship and the cost"],
                ["Top-down: rate × L × B × D", n(td), "Comparison only"],
                ["Gap, top-down over panel", f"{n(td - pn)} ({td / pn - 1:+.0%})", "Reported"]]
    else:
        rows = [["Top-down: rate × L × B × D", n(td), "Drives the lightship"],
                ["Bottom-up: plating build-up", n(bu), "Prices the cost only"],
                ["Gap, top-down over bottom-up", f"{n(td - bu)} (+{td / bu - 1:.0%})", "Not resolved"]]
    cap = f"Hull steel: the figures OVD prints, {o['name']}" + (f", {status}" if status else "")
    return table("tbl-steel", cap, ["Figure", "Hull steel (t)", "Use"], rows, aligns=[None, "right", None],
                 notes=f"Both figures are {status}." if status else None)


def lightship_items(o):
    """The lightship by item. The items add to the lightship: the covering above D is the remainder, checked."""
    r = R(o)
    m, st = r["cost_masses"]["masses"], r["cost_masses"]["steel"]
    items = [
        ("Hull steel to the moulded depth", st.get("panel_t", st["top_down_t"]),
         "Panel model: members with class minimum scantlings" if "panel_t" in st else "Top-down: rate × L × B × D"),
        ("Cargo tanks, shells and supports", m["tank_shell_t"] + m["tank_supports_t"], "Tank shell × structure factor"),
        ("Tank insulation", m["insulation_t"], "Area × thickness × density"),
        ("Piping and machinery", m["piping_machinery_t"], "Factor on hull steel and tanks"),
        ("Outfitting", m["outfitting_t"], "Factor on hull steel and tanks"),
        ("Stern block: propulsors, gensets, deckhouse", r["stern_block_mass"]["total_t"], "Items, each placed"),
    ]
    rest = r["results"]["lightship_t"] - sum(v for _, v, _ in items)
    assert 0 <= rest < 0.1 * r["results"]["lightship_t"], f"covering {rest}"
    items.insert(5, ("Covering above the moulded depth: shield, domes", rest, "Plate areas above D"))
    return items


def lightship(o):
    items = lightship_items(o)
    total = R(o)["results"]["lightship_t"]
    rows = [[a, n(v), f"{v / total:.0%}", c] for a, v, c in items]
    rows.append(["Lightship", n(total), "100%", "Uncalibrated for a gas hull"])
    st = R(o)["cost_masses"]["steel"]
    return table("tbl-lightship", f"Lightship by item, {o['name']}", ["Item", "Mass (t)", "Share", "Basis"], rows,
                 notes=((f"The hull steel is the panel model ({n(st['panel_t'])} t): structural members with class minimum "
                         "scantlings, which drive the lightship and the cost. The top-down rate "
                         f"({n(st['top_down_t'])} t) is printed for comparison only. " if "panel_t" in st else
                         f"The hull steel has two figures in OVD. The lightship uses the top-down one ({n(st['top_down_t'])} t). "
                         f"The bottom-up plating build-up ({n(st['bottom_up_t'])} t) prices the cost only. ")
                        + "The covering above the moulded depth is weighed once, as its own items."),
                 aligns=[None, "right", "right", None])


def hydrostatics(o, draughts=None):
    ht = R(o)["hydrostatic_table"]
    if draughts:
        ht = [r for r in ht if any(abs(r["draught_m"] - d) < 1e-6 for d in draughts)]
    rows = [[n(r["draught_m"], 2), n(r["displacement_t"]), n(r["tpc_t"], 1), n(r["km_t_m"], 2), n(r["l_wl_m"], 1), n(r["cb"], 3)] for r in ht]
    return table("hydrostatic-table", f"Hydrostatics by draught, {o['name']}",
                 ["Draught (m)", "Displacement (t)", "TPC (t/cm)", "KMt (m)", "Lwl (m)", "Cb"], rows, aligns=["right"] * 6)


def _is_code_note(o):
    if not _has(R(o), "rule_trace"):
        return None   # no trace, no statement about the criteria
    rows = [r for r in R(o)["rule_trace"]["rows"] if r["rule"].startswith("is2008/")]
    crit = [r for r in rows if r["rule"].startswith("is2008/A.2.2")]
    bad = sorted({r["rule"] for r in crit if r["outcome"] in ("fail", "warn")})
    ne = sorted({r["rule"] for r in rows if r["outcome"] == "not evaluated"})
    s = "Criteria of IS Code 2008 Part A 2.2: " + ("all pass." if crit and not bad else f"not all pass ({', '.join(bad) or 'none run'}).")
    if ne:
        s += " Not evaluated: " + ", ".join(ne) + "."
    return s


def stability(o):
    rows = []
    for c in R(o)["stability"]["conditions"]:
        rows.append([c["name"], n(c["displacement_t"]), n(c["draught_m"], 2), n(c["kg_m"], 2), n(c["gm_fluid_m"], 2),
                     n(c["max_gz_m"], 2), f"{c['max_gz_deg']:.0f}°", f"{c['deck_edge_deg']:.1f}°"])
    return table("tbl-stability", f"Intact stability, the loading conditions, {o['name']}",
                 ["Condition", "Displ. (t)", "T (m)", "KG (m)", "GM fluid (m)", "Max GZ (m)", "At", "Deck edge"], rows,
                 notes=_is_code_note(o), aligns=[None] + ["right"] * 7)


def gz_chart(o, path):
    """Draw the GZ curves of every loading condition to `path` (PNG). Needs matplotlib."""
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    fig, ax = plt.subplots(figsize=(6.3, 2.7))
    for c in R(o)["stability"]["conditions"]:
        ax.plot([p[0] for p in c["gz"]], [p[1] for p in c["gz"]], lw=1.4, label=c["name"])
    ax.axhline(0.2, lw=0.7, ls=":", color="grey")
    ax.set_xlabel("Heel (°)")
    ax.set_ylabel("GZ (m)")
    ax.set_title(f"Righting lever curves, {o['name']}", loc="left", fontsize=10)
    ax.legend(frameon=False, fontsize=7)
    fig.tight_layout()
    fig.savefig(path, dpi=200)
    plt.close(fig)


def powering(o):
    p = R(o)["powering"]
    e = p["environments"][0]
    b = R(o)["boil_off"]
    prop = R(o)["drawing"]["propulsors"]
    rows = [
        ["Design speed", f"{n(p['design_speed_kn'], 1)} kn", "Input"],
        ["Installed power", f"{n(p['installed_power_kw'])} kW", f"Power at the design speed × {p['margin']}"],
        ["Service speed, deep water", f"{n(e['service_speed_kn'], 2)} kn", "Speed at the installed power ÷ margin"],
        ["Power at the service speed", f"{n(e['power_at_service_kw'])} kW", "Resistance ÷ propulsive efficiency"],
        ["Propulsors", f"{len(prop['propulsors'])} × {n(prop['diameter_m'], 1)} m", prop["propulsor_type"].replace("_", ", ")],
    ]
    if e.get("fuel_gas_kg_per_h") is not None:
        rows.append(["Gas use at the service speed", f"{n(e['fuel_gas_kg_per_h'])} kg/h", "Genset heat rate (see the run's fuel_gas basis)"])
    if b.get("evaluated", True) and b.get("boil_off_kg_per_h") is not None:
        rows.append(["Boil-off", f"{n(b['boil_off_kg_per_h'])} kg/h", "Tank heat leak (IGC Code 7.2)"])
    return table("tbl-powering", f"Powering and gas balance, {o['name']}", ["Item", "Value", "Basis"], rows, aligns=[None, "right", None])


# --- rules ---

def _rule_label(r, limit=58):
    s = r["title"].split(";")[0].split(", for ")[0].split(" (")[0]
    if len(s) <= limit:
        return s
    return s[:limit].rsplit(" ", 1)[0].rstrip(",") + " …"


def set_and_clause(rule_id):
    """("IGC Code", "2.4.1.2") from "igc/2.4.1.2/side-clearance"; OVD's own checks have no published clause."""
    head, clause = rule_id.split("/")[0], rule_id.split("/")[1]
    name = RULE_SET_NAMES.get(head, head.upper())
    if head == "ovd":
        clause = f"own check ({clause})"
    return name, clause


def _fmt(x, unit):
    u = "" if unit in ("", "-", None) else f" {unit}"
    if x is None:
        return NOT_EVALUATED
    return "—" if isinstance(x, bool) else f"{x:.4g}{u}"


def _outcome(r):
    if r["outcome"] == "pass" and r.get("margin") is not None and abs(r["margin"]) < 1e-6:
        return "pass, at limit"
    return r["outcome"]


def governing(o, include=("pass", "warn", "fail")):
    """One trace line per Rule id that governs, warns or fails: the worst outcome over its objects (ties: the smallest margin)."""
    by, count = {}, {}
    for r in R(o)["rule_trace"]["rows"]:
        if r["outcome"] not in include or (r["outcome"] == "pass" and not r.get("governing")):
            continue
        k = r["rule"]
        count[k] = count.get(k, 0) + 1
        key = (OUTCOME_ORDER.index(r["outcome"]), r["margin"] if r.get("margin") is not None else 0)
        if k not in by or key < by[k][0]:
            by[k] = (key, r)
    out = [dict(r, n_objects=count[k]) for k, (_, r) in by.items()]
    return sorted(out, key=lambda r: (OUTCOME_ORDER.index(r["outcome"]), r["rule"]))


def compliance(o):
    """The compliance matrix: rule, clause, requirement, value, limit, outcome, verified or not."""
    rows = []
    for r in governing(o):
        name, clause = set_and_clause(r["rule"])
        rows.append([name, clause, _rule_label(r), _fmt(r["value"], r["unit"]), _fmt(r["limit"], r["unit"]), _outcome(r),
                     "yes" if r["rule_status"] == "verified" else "no"])
    return table("tbl-compliance", f"Compliance matrix, {o['name']}: the governing Rules and every warning",
                 ["Rule", "Clause", "Requirement", "Value", "Limit", "Outcome", "Verified"], rows,
                 notes=("One row per Rule: the worst outcome over the tanks and conditions it applies to. "
                        "\"Verified\" says whether the Rule's wording has been checked against the published clause; "
                        "a Rule not yet verified still runs. The full trace (all Rules, all objects) is in the run's output."),
                 aligns=[None, None, None, "right", "right", None, None], widths=[0.12, 0.12, 0.34, 0.11, 0.11, 0.11, 0.09])


def rule_paragraphs(o, notes=None):
    """A short paragraph on each governing Rule, from its trace lines, plus an optional note per rule id.

    Returns markdown, one paragraph per Rule of `governing(o)`, in its order. Every fact comes from the trace:
    the requirement, blocking or warning, the worst value against the limit and the number of objects."""
    notes = notes or {}
    paras = []
    for r in governing(o):
        name, clause = set_and_clause(r["rule"])
        verb = {"ge": "at least", "le": "at most", "gt": "more than", "lt": "less than", "eq": "equal to"}.get(r["compare"], r["compare"])
        objs = f"worst of {r['n_objects']} {r['object_type']}s" if r["n_objects"] > 1 else r["object_type"]
        own = r["origin"] == "method-validity" or not r.get("citation")
        role = "; it sizes the design" if r.get("sizes") else ""
        bits = [f"**{name} {clause}.** {r['title'].split(';')[0]}.",
                f"{'Blocking' if r['consequence'] == 'block' else 'A warning only'}{' (OVD guide, no published clause)' if own else ''}{role}.",
                f"{_fmt(r['value'], r['unit'])} against {verb} {_fmt(r['limit'], r['unit'])} ({objs}): {_outcome(r)}."]
        if r["rule"] in notes:
            bits.append(notes[r["rule"]])
        paras.append(" ".join(bits))
    return "\n\n".join(paras)


def not_evaluated(o):
    seen = {}
    for r in R(o)["rule_trace"]["rows"]:
        if r["outcome"] == "not evaluated" and r["rule"] not in seen:
            seen[r["rule"]] = r
    return [f"{k}: {_rule_label(r, 90)}" for k, r in sorted(seen.items())]


def rule_counts(o):
    from collections import Counter
    return Counter(r["outcome"] for r in R(o)["rule_trace"]["rows"])


# --- drawings and provenance ---

def slug(option):
    """The file stem `ovd draw` gives an Option: lower case, every run of other characters a hyphen."""
    return re.sub(r"[^a-z0-9]+", "-", option.lower()).strip("-")


def ga_figures(ovd, vdesign, option, out_dir, crop=True):
    """Draw the Option's GA with `ovd draw` and render each sheet to PNG (needs rsvg-convert; cropping needs Pillow)."""
    out_dir.mkdir(parents=True, exist_ok=True)
    subprocess.run([str(ovd), "draw", "--option", option, "--out", str(out_dir), str(vdesign)], capture_output=True, check=True)
    pngs = {}
    for view in ("side-elevation", "plan", "midship-section"):
        svg = out_dir / f"{slug(option)}-{view}.svg"
        png = out_dir / f"{slug(option)}-{view}.png"
        subprocess.run(["rsvg-convert", "-w", "2400", "-b", "white", str(svg), "-o", str(png)], check=True)
        if crop:
            _crop_views(png)
        pngs[view] = png
    return pngs


def _crop_views(png):
    """Crop a GA sheet to its drawing: inside the frame, above the notes block."""
    from PIL import Image, ImageOps
    im = Image.open(png).convert("L")
    w, h = im.size
    box = (int(0.055 * w), int(0.045 * h), int(0.97 * w), int(0.64 * h))
    inner = im.crop(box)
    bb = ImageOps.invert(inner).point(lambda v: 255 if v > 60 else 0).getbbox()
    if bb:
        pad = 30
        bb = (max(bb[0] - pad, 0), max(bb[1] - pad, 0), min(bb[2] + pad, inner.width), min(bb[3] + pad, inner.height))
        Image.open(png).convert("RGB").crop((box[0] + bb[0], box[1] + bb[1], box[0] + bb[2], box[1] + bb[3])).save(png)


def provenance(engine_version, commit, options):
    rows = [[o["name"], (o["results_hash"][:12] if o.get("results_hash") else NOT_EVALUATED), (o.get("hash_status") or NOT_EVALUATED).replace("_", " ")] for o in options]
    note = f"Open Vessel Designer {engine_version}" + (f", commit {commit[:12]}." if commit else ".")
    return table("tbl-provenance", "The run behind every figure", ["Option", "Results hash", "Hash check"], rows, notes=note)


# --- the skeleton (spec 0015 sections 4, 5, 10) ---

# The statuses of ORD's `standard` lifecycle a concept report may carry, lowest first. Spec 0015 P7: it may be
# sent for approval and is never approved, so the cap is `for-approval` (the maintainer's decision; the ticket
# said "below"). `approved`, `issued` and `superseded` are refused.
WORKING_STATUSES = ["draft", "for-checking", "for-approval"]
STATUS_CAP = WORKING_STATUSES[-1]
STATUS_ALIASES = {"for-review": "for-checking", "for-sign-off": "for-approval"}
BODY_PAGE_LIMIT = 12   # spec 0015 section 4.1: a warning above this many body pages, never an error

# One row per Appendix A chapter, in the order of section 4.2: id, title, body part, the run fields it needs
# (a tuple is "any of"; a dotted name walks into a block), what it would show, what would close it, and the
# issue that builds it (None when this module builds it already).
CHAPTERS = [
    ("a1", "Brief", "p1", [], "what the vessel must do and under which rules", "nothing: the brief is the input", None),
    ("a2", "Main dimensions and form", "p1", ["dims", "hydrostatics", "geometry", "results", "checks"],
     "the size and shape, and what bounds each dimension", "a run that holds the dimensions, the hydrostatics and the geometry", None),
    ("a3", "Cargo system and capacity", "p2", ["results", "geometry", "spaces", "arrangement"],
     "how much cargo, in what, and whether volume or weight binds", "a run that holds the cargo results, the geometry and the Spaces", 830),
    ("a4", "General arrangement", "p2", ["drawing", "sightline"],
     "where everything is, as the general arrangement sheets", "the sheets drawn with `ovd draw` and passed to the build", None),
    ("a5", "Structure", "p3", ["cost_masses", "structure"], "the midship section, the scantlings and the hull steel",
     "a run that holds the hull steel masses", None),
    ("a6", "Powering, propulsion and machinery", "p3", ["powering", "boil_off", "drawing"],
     "how fast, on what power, with which propulsors, stern and machinery", "a run that holds the powering, the boil-off and the drawing blocks", None),
    ("a7", "Weights", "p4", ["cost_masses", "stern_block_mass", "results", "calibration"], "lightship by item, deadweight by item, centres",
     "a run that holds the mass build-up and the stern block", None),
    ("a8", "Hydrostatics and loading conditions", "p5", ["hydrostatic_table", "hydrostatics", "loading_conditions"], "how it floats in each condition",
     "a run that holds the hydrostatic table", None),
    ("a9", "Freeboard", "p5", ["results.freeboard_m"], "the freeboard carried against the freeboard required",
     "a run that holds the freeboard", 831),
    ("a10", "Intact stability", "p5", ["stability", "curves", "stability_marks"], "whether it stands up, condition by condition",
     "a run that holds the loading conditions and their GZ curves", None),
    ("a11", "Damage stability", "p6", [("damage", "marpol_damage")], "whether it survives the rule damages",
     "the engine computing damage cases for the hull type and rule set (spec 0012, issue #838)", 838),
    ("a12", "Compliance", "p7", ["rule_trace"], "every Rule, its outcome, and what was not evaluated",
     "a run that holds the rule trace", None),
    ("a13", "Cost", "p8", ["cost"], "what it costs, on which rates and price date", "a run that holds the cost block", 831),
    ("a14", "Limits and open items", "p9", [], "what could not be checked, and what would close it", "nothing: it is gathered from the run", None),
]

# The body parts, in the order of section 4.1: id, title, the chapters it digests.
BODY_PARTS = [
    ("p1", "The vessel and its brief", ["a1", "a2"]),
    ("p2", "Capacity and arrangement", ["a3", "a4"]),
    ("p3", "Structure, powering and machinery", ["a5", "a6"]),
    ("p4", "Weights", ["a7"]),
    ("p5", "Floating, freeboard and stability", ["a8", "a9", "a10"]),
    ("p6", "Damage stability", ["a11"]),
    ("p7", "Compliance", ["a12"]),
    ("p8", "Cost", ["a13"]),
    ("p9", "Limits and open items", ["a14"]),
]

# Appendix B, in the order of section 4.3: id, title, what it holds, the issue that builds it (None: built here).
EVIDENCE = [
    ("b1", "Run provenance", "engine version, commit, results hash and hash status per Option", None),
    ("b2", "Full rule trace", "every Rule row, clause by clause", 826),
    ("b3", "Calibration", "the evidence behind each calibrated method, with ranges", 826),
    ("b4", "Drawings", "the drawings list and the full sheets", 826),
    ("b5", "Full tables", "hydrostatics, loading condition items and Spaces", 826),
    ("b6", "Trade studies", "trade studies, when supplied", None),
    ("b7", "Sources", "documents cited, by edition and paragraph", 826),
    ("b8", "Symbols and terms", "the glossary terms", 826),
]


def resolve_status(status):
    """A status as ORD names it: case, spaces and underscores do not matter; the one alias of the cap's range resolves."""
    s = re.sub(r"[\s_]+", "-", str(status).strip().lower())
    return STATUS_ALIASES.get(s, s)


def check_status(status):
    """The ORD status of a concept report, or ValueError. A concept report never passes `STATUS_CAP` (spec 0015 P7)."""
    s = resolve_status(status)
    if s not in WORKING_STATUSES:
        raise ValueError(f"status {status!r} is above the cap: a concept report is at most {STATUS_CAP!r} "
                         f"(ADR 0011: concept and preliminary design, never approval); allowed: {', '.join(WORKING_STATUSES)}")
    return s


def _has(r, spec):
    """True when the run block `spec` (a dotted name, or a tuple of them for any-of) is present and not null."""
    if isinstance(spec, tuple):
        return any(_has(r, x) for x in spec)
    cur = r
    for part in spec.split("."):
        if not isinstance(cur, dict) or cur.get(part) is None:
            return False
        cur = cur[part]
    return True


def _names(spec):
    return " or ".join(f"`{x}`" for x in spec) if isinstance(spec, tuple) else f"`{spec}`"


def _brief(o):
    r, t = R(o), R(o).get("rule_trace") or {}
    sets = t.get("rule_sets")
    rows = [["Vessel", r.get("vessel")], ["Cargo", r.get("cargo")], ["Phase", r.get("phase")],
            ["Service Area", t.get("service_area")],
            ["Rule Sets", ", ".join(sets) if isinstance(sets, list) else sets]]
    return [table("tbl-brief", f"The brief, {o['name']}", ["Item", "Value"], rows)]


def _ga(assets):
    out = []
    for fid, cap in (("ga-side", "Side elevation"), ("ga-mid", "Midship section")):
        if fid in assets:
            out.append(figure(fid, f"{cap} (drawn by OVD)", f"{cap} drawing of the Option"))
    return out


def _compliance_blocks(o):
    ne = not_evaluated(o)
    counts = rule_counts(o)
    return [prose(f"{counts['pass']} trace lines pass, {counts['warn']} warn, {counts.get('fail', 0)} fail and "
                  f"{counts['not evaluated']} are not evaluated. The matrix shows each governing Rule and every warning once."),
            compliance(o),
            prose("**The governing Rules, one by one.**\n\n" + rule_paragraphs(o)),
            prose("**Rules not evaluated in this run:**\n\n" + ("\n".join(f"- `{x}`" for x in ne) or "- none"))]


def _chapter_builders(o, ref, assets):
    """The chapters this module builds today: id -> a function giving blocks (an empty list: nothing to show)."""
    def stab():
        out = [stability(o)]
        if "gz" in assets:
            out.append(figure("gz", "Righting lever (GZ) curves", "GZ curves of the loading conditions"))
        return out
    return {
        "a1": lambda: _brief(o),
        "a2": lambda: [particulars(o, ref)],
        "a4": lambda: _ga(assets),
        "a5": lambda: [steel(o)],
        "a6": lambda: [prose(f"**How OVD works out the power.** {POWERING_METHOD}"), powering(o)],
        "a7": lambda: [lightship(o)],
        "a8": lambda: [hydrostatics(o)],
        "a10": stab,
        "a12": lambda: _compliance_blocks(o),
    }


def chapter_states(o, ref=None, assets=None):
    """Each Appendix A chapter's state: {"blocks": [...]} or {"why": reason}. A chapter whose run fields are
    missing, that this module does not build yet, or that has nothing to show, is not evaluated."""
    r = R(o)
    builders = _chapter_builders(o, ref, assets or {})
    states = {}
    for cid, _title, _part, needs, _shows, _closes, ticket in CHAPTERS:
        missing = [x for x in needs if not _has(r, x)]
        if missing:
            states[cid] = {"why": _missing_reason(o, cid, missing)}
        elif cid == "a14":
            states[cid] = {"blocks": []}   # gathered in build(), from the other states
        elif cid in builders:
            blocks = builders[cid]()
            states[cid] = {"blocks": blocks} if blocks else {"why": "No general arrangement sheet was drawn for this build."}
        else:
            states[cid] = {"pending": ticket}
    return states


def _missing_reason(o, cid, missing):
    """Why a chapter is not evaluated. For damage stability it is the engine gap the trace names, not a JSON key."""
    if cid == "a11":
        for r in R(o).get("rule_trace", {}).get("rows", []):
            if r["outcome"] == "not evaluated" and (r.get("measure") or {}).get("id") == "stability.damage" and r.get("gap"):
                return f"{r['gap'][0].upper()}{r['gap'][1:]} ({r['rule']}). Issue #838 builds it."
        return "The engine holds no damage cases for this hull type and rule set (spec 0012). Issue #838 builds it."
    return "The run holds no " + ", ".join(_names(x) for x in missing) + "."


PENDING = "Not yet in this report"


def pending_block(what, ticket):
    """A chapter or section the run holds the data for and the report build does not hold yet. It is not "not evaluated"."""
    return prose(f"**{PENDING}.**\n\nThe run holds the data. This report build does not show {what} yet. "
                 f"Issue #{ticket} builds it.")


def not_evaluated_block(shows, why, closes):
    """The fixed appendix block of spec 0015 section 5: what the part would show, why it is not evaluated, what would close it."""
    return prose(f"**Not evaluated.**\n\n- What this part would show: {shows}.\n- Why it is not evaluated: {why}\n"
                 f"- What would close it: {closes}.")


def _rule_gaps(o):
    seen = {}
    for r in R(o).get("rule_trace", {}).get("rows", []):
        if r["outcome"] == "not evaluated" and r["rule"] not in seen:
            seen[r["rule"]] = r
    return [[f"Rule {k}: {_rule_label(r, 90)}", (r.get("gap") or "The run does not evaluate it.").rstrip(".") + ".",
             "The engine capability the gap names" + (f" (issue {r['finding']})" if r.get("finding") else "")]
            for k, r in sorted(seen.items())]


def open_items(o, states, tid):
    """The open-items table (A.14 and body part 9): every not-evaluated chapter and Rule, with why and how it closes."""
    rows = [[f"Chapter {title}", states[cid]["why"], closes]
            for cid, title, _p, _n, _s, closes, _t in CHAPTERS if "why" in states[cid]]
    rows += _rule_gaps(o)
    if not rows:
        return prose("Nothing in this run is not evaluated.")
    return table(tid, f"Open items, {o['name']}: what the run did not evaluate", ["Item", "Why", "What would close it"], rows,
                 notes="The table states facts from the run. It asks nothing.", widths=[0.30, 0.40, 0.30])


def _part(o, pid, title, chapter_ids, states):
    by = {c[0]: c for c in CHAPTERS}
    lines, items = [], []
    for cid in chapter_ids:
        _id, ctitle, _p, needs, _s, _c, _t = by[cid]
        if "why" in states[cid]:
            lines.append(f"**Not evaluated.** {ctitle}: {states[cid]['why']} See [[#{cid}]].")
            items.append({"label": ctitle, "value": NOT_EVALUATED})
        elif "pending" in states[cid]:
            lines.append(f"**{PENDING}.** {ctitle}: issue #{states[cid]['pending']} builds it. See [[#{cid}]].")
            items.append({"label": ctitle, "value": PENDING.lower()})
        else:
            items.append({"label": ctitle, "value": ", ".join(_names(x).replace("`", "") for x in needs) or "the run's inputs"})
    ok = [c for c in chapter_ids if "blocks" in states[c]]   # pending chapters are not listed as built
    if ok:
        lines.append("The full " + ("chapter is " if len(ok) == 1 else "chapters are ") + " and ".join(f"[[#{c}]]" for c in ok) + ".")
    body = [prose("\n\n".join(lines))]
    if pid == "p9":
        body.append(open_items(o, states, "tbl-open-items-body"))
    body.append({"type": "data_box", "id": f"data-{pid}", "title": "Data and limits", "items": items})
    return section(pid, title, body)


def _summary(o):
    if not _has(R(o), "rule_trace"):
        return [prose("**Not evaluated.** The run holds no `rule_trace`, so there is no verdict.")]
    c = rule_counts(o)
    return [prose(f"Trace verdict: {c['pass']} pass, {c['warn']} warn, {c.get('fail', 0)} fail and {c['not evaluated']} not evaluated.")]


def build(run, option, ref=None, number=None, revision="A", date="1970-01-01", title=None, commit=None, assets=None,
          status="draft"):
    """The concept report of one Option of `run` (an `ovd run --json` output) as an ORD document.

    The document has the three zones of spec 0015 section 4, with every heading always present: the summary and
    nine body parts (under one level-1 section), Appendix A (chapters A.1 to A.14) and Appendix B (sections B.1 to B.8). A chapter or part the
    run cannot fill carries the "not evaluated" block (section 5). `ref` names a reference Option shown beside the
    main one. `assets` maps asset ids to PNG paths: "gz" adds the GZ chart, "ga-side" and "ga-mid" add the GA views.
    `status` is an ORD status; one above `STATUS_CAP` raises ValueError (P7). The control block holds the revision
    letter, the date, the status and the number; it has no change table (section 10)."""
    o = next(x for x in run["options"] if x["name"] == option)
    r = next(x for x in run["options"] if x["name"] == ref) if ref else None
    assets = assets or {}
    status = check_status(status)
    if not re.fullmatch(r"[A-Z]{1,2}", str(revision)):
        raise ValueError(f"revision {revision!r} must be a letter: a concept report never passes a working status")
    states = chapter_states(o, r, assets)
    states["a14"] = {"blocks": [open_items(o, states, "tbl-open-items")]}

    # One level-1 section holds the summary and the nine parts, so they flow without page breaks (spec 0015
    # section 4.1); the chapters are the appendices, which stay level 1.
    parts = [dict(section("summary", "Summary", _summary(o)), numbered=False)]
    parts += [_part(o, pid, ptitle, cids, states) for pid, ptitle, cids in BODY_PARTS]
    body = [section("body", "The concept design", parts)]

    chapters = []
    for cid, ctitle, _p, _n, shows, closes, _t in CHAPTERS:
        st = states[cid]
        if "blocks" in st:
            blocks = st["blocks"]
        elif "pending" in st:
            blocks = [pending_block(shows, st["pending"])]
        else:
            blocks = [not_evaluated_block(shows, st["why"], closes)]
        chapters.append(section(cid, ctitle, blocks))

    evidence = []
    for bid, btitle, holds, ticket in EVIDENCE:
        if bid == "b1":
            blocks = [provenance(run.get("engine_version", NOT_EVALUATED), commit, [o] + ([r] if r else []))]
        elif bid == "b6":
            blocks = [prose("None supplied.")]
        else:
            blocks = [pending_block(holds, ticket)]
        evidence.append(section(bid, btitle, blocks))

    control = {"revision": revision, "date": date, "status": status}
    if number:
        control["number"] = number
    doc = {
        "ord": "0.2", "class": "technical-note",
        "title": title or f"Concept design report: {o['name']}",
        "language": "en-GB", "contents": True, "notice": NOTICE, "control": control,
        "body": body,
        "appendices": [{"id": "appendix-a", "title": "The technical chapters", "body": chapters},
                       {"id": "appendix-b", "title": "The evidence", "body": evidence}],
    }
    if assets:
        doc["assets"] = {k: {"media_type": "image/png", "path": str(p)} for k, p in assets.items()}
    return doc


# --- the page budget (spec 0015 section 4.1) ---

def pdf_pages(pdf):
    """The page count of a PDF file, read with `pdfinfo` (poppler)."""
    out = subprocess.run(["pdfinfo", str(pdf)], capture_output=True, text=True, check=True).stdout
    return int(re.search(r"^Pages:\s+(\d+)", out, re.M).group(1))


def body_pages(doc, ord_bin, work_dir):
    """The pages of the report without its appendices: the cover, the contents and the body, rendered to PDF."""
    work_dir = pathlib.Path(work_dir)
    work_dir.mkdir(parents=True, exist_ok=True)
    src = work_dir / "body-only.json"
    # The appendices go; a cross-reference into them becomes a text of the length ORD prints, so the pages stay the same.
    text = re.sub(r"\[\[#[^\]]+\]\]", "Appendix A.14", json.dumps({k: v for k, v in doc.items() if k != "appendices"}, ensure_ascii=False))
    src.write_text(text, encoding="utf-8")
    subprocess.run([str(ord_bin), "render", str(src), "--format", "pdf", "-o", str(work_dir)], check=True, capture_output=True)
    return pdf_pages(work_dir / "body-only.pdf")


def page_budget_warnings(pages, limit=BODY_PAGE_LIMIT):
    """A warning when the body is longer than `limit` pages; none at `limit` pages or fewer. Never an error."""
    if pages > limit:
        return [f"the body is {pages} pages; the budget is {limit} (spec 0015 section 4.1)"]
    return []

def render(ord_bin, doc, out_stem):
    """Write `<stem dir>/report.json`, check it with `ord check`, and render PDF, HTML and Word beside `out_stem`."""
    out_stem = pathlib.Path(out_stem)
    src = out_stem.parent / "report.json"
    src.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    subprocess.run([str(ord_bin), "check", str(src)], check=True)
    tmp = out_stem.parent / ".render"
    tmp.mkdir(exist_ok=True)
    subprocess.run([str(ord_bin), "render", str(src), "--format", "all", "-o", str(tmp)], check=True)
    written = []
    for ext in ("pdf", "html", "docx"):
        p = tmp / f"report.{ext}"
        if p.exists():
            dest = pathlib.Path(f"{out_stem}.{ext}")
            dest.write_bytes(p.read_bytes())
            written.append(dest)
    return written


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("run", type=pathlib.Path, help="the output of `ovd run --json <file.vdesign>`")
    ap.add_argument("--option", required=True)
    ap.add_argument("--ref", help="a reference Option shown beside the main one")
    ap.add_argument("--number")
    ap.add_argument("--revision", default="A")
    ap.add_argument("--date", default="1970-01-01")
    ap.add_argument("--status", default="draft", help=f"an ORD status, at most {STATUS_CAP!r} (a concept report is never for approval)")
    ap.add_argument("--title")
    ap.add_argument("--commit", help="the engine commit the run was made at")
    ap.add_argument("-o", "--out", type=pathlib.Path, required=True, help="the ORD document to write")
    ap.add_argument("--render", type=pathlib.Path, help="also render PDF, HTML and Word into this folder")
    ap.add_argument("--ord", default="ord", help="the ord binary (for --render)")
    a = ap.parse_args(argv)
    run = json.loads(a.run.read_text(encoding="utf-8"))
    doc = build(run, a.option, a.ref, a.number, a.revision, a.date, a.title, a.commit, status=a.status)
    a.out.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    if a.render:
        a.render.mkdir(parents=True, exist_ok=True)
        for p in render(a.ord, doc, a.render / slug(a.option)):
            print(p)
        for w in page_budget_warnings(body_pages(doc, a.ord, a.render / ".budget")):
            print(f"warning: {w}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
