"""The damage oracle's cases file is what the oracle writes (issue #763)."""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def test_the_damage_cases_are_what_the_oracle_writes():
    """Label: property - `damage_oracle.py --check` finds no difference."""
    out = subprocess.run(
        [sys.executable, str(ROOT / "evaluation/oracle/damage_oracle.py"), "--check"],
        capture_output=True, text=True,
    )
    assert out.returncode == 0, out.stdout + out.stderr
