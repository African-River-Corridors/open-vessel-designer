"""Plain words in the app (spec 0017, D12; issue #1082).

No message the app can show tells the designer to run the command-line program or to use the Python
API. The engine's own message keeps its hint, for the command line and the Python API. The app shows
another message for the same id, `app-plain-<id>` in `locales/en/app.ftl`. This test reads the whole
catalogue, so a new message is covered with no change here.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import i18n_check  # noqa: E402

LOCALES = ROOT / "locales"
# The program's name and a sub-command (`ovd fill`, `ovd run --json`), the words "simulate call" or
# the Python API's own names.
COMMAND_LINE = re.compile(r"\bovd\s+[a-z]|(?i:\bsimulate\b|\bpython\b)")
APP_OVERRIDE = "app-plain-"


def messages(path: Path) -> dict[str, i18n_check.Message]:
    return i18n_check.parse(path).messages if path.exists() else {}


def languages() -> list[Path]:
    return sorted(d for d in LOCALES.iterdir() if d.is_dir())


def body(m: i18n_check.Message) -> str:
    return m.source.split("=", 1)[1]


def test_no_text_of_the_app_names_the_command_line_or_the_python_api():
    """Label: property"""
    found = []
    for folder in languages():
        for name in ("app.ftl", "inputs.ftl"):
            for m in messages(folder / name).values():
                if COMMAND_LINE.search(body(m)):
                    found.append(f"{folder.name}/{name}: {m.id}")
    assert found == []


def test_every_engine_message_with_a_command_line_hint_has_app_wording():
    """Label: property"""
    app = messages(LOCALES / "en" / "app.ftl")
    hinted = []
    for folder in languages():
        for m in messages(folder / "engine.ftl").values():
            if COMMAND_LINE.search(body(m)):
                hinted.append(m.id)
    assert "msg-service-area-required" in hinted, "the catalogue still holds the Service Area hint"
    missing = sorted({i for i in hinted if APP_OVERRIDE + i not in app})
    assert missing == [], f"no app wording (add app-plain-<id> to locales/en/app.ftl): {missing}"


def test_app_wording_is_plain_and_fits_the_engine_message_it_replaces():
    """Label: property"""
    app = messages(LOCALES / "en" / "app.ftl")
    engine = messages(LOCALES / "en" / "engine.ftl")
    wording = {k: m for k, m in app.items() if k.startswith(APP_OVERRIDE)}
    assert wording, "no app wording yet"
    for key, m in wording.items():
        ident = key[len(APP_OVERRIDE):]
        assert ident in engine, f"{key} replaces {ident}, which is not an engine message"
        assert not COMMAND_LINE.search(body(m)), key
        assert m.placeholders <= engine[ident].placeholders, f"{key} uses an argument {ident} does not have"


def test_the_service_area_error_reads_as_the_brief_says():
    """Label: property"""
    app = messages(LOCALES / "en" / "app.ftl")
    text = body(app[APP_OVERRIDE + "msg-service-area-required"]).strip()
    assert text == "Choose a Service Area: river only, or coastal and river."


def test_the_command_line_keeps_its_hint():
    """Label: property"""
    engine = messages(LOCALES / "en" / "engine.ftl")
    assert "ovd fill --service-area" in body(engine["msg-service-area-required"])
