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
NEED_CASES = [(c[0], need) for c in cr.CHAPTERS for need in c[3]]


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
    assert [s["id"] for s in doc["body"]] == ["summary"] + [f"p{i}" for i in range(1, 10)]
    assert doc["body"][0]["numbered"] is False
    assert [s["title"] for s in doc["body"][1:]] == [
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


@pytest.mark.parametrize("cid,need", NEED_CASES, ids=[f"{c}-{n if isinstance(n, str) else 'any'}" for c, n in NEED_CASES])
def test_deleting_a_run_block_keeps_every_heading_and_reads_not_evaluated_in_three_places(cid, need):
    """Label: property
    Spec 0015 section 11, test 4: the body line, the part 9 row and the appendix block.
    """
    base = cr.build(RUN, "A basis")
    doc = cr.build(_run_without(need), "A basis")
    assert _ids(doc) == _ids(base)
    chapter = next(c for c in cr.CHAPTERS if c[0] == cid)
    part = _section(doc, chapter[2])
    appendix = _section(doc, cid)
    assert "**Not evaluated.**" in appendix["body"][0]["markdown"]
    assert f"[[#{cid}]]" in part["body"][0]["markdown"] and "Not evaluated" in part["body"][0]["markdown"]
    assert {i["label"]: i["value"] for i in part["body"][-1]["items"]}[chapter[1]] == "not evaluated"
    for table_holder in (_section(doc, "p9"), _section(doc, "a14")):
        tables = [b for b in table_holder["body"] if b["type"] == "table"]
        assert any(r[0] == f"Chapter {chapter[1]}" for t in tables for r in t["rows"])


def test_an_appendix_block_says_what_it_would_show_why_and_what_would_close_it():
    """Label: property"""
    doc = cr.build(RUN, "A basis")
    md = _section(doc, "a11")["body"][0]["markdown"]    # the gas example holds no damage block
    assert md.startswith("**Not evaluated.**")
    for lead in ("What this part would show:", "Why it is not evaluated:", "What would close it:"):
        assert lead in md
    assert "`damage` or `marpol_damage`" in md


def test_not_applicable_is_never_printed_by_the_skeleton():
    """Label: property
    A separate state, printed only when a rule or module says so (none does yet).
    """
    assert "not applicable" not in _text(cr.build(RUN, "A basis")).lower()


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


@pytest.mark.parametrize("status", ["for-approval", "For Approval", "for_approval", "approved", "issued", "final", "superseded", "bogus"])
def test_the_status_cannot_be_set_above_the_cap(status):
    """Label: property
    Spec 0015 P7.
    """
    with pytest.raises(ValueError):
        cr.build(RUN, "A basis", status=status)


@pytest.mark.parametrize("status,want", [("draft", "draft"), ("for-checking", "for-checking"), ("For Review", "for-checking"), ("for_checking", "for-checking")])
def test_a_status_at_or_below_the_cap_is_kept(status, want):
    """Label: property"""
    assert cr.build(RUN, "A basis", status=status)["control"]["status"] == want
    assert cr.STATUS_CAP == "for-checking"


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
    Real ORD renders of small documents of 12 and 13 body pages. ORD starts every numbered
    section on a new page, so one section is one page.
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
