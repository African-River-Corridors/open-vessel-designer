"""The web app's end-to-end job runs on pull requests (issue #1094), and no workflow runs the paused desktop
app's checks (ADR 0026). Reads app-gate.yml, release.yml and the workflow folder."""
import re
import subprocess
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"
APP_GATE = WORKFLOWS / "app-gate.yml"


def workflow(path: Path) -> dict:
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    data["on"] = data.pop(True, data.get("on"))  # YAML reads a bare `on` as True
    return data


def paths_pattern() -> re.Pattern:
    """The regex the `changes` job greps the changed paths with."""
    run = next(s["run"] for s in workflow(APP_GATE)["jobs"]["changes"]["steps"] if s.get("id") == "paths")
    return re.compile(re.search(r"grep -qE '([^']+)'", run).group(1))


def test_the_web_job_runs_on_pull_requests_merge_groups_and_releases():
    """Label: property"""
    on = workflow(APP_GATE)["on"]
    assert {"pull_request", "merge_group", "workflow_call", "workflow_dispatch"} <= set(on)
    release = workflow(WORKFLOWS / "release.yml")["jobs"]
    assert any(str(j.get("uses", "")).endswith("app-gate.yml") for j in release.values()), "a release waits on the web job"


def test_the_web_job_runs_when_the_app_the_engine_or_the_text_library_changes():
    """Label: property"""
    jobs = workflow(APP_GATE)["jobs"]
    assert jobs["web-app"]["needs"] == "changes"
    assert jobs["web-app"]["if"] == "needs.changes.outputs.app == 'true'"
    pattern = paths_pattern()
    for path in ["app/src/App.svelte", "crates/ovd-core/src/lib.rs", "locales/en/app.ftl", "Cargo.lock", "Cargo.toml",
                 "rust-toolchain.toml", "tools/build_web.sh", ".github/workflows/app-gate.yml"]:
        assert pattern.search(path), path
    for path in ["README.md", "docs/adr/0026-x.md", "tests/test_x.py", "website/index.html", "src/ovd/__init__.py"]:
        assert not pattern.search(path), path


def test_the_changed_paths_step_passes_a_shell_check():
    """Label: property"""
    run = next(s["run"] for s in workflow(APP_GATE)["jobs"]["changes"]["steps"] if s.get("id") == "paths")
    assert subprocess.run(["bash", "-n"], input=run, text=True).returncode == 0


def test_no_workflow_runs_the_paused_desktop_app():
    """Label: property"""
    assert not (WORKFLOWS / "app-checks.yml").exists()
    assert set(workflow(APP_GATE)["jobs"]) == {"changes", "web-app"}
    for path in WORKFLOWS.glob("*.yml"):
        text = path.read_text(encoding="utf-8")
        assert "app-checks.yml" not in text, path.name
        assert "app-e2e" not in text, path.name
