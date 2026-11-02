"""tools/refresh_examples.py: the README sensitivities table, the examples folder, the options (issue #857)."""
import importlib.util
import sys
from pathlib import Path

import pytest

SPEC = importlib.util.spec_from_file_location("refresh_examples", Path(__file__).resolve().parents[1] / "tools" / "refresh_examples.py")
tool = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(tool)

TABLE = """# x

| Option | Change from A | Tank OD (m) | Cargo (m3) | Cargo (t) | Draught (m) |
|---|---|---:|---:|---:|---:|
| I lift | a | 1.00 | 1 | 1 | 1.00 |
| M river | b | 1.00 | 1 | 1 | 1.00 |

after
"""


def option(name, cargo=5.0):
    return {"name": name, "results": {"arrangement": {"tank_od_m": 11.0}, "results": {"cargo_volume_m3": 1000.0, "cargo_t": cargo, "draught_m": 3.0}}}


def readme(tmp_path, text=TABLE):
    (tmp_path / "README.md").write_text(text)
    return tmp_path


def test_sensitivities_cells_are_replaced(tmp_path):
    """Label: property"""
    d = readme(tmp_path)
    assert tool.update_sensitivities(d, [option("I lift"), option("M river", cargo=7.0)]) == []
    text = (d / "README.md").read_text()
    assert "| M river | b | 11.00 | 1,000 | 7 | 3.00 |" in text and text.endswith("after\n")


def test_a_renamed_row_fails(tmp_path):
    """Label: property"""
    d = readme(tmp_path, TABLE.replace("| M river |", "| M  river |"))
    problems = tool.update_sensitivities(d, [option("I lift"), option("M river")])
    assert any("'M  river' matches no Option" in p for p in problems)
    assert any("'M river' has no row" in p for p in problems)


def test_a_row_with_the_wrong_cell_count_fails(tmp_path):
    """Label: property"""
    d = readme(tmp_path, TABLE.replace("| M river | b | 1.00 |", "| M river | b |"))
    assert any("cells, not 6" in p for p in tool.update_sensitivities(d, [option("I lift"), option("M river")]))


def test_an_option_with_no_row_fails(tmp_path):
    """Label: property"""
    d = readme(tmp_path)
    assert any("'K both' has no row" in p for p in tool.update_sensitivities(d, [option("I lift"), option("M river"), option("K both")]))


def test_a_missing_table_fails(tmp_path):
    """Label: property"""
    assert tool.update_sensitivities(readme(tmp_path, "# nothing\n"), [option("I lift")])


def test_every_examples_folder_is_handled():
    """Label: property"""
    assert tool.unhandled_examples() == []


def test_an_unlisted_example_folder_fails(tmp_path):
    """Label: property"""
    (tmp_path / "examples" / "new-one").mkdir(parents=True)
    (tmp_path / "examples" / "new-one" / "a.vdesign").write_text("{}")
    name = tool.EXAMPLES[0][0]
    (tmp_path / "examples" / name).mkdir()
    (tmp_path / "examples" / name / "stray.vdesign").write_text("{}")
    problems = tool.unhandled_examples(tmp_path)
    assert any("new-one" in p for p in problems) and any("stray.vdesign" in p for p in problems)


def test_check_with_from_results_is_refused(monkeypatch):
    """Label: property"""
    monkeypatch.setattr(sys, "argv", ["refresh_examples.py", "--check", "--from-results", "r.json"])
    with pytest.raises(SystemExit) as e:
        tool.main()
    assert e.value.code != 0


def test_an_untabulated_option_must_match_the_declared_pattern(tmp_path):
    """Label: property"""
    d = readme(tmp_path)
    opts = [option("I lift"), option("M river"), option("N1 Cb 0.97"), option("Z other")]
    problems = tool.update_sensitivities(d, opts, r"[NP][1-9] Cb \d\.\d+")
    assert problems == ["sensitivities Option 'Z other' has no row in the README table"]
