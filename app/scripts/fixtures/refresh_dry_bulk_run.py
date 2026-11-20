"""Refresh scripts/fixtures/dry-bulk-run.json for summary.test.mjs (issue #1081).

Run from the repo root with the CLI built: python3 app/scripts/fixtures/refresh_dry_bulk_run.py target/debug/ovd
It keeps only the parts of the run that src/lib/summary.ts reads.
"""
import json
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
run = json.loads(subprocess.run([sys.argv[1], "run", "--json", str(root / "crates/ovd-format/tests/fixtures/dry_bulk_box_hold.vdesign")], capture_output=True, text=True, check=True).stdout)
r = run["options"][0]["results"]
keep = {k: r.get(k) for k in ("results", "checks", "arrangement", "powering", "damage_status")}
trace = r["rule_trace"]
keep["rule_trace"] = {"scope": trace["scope"], "rows": [{k: row[k] for k in ("title", "consequence", "outcome")} for row in trace["rows"]]}
out = {"options": [{"name": run["options"][0]["name"], "results": keep}]}
(root / "app/scripts/fixtures/dry-bulk-run.json").write_text(json.dumps(out, indent=1) + "\n")
