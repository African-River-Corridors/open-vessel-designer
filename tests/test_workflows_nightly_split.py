"""The nightly every-test run is split across parallel CI jobs (issue #1046). One job running the whole
fast gate with every test reached its 30 minute limit and was cancelled, and a cancelled run hides a
real failure. The jobs together must still run every step group and every test, and `every-test` must
stand for all of them.

Label: property
"""
import json
import re
import subprocess
import sys
import textwrap
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
NIGHTLY = yaml.safe_load((ROOT / ".github" / "workflows" / "nightly.yml").read_text(encoding="utf-8"))
GATES = (ROOT / "tools" / "gates.sh").read_text(encoding="utf-8")
JOBS = NIGHTLY["jobs"]


def _parts():
    return {n: j for n, j in JOBS.items() if n.startswith("nightly-")}


def _env_of(job):
    out = {}
    for step in job["steps"]:
        out.update(step.get("env", {}))
    return out


def test_no_job_runs_the_whole_gate_in_one_job():
    for name, job in _parts().items():
        assert _env_of(job).get("GATES_STEPS"), f"{name} runs every step group in one job"


def test_the_parts_together_run_every_step_group_of_the_script():
    groups = set(re.search(r'GROUPS_ALL="([^"]*)"', GATES).group(1).split())
    run = set()
    for job in _parts().values():
        run |= set(filter(None, _env_of(job).get("GATES_STEPS", "").split(",")))
    assert run == groups, (run, groups)


def test_the_rust_tests_run_every_test_in_partitions_of_the_nightly_profile():
    job = JOBS["nightly-rust"]
    parts = job["strategy"]["matrix"]["part"]
    assert isinstance(parts, list) and parts == list(range(1, len(parts) + 1)) and len(parts) > 1
    env = _env_of(job)
    assert env["GATES_PARTITION"] == "${{ matrix.part }}/${{ strategy.job-total }}"
    assert env["GATES_NEXTEST_PROFILE"] == "nightly"      # the slow tests too
    assert job["strategy"]["fail-fast"] is False


def test_pytest_runs_every_test_in_shards_of_the_nightly_profile():
    job = JOBS["nightly-python"]
    shards = job["strategy"]["matrix"]["shard"]
    assert shards == list(range(1, len(shards) + 1)) and len(shards) > 1
    env = _env_of(job)
    assert env["PYTEST_SHARD"] == "${{ matrix.shard }}/%d" % len(shards)
    assert env["GATES_NEXTEST_PROFILE"] == "nightly"      # no `slow` mark left out
    steps = "\n".join(s.get("run", "") for s in job["steps"])
    assert "python evaluation/run.py --check" in steps
    assert "python evaluation/oracle/mutants.py --check" in steps


def test_every_part_has_a_timeout_inside_thirty_minutes():
    for name, job in _parts().items():
        assert job["timeout-minutes"] <= 30, name


def test_every_test_waits_for_every_part_and_does_not_skip_on_failure():
    job = JOBS["every-test"]
    assert set(job["needs"]) == set(_parts())
    assert job["if"] == "always()"
    text = "\n".join(s.get("run", "") for s in job["steps"])
    for name in _parts():
        assert f'"{name}"' in text, name


def _roll_up_script():
    """The Python the `every-test` step runs, taken out of its shell heredoc."""
    run = "\n".join(s.get("run", "") for s in JOBS["every-test"]["steps"])
    return re.search(r"<<'PY'\n(.*?)\n\s*PY\s*$", run, re.S).group(1)


def _roll_up_exit(results):
    needs = {n: {"result": r} for n, r in results.items()}
    script = textwrap.dedent(_roll_up_script())
    done = subprocess.run([sys.executable, "-I", "-c", script], env={"RESULTS": json.dumps(needs)},
                          capture_output=True, text=True)
    return done.returncode


def _all(result="success"):
    return {n: result for n in _parts()}


def test_the_roll_up_passes_only_when_every_part_succeeded():
    assert _roll_up_exit(_all()) == 0
    for bad in ("failure", "cancelled", "skipped"):
        for name in _parts():
            assert _roll_up_exit({**_all(), name: bad}) == 1, (name, bad)


def test_the_roll_up_fails_on_a_missing_or_an_extra_part():
    for name in _parts():
        missing = {n: r for n, r in _all().items() if n != name}
        assert _roll_up_exit(missing) == 1, name
    assert _roll_up_exit({**_all(), "nightly-extra": "success"}) == 1
