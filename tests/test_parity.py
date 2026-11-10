"""Parity guard (issue #939): the Python `simulate` dict builds the Option the file does.

`simulate` and `ovd run` share one run (issue #937). The one step they do not share is the
dict-to-Option step: names, shapes, the Formation and the Cargo Containment derivation. These tests
check that step by the input values it builds, with no run, through the `option_inputs` seam of the
`api` entry (the same seam `crates/ovd-format/tests/api_parity.rs` uses). The file side is
`ovd fill --as-new`. A difference fails and names the key.

Label: property. Each test compares two paths to one answer; no number is pinned.
"""
import json
from pathlib import Path

import pytest

from ovd._bridge import call
from ovd.simulate import simulate, simulate_damage

ROOT = Path(__file__).resolve().parents[1]
# The fixture Design of each Cargo Containment (box_hold, sloped_hopper_hold, type_c_tanks,
# integral_tanks): the Designs of the omit and full-run cases.
PER_CONTAINMENT = [
    "dry_bulk_box_hold",
    "dry_bulk_sloped_hopper",
    "gas_igc_2g_self_propelled",
    "crude_integral_tanks_coastal",
]


def _cases():
    """Every Option of every file of `examples/` and of the fixtures: (label, text, index, stored)."""
    files = sorted((ROOT / "examples").rglob("*.vdesign"))
    files += sorted((ROOT / "crates/ovd-format/tests/fixtures").rglob("*.vdesign"))
    assert len(files) >= 15, files
    out = []
    for path in files:
        text = path.read_text(encoding="utf-8")
        for index, option in enumerate(json.loads(text)["options"]):
            stored = {k: (i["value"], i["source"]) for k, i in option["inputs"].items()}
            out.append((f"{path.name} #{index} ({option['name']})", text, index, stored))
    return out


CASES = _cases()
FIRST = {Path(label.split(" #")[0]).stem: c for c in CASES for label in [c[0]] if c[2] == 0}


def _inputs(args):
    return call("api", "option_inputs", args, int_ok=("index",))


def _by_file(text, index=0):
    return _inputs({"design": text, "index": index})


def _file_of(dict_):
    """The `.vdesign` text of a dict of inputs, every one typed."""
    inputs = {k: {"value": v, "source": "typed"} for k, v in dict_.items()}
    return json.dumps({"format": "vdesign", "format_version": 4, "engine_version": "0",
                       "units": "SI", "options": [{"name": "Option 1", "inputs": inputs}]})


def _refused_by_design(dict_):
    """Inputs the dict step refuses on purpose (`midship_coefficient` of a hull from offsets or ShipD)
    that a stored file may still hold. The message says `is not an input`."""
    try:
        _inputs({"params": dict_})
    except TypeError as e:
        return [part.rsplit("`", 2)[-2] for part in str(e).split("is not an input")[:-1]]
    return []


def _without_unused(dict_):
    """The dict step's Option, once an input the omission made unused is dropped. A file only warns
    about an input its Option does not use (omit `propulsor_type` and the stored `thruster_count` no
    longer applies); the dict step raises, by design (issue #937), so that is not drift."""
    dict_ = dict(dict_)
    for _ in range(5):
        try:
            return _inputs({"params": dict_})
        except TypeError as e:
            unused = [part.rsplit("`", 2)[-2] for part in str(e).split(" is not used")[:-1]]
            if not unused or any(k not in dict_ for k in unused):
                raise
            for k in unused:
                del dict_[k]
    raise AssertionError("inputs still unused after five rounds")


def _differences(got, want):
    return [f"{k}: dict {got.get(k)!r}, file {want.get(k)!r}"
            for k in sorted(set(got) | set(want)) if got.get(k) != want.get(k)]


def _typed(stored):
    # The Service Area has no default (issue #925), so a fixture that holds it as a default gives it too.
    return {k: v for k, (v, source) in stored.items() if source == "typed" or k == "service_area"}


def test_the_dict_step_builds_the_files_option_input_by_input():
    """Label: property"""
    failed = []
    for label, text, index, stored in CASES:
        dict_ = {k: v for k, (v, _) in stored.items()}
        refused = _refused_by_design(dict_)
        for k in refused:
            del dict_[k]
        got = _inputs({"params": dict_})
        want = {k: v for k, v in _by_file(text, index).items() if k not in refused}
        failed += [f"{label}: {d}" for d in _differences(got, want)]
    assert not failed, "\n".join(failed)


def test_every_profile_defaulted_input_left_out_at_once_is_built_as_the_file_path_builds_it():
    """Label: property"""
    # Only the typed inputs are given, so the Profile fills the rest on both paths. Four Designs: a
    # build with every default left out runs the arrangement search of a gas Design.
    failed = []
    for name in PER_CONTAINMENT:
        label, _, _, stored = FIRST[name]
        typed = _typed(stored)
        for k in _refused_by_design(typed):
            del typed[k]
        got = _inputs({"params": typed})
        want = _by_file(_file_of(typed))
        failed += [f"{label}: {d}" for d in _differences(got, want)]
    assert not failed, "\n".join(failed)


@pytest.mark.slow
def test_each_profile_defaulted_input_left_out_one_at_a_time_is_built_as_the_file_path_builds_it():
    """Label: property"""
    failed, tried = [], 0
    for name in PER_CONTAINMENT:
        label, _, _, stored = FIRST[name]
        full = {k: v for k, (v, _) in stored.items()}
        for k in _refused_by_design(full):
            del full[k]
        for key, (_, source) in stored.items():
            if source != "default" or key == "service_area" or key not in full:
                continue
            left_out = {k: v for k, v in full.items() if k != key}
            tried += 1
            failed += [f"{label} without `{key}`: {d}" for d in
                       _differences(_without_unused(left_out), _by_file(_file_of(left_out)))]
    assert not failed, "\n".join(failed)
    assert tried > 400, tried


def test_every_input_of_the_schema_is_accepted_by_the_dict_step():
    """Label: property"""
    # An input is settable if the dict step never calls its name unknown. It may refuse the
    # placeholder value, or the input as unused by this Option.
    names = _inputs({"schema": True})
    assert len(names) > 100, names
    unknown = []
    for name in names:
        try:
            _inputs({"params": {name: 1.0}})
        except (TypeError, ValueError) as e:
            if f"unknown input `{name}`" in str(e):
                unknown.append(name)
    assert not unknown, f"the dict step does not know: {unknown}"


@pytest.mark.parametrize("name", PER_CONTAINMENT)
def test_simulate_with_damage_equals_the_damage_stage_on_the_whole_value(name):
    """Label: property"""
    # The `ovd run` side of this equality is in crates/ovd-format/tests/api_parity.rs.
    _, _, _, stored = FIRST[name]
    dict_ = _typed(stored)
    got = simulate(dict_, damage=True)
    assert json.dumps(got, sort_keys=True) == json.dumps(simulate_damage(dict_), sort_keys=True)
