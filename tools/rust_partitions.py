"""Write `.config/rust-partitions.txt` from measured test times (issue #819).

    python3 tools/rust_partitions.py <N> <junit.xml>... > .config/rust-partitions.txt

nextest splits the Rust tests over the `gate-rust` jobs by a hash of their names, so a job can draw
several slow tests and wait on them while the others are done. This script places the slow tests by
their time: each test of at least `--over` seconds (default 15) goes to the job with the least time so
far, longest first, counting each job's share of the other tests as already placed. `tools/gates.sh`
runs a job's placed tests, then its hash share of every test that is not placed, so every test still
runs in exactly one job.

The inputs are the JUnit files of a CI run (the `nextest-junit-*` artifacts of `ci.yml`). Run it again
when the slow tests change; a stale line costs only balance, never a test.
"""
import argparse
import sys
import xml.etree.ElementTree as ET


def times(paths):
    out = {}
    for path in paths:
        for case in ET.parse(path).getroot().iter("testcase"):
            out[(case.get("classname"), case.get("name"))] = float(case.get("time"))
    return out


def place(measured, parts, over):
    slow = sorted(((t, k) for k, t in measured.items() if t >= over), reverse=True)
    rest = sum(t for t in measured.values() if t < over)
    load = [rest / parts] * parts
    placed = []
    for t, key in slow:
        k = min(range(parts), key=lambda i: (load[i], i))
        load[k] += t
        placed.append((k + 1, key, t))
    return sorted(placed), load


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("parts", type=int)
    ap.add_argument("junit", nargs="+")
    ap.add_argument("--over", type=float, default=15.0, help="place tests of at least this many seconds")
    a = ap.parse_args(argv)
    placed, load = place(times(a.junit), a.parts, a.over)
    print("# The slow Rust tests, each placed in one gate-rust job by its measured time (issue #819).")
    print("# Written by tools/rust_partitions.py; the job, the binary and the test, then the seconds.")
    print(f"# Expected test seconds per job: {', '.join(f'{x:.0f}' for x in load)}.")
    for k, (binary, test), t in placed:
        print(f"{k} {binary} {test} {t:.0f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
