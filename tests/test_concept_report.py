"""The concept design report script builds every part from the run (issue #727).

`tools/concept_report.py` turns one `ovd run --json` Option into an Open Report Designer document. Risk
stopped: a report part says something the run does not, a typed figure creeps in, a warning Rule drops out
of the compliance matrix, or the resistance method is named as something the engine does not run.

The fixture is the gas example's stored run, `examples/gas-140x29-coastal-lng/results.json`.

Label: property. Each test checks the report against the run it is built from; no number is pinned.
"""
import importlib.util
import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("concept_report", ROOT / "tools" / "concept_report.py")
cr = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cr)

RUN = json.loads((ROOT / "examples" / "gas-140x29-coastal-lng" / "results.json").read_text(encoding="utf-8"))
A = next(o for o in RUN["options"] if o["name"] == "A basis")
C = next(o for o in RUN["options"] if o["name"] == "C deep")


def test_lightship_items_add_to_the_lightship():
    """Label: property"""
    items = cr.lightship_items(A)
    total = A["results"]["results"]["lightship_t"]
    assert abs(sum(v for _, v, _ in items) - total) < 1e-6
    assert all(v >= 0 for _, v, _ in items)
    table = cr.lightship(A)
    assert table["rows"][-1][1] == cr.n(total)


def test_particulars_come_from_the_run_and_show_the_reference_beside_them():
    """Label: property"""
    one = cr.particulars(A)
    assert [c["title"] for c in one["columns"]] == ["Item", "Value", "Basis"]
    two = cr.particulars(C, ref=A)
    assert [c["title"] for c in two["columns"]] == ["Item", "C deep", "Reference: A basis", "Basis"]
    depth = next(r for r in two["rows"] if r[0] == "Depth, moulded")
    assert depth[1] == f"{C['results']['dims']['depth_m']:,.2f} m"
    assert depth[2] == f"{A['results']['dims']['depth_m']:,.2f} m"


def test_the_steel_table_gives_both_figures_and_their_gap():
    """Label: property"""
    st = A["results"]["cost_masses"]["steel"]
    t = cr.steel(A, "under review")
    # Issue #722: the panel model drives the lightship and the cost; the top-down rate is beside it.
    assert t["rows"][0][1] == cr.n(st["panel_t"])
    assert t["rows"][1][1] == cr.n(st["top_down_t"])
    assert t["rows"][2][1].startswith(cr.n(st["top_down_t"] - st["panel_t"]))
    assert "under review" in t["caption"]


def test_the_matrix_has_one_row_per_governing_or_warning_rule_with_its_worst_outcome():
    """Label: property"""
    rows = A["results"]["rule_trace"]["rows"]
    want = {r["rule"] for r in rows if r["outcome"] in ("warn", "fail") or (r["outcome"] == "pass" and r.get("governing"))}
    got = cr.governing(A)
    assert sorted(r["rule"] for r in got) == sorted(want)
    for g in got:
        outcomes = [r["outcome"] for r in rows if r["rule"] == g["rule"] and r["outcome"] in ("pass", "warn", "fail")]
        assert g["outcome"] == min(outcomes, key=cr.OUTCOME_ORDER.index)
    m = cr.compliance(A)
    assert [c["title"] for c in m["columns"]] == ["Rule", "Clause", "Requirement", "Value", "Limit", "Outcome", "Verified"]
    assert len(m["rows"]) == len(want)
    assert {r[6] for r in m["rows"]} <= {"yes", "no"}


def test_every_governing_rule_gets_one_paragraph_and_a_note_is_added_where_given():
    """Label: property"""
    gov = cr.governing(A)
    first = gov[0]["rule"]
    text = cr.rule_paragraphs(A, {first: "A note for this rule."})
    paras = text.split("\n\n")
    assert len(paras) == len(gov)
    assert paras[0].endswith("A note for this rule.")
    for p, g in zip(paras, gov):
        name, clause = cr.set_and_clause(g["rule"])
        assert p.startswith(f"**{name} {clause}.**")


def test_the_powering_method_is_named_as_the_engine_runs_it():
    """Label: property"""
    m = cr.POWERING_METHOD
    assert "ITTC-1957" in m and "Cr/Cf" in m
    assert "Holtrop–Raven" not in m and "Holtrop-Raven" not in m
    assert "no hull-form term" in m or "only through the wetted surface" in m


def test_not_evaluated_lists_each_rule_once():
    """Label: property"""
    ne = cr.not_evaluated(A)
    ids = [x.split(":")[0] for x in ne]
    assert len(ids) == len(set(ids))
    assert set(ids) == {r["rule"] for r in A["results"]["rule_trace"]["rows"] if r["outcome"] == "not evaluated"}


# --- the skeleton (issue #825, spec 0015 sections 4, 5, 9, 10, 11) ---

ORD = shutil.which("ord")
needs_ord = pytest.mark.skipif(ORD is None, reason="ord is not on the path")
CHAPTER_TITLES = {c[0]: c[1] for c in cr.CHAPTERS}


def _ids(doc):
    """Every section id of the document, body then appendices, in order."""
    def walk(blocks):
        for b in blocks:
            if b["type"] == "section":
                yield b["id"]
                yield from walk(b["body"])
    out = list(walk(doc["body"]))
    for a in doc["appendices"]:
        out.append(a["id"])
        out += walk(a["body"])
    return out


def _section(doc, sid):
    def walk(blocks):
        for b in blocks:
            if b["type"] == "section":
                if b["id"] == sid:
                    return b
                hit = walk(b["body"])
                if hit:
                    return hit
    return walk(doc["body"]) or next(h for a in doc["appendices"] if (h := walk(a["body"])))


def _text(block):
    return json.dumps(block, ensure_ascii=False)


def _run_without(path):
    if isinstance(path, tuple):
        run = RUN
        for p in path:
            run = _run_without_one(run, p)
        return run
    return _run_without_one(RUN, path)


def _run_without_one(base, path):
    run = json.loads(json.dumps(base))
    for o in run["options"]:
        res = o["results"]
        keys = path.split(".")
        for k in keys[:-1]:
            res = res[k]
        res.pop(keys[-1], None)
    return run


def test_the_document_has_every_heading_of_the_skeleton_in_order():
    """Label: property"""
    doc = cr.build(RUN, "A basis", number="TN-0001", date="2026-10-30")
    assert doc["ord"] == "0.2"
    assert [s["id"] for s in doc["body"]] == ["body"]
    parts = doc["body"][0]["body"]
    assert [s["id"] for s in parts] == ["summary"] + [f"p{i}" for i in range(1, 10)]
    assert parts[0]["numbered"] is False
    assert [s["title"] for s in parts[1:]] == [
        "The vessel and its brief", "Capacity and arrangement", "Structure, powering and machinery", "Weights",
        "Floating, freeboard and stability", "Damage stability", "Compliance", "Cost", "Limits and open items"]
    a, b = doc["appendices"]
    assert [s["id"] for s in a["body"]] == [f"a{i}" for i in range(1, 15)]
    assert [s["title"] for s in a["body"]][:4] == ["Brief", "Main dimensions and form", "Cargo system and capacity", "General arrangement"]
    assert [s["id"] for s in b["body"]] == [f"b{i}" for i in range(1, 9)]
    assert [s["title"] for s in b["body"]] == ["Run provenance", "Full rule trace", "Calibration", "Drawings", "Full tables",
                                                "Trade studies", "Sources", "Symbols and terms"]
    assert doc["notice"] == cr.NOTICE
    prov = _section(doc, "b1")["body"][0]
    assert prov["rows"][0][1] == A["results_hash"][:12]
    assert cr.build(RUN, "A basis", date="2026-10-30") == cr.build(RUN, "A basis", date="2026-10-30")


# Which chapters read each top-level run block, written here from spec 0015 section 4.2 and not from CHAPTERS.
READ_BY = {
    "results": ["a2", "a3", "a7", "a9"], "dims": ["a2"], "hydrostatics": ["a2", "a8"], "geometry": ["a2", "a3"],
    "checks": ["a2"], "spaces": ["a3"], "arrangement": ["a3"], "drawing": ["a4", "a6"], "sightline": ["a4"],
    "cost_masses": ["a5", "a7"], "structure": ["a5"], "powering": ["a6"], "boil_off": ["a6"],
    "stern_block_mass": ["a7"], "calibration": ["a7"], "hydrostatic_table": ["a8"], "loading_conditions": ["a8"],
    "stability": ["a10"], "curves": ["a10"], "stability_marks": ["a10"], "rule_trace": ["a12"], "cost": ["a13"],
    "vessel": [], "cargo": [], "phase": [],   # A.1 prints a cell "not evaluated" for these
    # no chapter reads these yet: a deletion must change nothing in the skeleton
    "self_propelled": [], "warnings": [], "fit_tried": [], "text_ids": [],
    "gross_tonnage": [], "holding_time": [],   # the holding-time block (#870); a chapter reads it with #871
    "crew": [],   # no chapter reads it yet; #830 maps it to A4
    "boil_off_detail": [],   # unhashed (#919): the U-value and the heat leak per tank; no chapter reads it yet
    "air_drafts": [],   # unhashed (#888): the app and the side elevation read it; no chapter yet
    "air_draft_report": [],   # unhashed (#991): the air draft, the highest point and the bridge limit; no chapter yet
    "stern_arrangement": [],   # the Stern & crew view reads it (#887); no chapter does yet
    "damage_status": ["a11"],   # whether damage ran (#923): A.11 gives it as its reason; the summary states it too
    # `damage_cases` (#923) is in a run only when damage ran or was skipped; the stored example has neither
}
GA = {"ga-side": "side.png", "ga-mid": "mid.png"}   # so A.4 is built, and its deletion shows


def test_every_top_level_run_block_is_accounted_for():
    """Label: property"""
    assert set(A["results"]) == set(READ_BY)


@pytest.mark.parametrize("key", sorted(READ_BY))
def test_deleting_a_run_block_keeps_every_heading_and_reads_not_evaluated_in_three_places(key):
    """Label: property
    Spec 0015 section 11, test 4, for every block of the run: the body line, the part 9 row and the
    appendix block of each chapter that reads it, and no change in a chapter that does not.
    """
    base = cr.build(RUN, "A basis", assets=GA)
    doc = cr.build(_run_without(key), "A basis", assets=GA)
    assert _ids(doc) == _ids(base)
    for chapter in cr.CHAPTERS:
        cid, title, part = chapter[0], chapter[1], chapter[2]
        md = _section(doc, cid)["body"][0]
        was = _section(base, cid)["body"][0]
        if cid in READ_BY[key]:
            assert md["markdown"].startswith("**Not evaluated.**"), cid
            body_line = _section(doc, part)["body"][0]["markdown"]
            assert f"**Not evaluated.** {title}:" in body_line and f"[[#{cid}]]" in body_line
            assert {i["label"]: i["value"] for i in _section(doc, part)["body"][-1]["items"]}[title] == "not evaluated"
            for holder in ("p9", "a14"):
                tables = [b for b in _section(doc, holder)["body"] if b["type"] == "table"]
                assert any(r[0] == f"Chapter {title}" for t in tables for r in t["rows"]), (holder, cid)
        elif (key, cid) == ("powering", "a2"):   # the particulars table has power and speed cells
            assert [r[1] for r in md["rows"] if r[0] in ("Installed power", "Service speed, deep water")] == ["not evaluated"] * 2
        elif (key, cid) == ("rule_trace", "a10"):   # the table's note on the IS Code criteria needs the trace
            assert "notes" not in md and md["rows"] == was["rows"]
        elif (key, cid) == ("rule_trace", "a11"):   # the reason is the trace's gap; without it, the general reason
            assert md["markdown"].startswith("**Not evaluated.**") and "#838" in md["markdown"]
        elif cid != "a14" and cid not in ("a1",) and key not in ("vessel", "cargo", "phase"):
            assert md == was, cid
    if key in ("vessel", "cargo", "phase"):
        rows = _section(doc, "a1")["body"][0]["rows"]
        assert any(r[1] == "not evaluated" for r in rows) and all(c not in ("", None) for r in rows for c in r)


def test_a_chapter_the_run_can_fill_but_the_build_lacks_is_not_yet_in_this_report_not_not_evaluated():
    """Label: property
    A.3, A.9 and A.13 and the Appendix B sections B.2 to B.5, B.7 and B.8 carry their own state. It names the
    issue that builds them, says the run holds the data, and never reaches the not-evaluated list.
    """
    doc = cr.build(RUN, "A basis")
    want = {"a3": 830, "a9": 831, "a13": 831, **{b: 826 for b in ("b2", "b3", "b4", "b5", "b7", "b8")}}
    for sid, ticket in want.items():
        md = _section(doc, sid)["body"][0]["markdown"]
        assert md.startswith("**Not yet in this report.**") and f"#{ticket}" in md and "The run holds the data" in md
        assert "Not evaluated" not in md and "What would close it" not in md
    for part in ("p2", "p5", "p8"):
        text = _section(doc, part)["body"][0]["markdown"]
        assert "Not yet in this report" in text
    for holder in ("p9", "a14"):
        rows = [r for b in _section(doc, holder)["body"] if b["type"] == "table" for r in b["rows"]]
        for sid in ("a3", "a9", "a13"):
            assert not any(r[0] == f"Chapter {CHAPTER_TITLES[sid]}" for r in rows)
    assert _section(doc, "p8")["body"][-1]["items"][0]["value"] == "not yet in this report"


def test_an_appendix_block_says_what_it_would_show_why_and_what_would_close_it():
    """Label: property"""
    doc = cr.build(RUN, "A basis")
    md = _section(doc, "a11")["body"][0]["markdown"]    # the gas example holds no damage cases
    assert md.startswith("**Not evaluated.**")
    for lead in ("What this part would show:", "Why it is not evaluated:", "What would close it:"):
        assert lead in md
    gap = next(r["gap"] for r in A["results"]["rule_trace"]["rows"] if r["rule"] == "igc/2.7/damage-survival")
    assert gap[0].upper() + gap[1:] in md and "igc/2.7/damage-survival" in md and "#838" in md
    assert "`damage`" not in md and "marpol_damage" not in md


def test_not_applicable_is_never_printed_by_the_skeleton():
    """Label: property
    A separate state, printed only when a rule or module says so. One does: the engine's `damage_status`
    (issue #923), printed as the damage line of the summary and nowhere else.
    """
    text = _text(cr.build(RUN, "A basis")).lower()
    line = cr.damage_line(A).lower()
    assert text.count(line) == 1
    assert "not applicable" not in text.replace(line, "")


def test_a_missing_table_value_prints_not_evaluated_never_blank_or_zero():
    """Label: property"""
    t = cr.table("t", "cap", ["a", "b", "c"], [["x", None, ""]])
    assert t["rows"] == [["x", "not evaluated", "not evaluated"]]
    assert cr.n(None) == "not evaluated" and cr._fmt(None, "m") == "not evaluated"
    doc = cr.build(_run_without("cargo"), "A basis")
    rows = _section(doc, "a1")["body"][0]["rows"]
    assert ["Cargo", "not evaluated"] in rows
    assert all(c not in ("", None, "0") for r in rows for c in r)


def test_every_part_has_one_data_box_and_the_summary_gives_the_trace_verdict():
    """Label: property"""
    doc = cr.build(RUN, "A basis")
    for i in range(1, 10):
        boxes = [b for b in _section(doc, f"p{i}")["body"] if b["type"] == "data_box"]
        assert len(boxes) == 1 and boxes[0]["items"]
    counts = cr.rule_counts(A)
    summary = _section(doc, "summary")["body"][0]["markdown"]
    assert f"{counts['pass']} pass" in summary and f"{counts['not evaluated']} not evaluated" in summary


def test_the_control_block_has_a_revision_letter_and_date_and_no_change_table():
    """Label: property"""
    doc = cr.build(RUN, "A basis", number="TN-0001", revision="B", date="2026-10-30")
    assert doc["control"] == {"revision": "B", "date": "2026-10-30", "status": "draft", "number": "TN-0001"}
    assert "history" not in doc["control"] and "history" not in doc
    with pytest.raises(ValueError):
        cr.build(RUN, "A basis", revision="1")


@pytest.mark.parametrize("status", ["approved", "issued", "final", "superseded", "bogus"])
def test_the_status_cannot_be_set_above_the_cap(status):
    """Label: property
    Spec 0015 P7.
    """
    with pytest.raises(ValueError):
        cr.build(RUN, "A basis", status=status)


@pytest.mark.parametrize("status,kw", [("for-checking", {}), ("for-approval", {}), ("for-approval", {"prepared": "A"})])
def test_a_status_that_needs_a_sign_off_name_fails_clearly_without_it(status, kw):
    """Label: property"""
    with pytest.raises(ValueError, match="ORD004"):
        cr.build(RUN, "A basis", status=status, **kw)


@needs_ord
def test_ord_check_passes_at_for_approval_with_both_names_and_fails_without(tmp_path):
    """Label: property"""
    doc = cr.build(RUN, "A basis", status="for-approval", prepared="Preparer", checked="Checker", number="TN-0001")
    assert doc["control"]["prepared"] == "Preparer" and doc["control"]["checked"] == "Checker"
    src = tmp_path / "ok.json"
    src.write_text(json.dumps(doc), encoding="utf-8")
    assert subprocess.run([ORD, "check", str(src)], capture_output=True).returncode == 0
    del doc["control"]["checked"]
    src.write_text(json.dumps(doc), encoding="utf-8")
    bad = subprocess.run([ORD, "check", str(src)], capture_output=True, text=True)
    assert bad.returncode != 0 and "ORD004" in bad.stderr


@pytest.mark.parametrize("status,want", [("draft", "draft"), ("for-checking", "for-checking"), ("For Review", "for-checking"),
                                         ("for_checking", "for-checking"), ("for-approval", "for-approval"),
                                         ("For Approval", "for-approval"), ("for-sign-off", "for-approval")])
def test_a_status_at_or_below_the_cap_is_kept(status, want):
    """Label: property"""
    assert cr.build(RUN, "A basis", status=status, prepared="P", checked="C")["control"]["status"] == want
    assert cr.STATUS_CAP == "for-approval"


def test_the_page_budget_warns_above_twelve_body_pages_and_not_at_twelve():
    """Label: property
    Spec 0015 section 11, test 9, on the page counts.
    """
    assert cr.page_budget_warnings(12) == []
    assert cr.page_budget_warnings(13) != []
    assert "13" in cr.page_budget_warnings(13)[0]
    assert cr.page_budget_warnings(3) == []


@needs_ord
def test_the_page_budget_fires_on_a_rendered_body_of_thirteen_pages_and_not_twelve(tmp_path):
    """Label: property
    Real ORD renders of small documents of 12 and 13 body pages. ORD breaks the page before each
    level-1 section, and every body section here is level 1, so one section is one page.
    """
    def doc_of(pages):
        sections = [{"type": "section", "id": f"s{k}", "title": f"Part {k}", "body": [{"type": "prose", "markdown": "Text."}]}
                    for k in range(pages)]
        return {"ord": "0.2", "class": "technical-note", "title": "T", "language": "en-GB", "notice": cr.NOTICE,
                "control": {"revision": "A", "date": "2026-11-02", "status": "draft"}, "body": sections, "appendices": []}
    base = cr.body_pages(doc_of(1), ORD, tmp_path)   # the cover and the control page come with any body
    assert cr.body_pages(doc_of(12 - base + 1), ORD, tmp_path) == 12
    assert cr.page_budget_warnings(12) == []
    assert cr.body_pages(doc_of(13 - base + 1), ORD, tmp_path) == 13
    assert cr.page_budget_warnings(13) != []


@needs_ord
def test_the_example_body_fits_the_page_budget_with_no_warning(tmp_path):
    """Label: property
    The summary and the nine parts flow under one level-1 section, so the body is far under the budget.
    """
    doc = cr.build(RUN, "A basis", number="TN-0001", date="2026-11-02")
    pages = cr.body_pages(doc, ORD, tmp_path)
    assert pages <= cr.BODY_PAGE_LIMIT, pages
    assert cr.page_budget_warnings(pages) == []


@needs_ord
def test_the_notice_is_on_the_cover_and_in_every_footer_of_the_pdf(tmp_path):
    """Label: property"""
    doc = cr.build(RUN, "A basis", number="TN-0001", date="2026-10-30")
    written = cr.render(ORD, doc, tmp_path / "r")
    pdf = next(p for p in written if p.suffix == ".pdf")
    pages = cr.pdf_pages(pdf)
    assert pages > 2
    for pg in range(1, pages + 1):
        text = subprocess.run(["pdftotext", "-f", str(pg), "-l", str(pg), str(pdf), "-"], capture_output=True, text=True, check=True).stdout
        assert " ".join(cr.NOTICE.split()) in " ".join(text.split()), f"page {pg}"


@needs_ord
def test_the_skeleton_passes_ord_check_and_the_three_formats_render(tmp_path):
    """Label: property"""
    doc = cr.build(RUN, "C deep", ref="A basis", number="TN-0001", date="2026-10-30")
    written = cr.render(ORD, doc, tmp_path / "r")
    assert {p.suffix for p in written} == {".pdf", ".html", ".docx"}


def test_the_cli_writes_a_document_that_ord_accepts(tmp_path):
    """Label: property"""
    run = tmp_path / "run.json"
    run.write_text(json.dumps(RUN), encoding="utf-8")
    out = tmp_path / "doc.json"
    subprocess.run([sys.executable, str(ROOT / "tools" / "concept_report.py"), str(run), "--option", "C deep", "--ref", "A basis",
                    "-o", str(out)], check=True)
    doc = json.loads(out.read_text(encoding="utf-8"))
    assert _section(doc, "a2")["body"][0]["columns"][2]["title"] == "Reference: A basis"
    ord_bin = shutil.which("ord")
    if ord_bin is None:
        pytest.skip("ord is not on the path")
    subprocess.run([ord_bin, "check", str(out)], check=True)


# --- issue #889: the principal characteristics, one data file for the report summary and the app strip ---

PRINCIPAL = json.loads((ROOT / "catalogue" / "report" / "principal_characteristics.json").read_text(encoding="utf-8"))["rows"]


def _summary_rows(run, option):
    """The rows of the principal characteristics table that `cr.build` puts in the summary."""
    return next(b for b in _section(cr.build(run, option), "summary")["body"] if b["type"] == "table")["rows"]


def test_the_data_file_has_the_rows_of_the_ticket_in_order():
    """Label: property"""
    assert [r["id"] for r in PRINCIPAL] == ["loa", "beam", "depth", "draught-laden", "draught-light", "displacement", "lightship",
                                            "deadweight", "cargo", "gross-tonnage", "installed-power", "service-speed",
                                            "crew-minimum", "crew-berths"]


def test_the_report_summary_table_is_the_data_file_applied_to_the_run():
    """Label: property"""
    for o in (A, C):
        rows = _summary_rows(RUN, o["name"])
        assert [r[0] for r in rows] == [r["label"] for r in PRINCIPAL]
        d = o["results"]["dims"]
        assert rows[0][1] == f"{d['loa_m']:,.2f} m" and rows[1][1] == f"{d['beam_m']:,.2f} m"
        assert rows[8][1] == f"{o['results']['results']['cargo_t']:,.0f} t"
    # The stored gas run has a crew block: its values show.
    rows = dict(_summary_rows(RUN, "A basis"))
    crew = A["results"]["crew"]
    assert rows["Crew, minimum"] == f"{crew['minimum']:,.0f}" and rows["Berths"] == f"{crew['berths']:,.0f}"
    # A copy of the run with the crew block removed says "not evaluated" for both, the others still hold numbers.
    bare = json.loads(json.dumps(RUN))
    del next(o for o in bare["options"] if o["name"] == "A basis")["results"]["crew"]
    rows = dict(_summary_rows(bare, "A basis"))
    assert rows["Crew, minimum"] == "not evaluated" and rows["Berths"] == "not evaluated"
    assert rows["Gross tonnage"].endswith(" GT") and rows["Length overall (LOA)"] == "140.00 m"


def test_a_number_that_is_not_finite_is_not_evaluated_in_the_report():
    """Label: property"""
    run = json.loads(json.dumps(RUN))
    res = next(o for o in run["options"] if o["name"] == "A basis")["results"]
    res["dims"]["loa_m"] = float("nan")
    res["dims"]["beam_m"] = float("inf")
    rows = dict(_summary_rows(run, "A basis"))
    assert rows["Length overall (LOA)"] == "not evaluated" and rows["Beam"] == "not evaluated"


def test_a_run_with_crew_and_one_without_both_follow_the_file():
    """Label: property"""
    run = json.loads(json.dumps(RUN))
    res = next(o for o in run["options"] if o["name"] == "A basis")["results"]
    res["crew"] = {"minimum": 7, "berths": 11}
    rows = dict(_summary_rows(run, "A basis"))
    assert rows["Crew, minimum"] == "7" and rows["Berths"] == "11"


def test_the_app_strip_shows_what_the_report_summary_shows():
    """Label: property"""
    node = shutil.which("node")
    if node is None:
        pytest.skip("node is not installed")
    nan_run = json.loads(json.dumps(RUN))
    nan_run["options"][0]["results"]["dims"]["loa_m"] = float("nan")
    script = (
        "import {principalItems} from './app/src/lib/principal.ts';"
        "import fs from 'node:fs';"
        "const run = JSON.parse(fs.readFileSync(0,'utf8').replace(/\\bNaN\\b/g,'\"__nan__\"'),(k,v)=>v==='__nan__'?NaN:v);"
        "console.log(JSON.stringify(run.options.map(o=>principalItems(o.results,'en-GB','not evaluated').map(i=>i.text))));"
    )
    for run in (RUN, nan_run):
        out = subprocess.run([node, "--input-type=module", "-e", script], cwd=ROOT, input=json.dumps(run), capture_output=True,
                             text=True, check=True).stdout
        for texts, o in zip(json.loads(out), run["options"]):
            assert [t.replace("\u00a0", " ") for t in texts] == [r[1] for r in _summary_rows(run, o["name"])]


def _with_damage(status, cases=None, rows_not_run=False):
    """The run A with `damage_status` set, and, for a skipped run, the damage Rule row read `not run` (issue #923)."""
    run = json.loads(json.dumps(RUN))
    o = next(x for x in run["options"] if x["name"] == "A basis")
    o["results"]["damage_status"] = status
    if cases is not None:
        o["results"]["damage_cases"] = cases
    if rows_not_run:
        for row in o["results"]["rule_trace"]["rows"]:
            if row["rule"] == "igc/2.7/damage-survival":
                row.update(outcome="not run", gap="damage stability was not run: the run skipped it")
    return run, o


def _summary_text(doc):
    return "\n".join(b["markdown"] for b in _section(doc, "summary")["body"] if b["type"] == "prose")


def test_the_summary_and_a11_say_whether_damage_ran():
    """Label: property
    Issue #923: the summary states `damage_status` for every value, and a skipped run says so in A.11 with the
    cases it skipped. Nothing in the report says the damage Rules were met when the run did not run them.
    """
    said = {}
    for status, cases in (("run", 40), ("not run", 40), ("not applicable", None), ("not evaluated", None)):
        run, o = _with_damage(status, cases, rows_not_run=status == "not run")
        doc = cr.build(run, "A basis", assets=GA)
        said[status] = _summary_text(doc)
        a11 = _section(doc, "a11")["body"][0]["markdown"]
        if status == "not run":
            assert "40 cases skipped" in said[status] and "not run" in said[status]
            assert "Damage stability was not run for this result (40 cases skipped)" in a11
            assert "1 not run" in said[status]
            items = [r for t in _section(doc, "a14")["body"] if t["type"] == "table" for r in t["rows"]]
            assert any("igc/2.7/damage-survival" in r[0] and r[2] == "Run the design again with damage on" for r in items)
        else:
            assert "was not run" not in said[status]
    assert "Damage stability was run (40 cases)." in said["run"]
    assert "not applicable" in said["not applicable"]
    # A damage Rule the engine does not evaluate is not "no damage Rule applies" (issue #923).
    assert "not evaluated" in said["not evaluated"] and "applies" not in said["not evaluated"]
    # A run that does not say whether damage ran is not read as run.
    assert "Damage stability: not evaluated" in _summary_text(cr.build(_run_without("damage_status"), "A basis", assets=GA))
