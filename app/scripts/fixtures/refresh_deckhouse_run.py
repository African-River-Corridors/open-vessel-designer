#!/usr/bin/env python3
"""Rewrite deckhouse-run.json: the keys of `ovd run --json` that the Stern & crew view reads, for the deckhouse design
of app/e2e/fixtures. Run from the repo root: python3 app/scripts/fixtures/refresh_deckhouse_run.py <path to ovd>.
A Rust test (crates/ovd-format/tests/arrangement_style.rs) fails when this file is stale."""
import json, subprocess, sys

KEEP = ["stern_arrangement", "crew", "rooms", "warnings", "text_ids", "stern_block_mass", "results"]
out = subprocess.run([sys.argv[1], "run", "--json", "app/e2e/fixtures/gas_deckhouse_self_propelled.vdesign"], capture_output=True, text=True, check=True).stdout
r = json.loads(out)["options"][0]["results"]
with open("app/scripts/fixtures/deckhouse-run.json", "w") as f:
    json.dump({k: r[k] for k in KEEP}, f, indent=1, sort_keys=True)
    f.write("\n")
