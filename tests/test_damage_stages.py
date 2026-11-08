"""Damage as a second stage in the Python API, with progress and cancel (issue #940).

Label: property. Each test compares two paths to one answer; no number is pinned. The first
compares the whole results value, damage blocks included: the results hash leaves them out, so it
cannot prove that the stage and `simulate(p, damage=True)` agree.
"""
import _thread
import json
import threading
from pathlib import Path

import pytest

from ovd import _bridge
from ovd.simulate import simulate, simulate_damage

ROOT = Path(__file__).resolve().parents[1]
GAS = ROOT / "examples" / "gas-140x29-coastal-lng" / "gas-140x29-coastal-lng.vdesign"
# A dry bulk box barge: no damage Rule Set applies to it.
DRY_BULK = {"cargo": "bauxite", "hull_form": "raked_box", "formation": {"kind": "pushed_barge"},
            "length_overall_m": 76.5, "beam_m": 11.4, "moulded_depth_m": 3.5,
            "service_area": "river-only"}


def typed(path: Path) -> dict:
    """The typed inputs of the first Option of a file, as the dict `simulate` takes."""
    inputs = json.loads(path.read_text(encoding="utf-8"))["options"][0]["inputs"]
    p = {k: v["value"] for k, v in inputs.items() if v["source"] == "typed"}
    p.setdefault("service_area", "river-only")
    return p


def whole(result: dict) -> str:
    return json.dumps(result, sort_keys=True)


@pytest.fixture(scope="module")
def coastal() -> dict:
    return typed(GAS)


@pytest.fixture(scope="module")
def plain(coastal) -> dict:
    return simulate(coastal)


@pytest.fixture(scope="module")
def with_damage(coastal) -> dict:
    return simulate(coastal, damage=True)


@pytest.mark.slow
def test_the_two_stages_equal_one_call_with_damage_on_the_whole_value(coastal, plain, with_damage):
    """Label: property"""
    staged = simulate_damage(coastal)
    assert staged["damage_status"] == "run"
    assert plain["damage_status"] == "not run"
    assert whole(staged) == whole(with_damage)
    # The progress callable changes nothing in the result.
    assert whole(simulate_damage(coastal, progress=lambda done, total: None)) == whole(with_damage)


def test_progress_counts_up_to_the_total(coastal, with_damage):
    """Label: property"""
    calls = []
    simulate_damage(coastal, progress=lambda done, total: calls.append((done, total)))
    total = with_damage["damage_cases"]
    assert total > 3
    # Issue #935: the search runs damage on each candidate that passes stability, then the answer's run follows.
    # The count is cumulative over those runs: done never falls, total is the cases of the runs so far and never
    # falls, and the last call has every case done. The answer's own cases are the last run's share of it.
    assert calls[0][0] == 0
    assert all(a <= b for (a, _), (b, _) in zip(calls, calls[1:])), "done fell"
    # The total is planned (the runs the search expects) and shrinks at the end: the fraction done never falls.
    assert all(a / ta <= b / tb + 1e-12 for (a, ta), (b, tb) in zip(calls, calls[1:])), "the fraction fell"
    assert calls[-1][0] == calls[-1][1]
    assert calls[-1][1] >= total


@pytest.mark.slow
def test_a_cancel_returns_the_results_of_a_run_without_damage(coastal, plain):
    """Label: property"""
    asked = []
    seen = []

    def cancel():
        asked.append(1)
        return len(asked) > 3

    result = simulate_damage(coastal, progress=lambda d, t: seen.append((d, t)), cancel=cancel)
    assert result["damage_status"] == "not run"
    assert whole(result) == whole(plain)
    assert len(asked) == 4, "cancel() was asked again after it said yes"
    assert seen[-1][0] < seen[-1][1], "the run was not stopped part-way"


@pytest.mark.slow
def test_an_exception_in_a_callable_stops_the_run_and_is_raised(coastal):
    """Label: property"""
    def boom(done, total):
        if done >= 1:
            raise ValueError("progress broke")

    with pytest.raises(ValueError, match="progress broke"):
        simulate_damage(coastal, progress=boom)

    def boom_cancel():
        raise KeyError("cancel broke")

    with pytest.raises(KeyError, match="cancel broke"):
        simulate_damage(coastal, cancel=boom_cancel)


def test_a_hull_with_no_damage_rule_set_is_not_applicable_and_never_calls_progress():
    """Label: property"""
    p = dict(DRY_BULK)
    calls = []
    result = simulate_damage(p, progress=lambda d, t: calls.append((d, t)))
    assert result["damage_status"] == "not applicable"
    assert whole(result) == whole(simulate(p, damage=True))
    assert calls == []


def test_a_callback_that_is_not_callable_is_refused_before_any_work():
    """Label: property"""
    with pytest.raises(TypeError, match="progress"):
        simulate_damage({}, progress=3)
    with pytest.raises(TypeError, match="cancel"):
        simulate_damage({}, cancel="stop")


def test_a_refused_input_raises_as_simulate_raises():
    """Label: property"""
    with pytest.raises(TypeError) as staged:
        simulate_damage({"no_such_input": 1})
    with pytest.raises(TypeError) as plain:
        simulate({"no_such_input": 1})
    assert str(staged.value) == str(plain.value)


def test_another_python_thread_runs_while_the_core_calculates(coastal, monkeypatch):
    """Label: property

    The proxy raises a flag on the line before the call into the core and drops it
    on the line after. A thread that counts only while the flag is up can count only if the call let go
    of the GIL; it does not depend on how long the call takes."""
    inside = threading.Event()
    stop = threading.Event()
    started = threading.Event()
    counted = [0]

    def count():
        started.set()
        while not stop.is_set():
            if inside.is_set():
                counted[0] += 1

    class Proxy:
        def __getattr__(self, name):
            return getattr(_real, name)

        def eval(self, entry_point, case_json):
            inside.set()
            try:
                return _real.eval(entry_point, case_json)
            finally:
                inside.clear()

    _real = _bridge._native
    monkeypatch.setattr(_bridge, "_native", Proxy())
    thread = threading.Thread(target=count)
    thread.start()
    try:
        started.wait()
        simulate(coastal)
        during = counted[0]
    finally:
        stop.set()
        thread.join()
    assert during > 100_000, during


def test_ctrl_c_in_a_callable_stops_the_run_with_keyboard_interrupt(coastal):
    """Label: property

    `interrupt_main` is what Ctrl-C does to the main thread. The run raises
    `KeyboardInterrupt` only if the Python calls are made on the thread that called `simulate_damage`,
    because only there is a pending signal seen. No timing: the interrupt is raised by the callable, on
    its second call."""
    calls = []

    def progress(done, total):
        calls.append(done)
        if len(calls) == 2:
            _thread.interrupt_main()

    with pytest.raises(KeyboardInterrupt):
        simulate_damage(coastal, progress=progress)
    assert len(calls) == 2, "the run went on after the signal"
