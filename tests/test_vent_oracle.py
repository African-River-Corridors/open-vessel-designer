"""The vent oracle of the evaluation harness (issue #849): its limits worked out by hand from IGC Code
8.2.10.3, 8.2.10.4 and 8.2.11.1, the generated case files are what it writes, and every boundary case moves
one number across one limit.

Label: property
"""
import sys
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "evaluation" / "oracle"))
import mutants  # noqa: E402
import vent_oracle as oracle  # noqa: E402
import write_cases  # noqa: E402

EVAL = ROOT / "evaluation"


def case(**over) -> dict:
    i = dict(write_cases.VENT_HULL)
    i.update(over)
    return i


def test_the_height_limit_by_hand():
    """Label: sourced - IGC 8.2.10.3: not less than B/3 or 6 m, whichever is the greater, above the weather deck.
    B = 14 m: B/3 is 4.67 m, so 6 m. B = 18 m: both are 6 m. B = 24 m: 8 m."""
    key = oracle.DECK
    assert oracle.verdicts(case(beam_m=14.0))[key]["limit"] == 6.0
    assert oracle.verdicts(case(beam_m=18.0))[key]["limit"] == 6.0
    assert oracle.verdicts(case(beam_m=24.0))[key]["limit"] == 8.0
    assert oracle.verdicts(case(beam_m=24.0, vent_exit_above_deck_m=8.0))[key]["verdict"] == "pass"
    assert oracle.verdicts(case(beam_m=24.0, vent_exit_above_deck_m=7.99))[key]["verdict"] == "fail"


def test_the_working_area_limit_by_hand():
    """Label: sourced - IGC 8.2.10.4: 6 m above working areas and walkways."""
    key = oracle.WORKING
    assert oracle.verdicts(case(vent_exit_above_working_area_m=6.0))[key]["verdict"] == "pass"
    assert oracle.verdicts(case(vent_exit_above_working_area_m=5.99))[key]["verdict"] == "fail"


def test_the_distance_limit_by_hand():
    """Label: sourced - IGC 8.2.11.1: at least B or 25 m, whichever is less. B = 14 m: 14 m. B = 30 m: 25 m."""
    key = oracle.DISTANCE
    assert oracle.verdicts(case(beam_m=14.0))[key]["limit"] == 14.0
    assert oracle.verdicts(case(beam_m=30.0))[key]["limit"] == 25.0
    assert oracle.verdicts(case(beam_m=30.0, vent_distance_to_openings_m=25.0))[key]["verdict"] == "pass"
    assert oracle.verdicts(case(beam_m=30.0, vent_distance_to_openings_m=24.99))[key]["verdict"] == "fail"


def test_the_case_files_are_what_the_oracle_writes():
    """Label: property"""
    for c in write_cases.VENT_BASES:
        path = EVAL / "cases" / f"{c['id']}.yaml"
        want = yaml.safe_load(write_cases.render(write_cases.case_document(c)))
        assert write_cases.first_difference(yaml.safe_load(path.read_text(encoding="utf-8")), want) is None, c["id"]
    assert mutants.main(["mutants.py", "--check"]) == 0


def test_every_boundary_case_moves_one_number_across_one_limit():
    """Label: property"""
    docs = yaml.safe_load((EVAL / "vent_mutants.yaml").read_text(encoding="utf-8"))["mutants"]
    assert len(docs) == 56
    parents = {c["id"]: c for c in write_cases.VENT_BASES}
    for d in docs:
        m = d["mutant"]
        parent = parents[m["parent"]]["inputs"]
        changed = [k for k in parent if parent[k] != d["inputs"][k]]
        assert changed == [m["moved"]], (d["id"], changed)
        e = d["expected"][m["criterion"]]
        assert e["verdict"] == m["side"], d["id"]
        if m["magnitude"] == "big":
            assert abs(e["margin"]) == pytest.approx(oracle.STEPS["m"][0], rel=1e-3, abs=1e-9)
        else:
            assert abs(e["margin"]) < 1e-9
    # Every row has a case exactly on the limit (meets) and one float step under it (fails).
    seen = {(d["mutant"]["criterion"], d["mutant"]["magnitude"], d["mutant"]["side"]) for d in docs}
    for rule in (oracle.DECK, oracle.WORKING, oracle.DISTANCE, oracle.ADN_HEIGHT, oracle.ADN_DISTANCE):
        assert (rule, "exact", "pass") in seen and (rule, "ulp", "fail") in seen


def test_the_adn_limits_by_hand():
    """Label: sourced - ADN 2025 9.3.1.22.3 (printed p. 523): the outlets of the pressure relief valves not
    less than 2.00 m above the deck and not less than 6.00 m from the accommodation. River-only service."""
    i = case(service_area="river-only", vent_distance_to_accommodation_m=6.0)
    v = oracle.verdicts(i)
    assert set(v) == {oracle.ADN_HEIGHT, oracle.ADN_DISTANCE}
    assert v[oracle.ADN_HEIGHT]["limit"] == 2.0 and v[oracle.ADN_DISTANCE]["limit"] == 6.0
    assert v[oracle.ADN_DISTANCE]["verdict"] == "pass"
    assert oracle.verdicts({**i, "vent_distance_to_accommodation_m": 5.99})[oracle.ADN_DISTANCE]["verdict"] == "fail"
    assert oracle.verdicts({**i, "vent_exit_above_deck_m": 2.0})[oracle.ADN_HEIGHT]["verdict"] == "pass"
    assert oracle.verdicts({**i, "vent_exit_above_deck_m": 1.99})[oracle.ADN_HEIGHT]["verdict"] == "fail"
