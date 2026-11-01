"""The ATB calibration (issue #788, spec 0014 D10, D19): the case file is well formed and the held-out
cases stay out of the fit. The pin of the allowance to `calibration/atb.yaml` (the embedded file and
the Defaults Profile) is `crates/ovd-format/tests/atb_powering.rs`,
`the_allowance_is_pinned_to_calibration_atb_yaml`; this file reads only `calibration/`, which the
wheel check copies. The fit itself needs the `ovd` binary:
`python3 tools/atb_calibration.py <ovd> --check` (see the tool)."""

from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
CAL = ROOT / "calibration"


def _file():
    return yaml.safe_load((CAL / "atb.yaml").read_text(encoding="utf-8"))


def _fit():
    return _file()["fit"]["formation_resistance_allowance"]["atb"]


def test_the_fit_is_an_estimate_that_states_its_basis():
    """Label: property"""
    fit = _fit()
    assert fit["status"] == "estimate"
    assert fit["basis"].startswith(f"Fitted to {fit['n']} vessel")
    assert "assumed" in fit["basis"]


def test_held_out_cases_stay_out_of_the_fit():
    """Label: property"""
    cases = _file()["atbs"]
    fit = _fit()
    held = {c["id"] for c in cases if c.get("held_out")}
    fitted = {e["case"] for e in fit["evidence"]}
    assert held and fitted
    assert not held & fitted
    assert fitted == {c["id"] for c in cases if not c.get("held_out")}
    assert fit["n"] == len(fitted)


def test_every_case_names_its_source_and_its_published_and_assumed_figures():
    """Label: property"""
    for c in _file()["atbs"]:
        assert c["source"] and c["installed_power_hp"] > 0 and c["speed_kn"] > 0
        assert c["basis"].startswith("published:")
        assert c["confidence"] in {"high", "medium", "low"}
        assert c["tug"]["propulsor_type"]
    # An LNG ATB is in the fit (spec 0014 D10).
    assert any("LNG" in c["name"] and not c.get("held_out") for c in _file()["atbs"])


def test_a_listed_design_that_is_not_run_says_why():
    """Label: property"""
    for c in _file()["listed_not_run"]:
        assert c["published"] and c["not_run"]
