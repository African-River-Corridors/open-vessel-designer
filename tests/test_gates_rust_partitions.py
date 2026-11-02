"""The slow Rust tests placed by measured time (issue #819): each line of .config/rust-partitions.txt
places one test in one `gate-rust` job that exists, so `tools/gates.sh` runs it exactly once.

Label: property
"""
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
PLACED = ROOT / ".config" / "rust-partitions.txt"
CI = yaml.safe_load((ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8"))


def _lines():
    out = []
    for line in PLACED.read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.startswith("#"):
            out.append(line.split())
    return out


def test_every_placed_test_is_in_a_job_that_exists_and_is_placed_once():
    """Label: property"""
    jobs = len(CI["jobs"]["gate-rust"]["strategy"]["matrix"]["part"])
    lines = _lines()
    assert lines
    for k, binary, test, seconds in lines:
        assert 1 <= int(k) <= jobs, (k, test)
        assert "::" in binary and test.isidentifier(), (binary, test)
        float(seconds)
    keys = [(b, t) for _, b, t, _ in lines]
    assert len(keys) == len(set(keys))



def test_no_placed_test_is_one_the_pr_profile_leaves_out():
    """The placed run has no filter of its own, so a placed test the `pr` profile leaves out would
    run in every PR (docs/agents/slow-tests.md lists those tests).

    Label: property"""
    import tomllib

    pr = tomllib.loads((ROOT / ".config" / "nextest.toml").read_text(encoding="utf-8"))["profile"]["pr"]
    for _, binary, test, _ in _lines():
        assert f"test(={test})" not in pr["default-filter"], test
