"""The Text Library check (spec 0005): keeps the Fluent files whole, fast, with the standard library only.

    python3 tools/i18n_check.py          # report; exit 1 on any problem

It checks:
  - every id the code uses is in English (missing keys);
  - every English id is used by the code (unused keys), except the derived ids (`input-`, `choice-`,
    `group-`, and any prefix the code builds with a template string), which a Rust test checks;
  - the engine's and the drawings' English match the Rust source, and `inputs.ftl` matches the input table;
  - `locales/` holds English only: the other Display Languages are paused for now (ADR 0026). The
    checks of translated files (orphans, placeholders, review status, the Term List) left with them;
    the last commit that has them is named in the ADR.
"""
from __future__ import annotations

import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = "en"
DERIVED = ("input-", "choice-", "group-")
CODE = [("app/src", (".ts", ".svelte")), ("app/index.html", ()), ("catalogue/report", (".json",)), ("crates", (".rs",))]
# In Rust only the id of a `text!` call counts: other quoted kebab strings there are not Message Ids.
RUST_USED = re.compile(r'\b(?:text!|label!|Text::fixed)\(\s*"([a-z0-9-]+)"')

MESSAGE = re.compile(r"^([a-zA-Z][a-zA-Z0-9_-]*)\s*=(.*)$")
PLACEHOLDER = re.compile(r"\{\s*\$([a-zA-Z][a-zA-Z0-9_-]*)")
# A Message Id in code: a quoted literal of an id with a known prefix, or the head of a template string.
USED = re.compile(r"""["'`]((?:app|check|warn|stab|name|msg|draw|err)-[a-z0-9-]+)["'`]""")
BUILT = re.compile(r"`((?:app|check|warn|stab|name|msg|draw|err|input|choice|group)-[a-z0-9-]*)\$\{")


@dataclass
class Message:
    id: str
    source: str
    placeholders: set[str]
    line: int = 0


@dataclass
class File:
    path: Path
    messages: dict[str, Message] = field(default_factory=dict)
    problems: list[str] = field(default_factory=list)


def parse(path: Path) -> File:
    """Read the messages of one Fluent file: id, source lines and placeholders."""
    out = File(path)
    lines = path.read_text(encoding="utf-8").splitlines()
    current: Message | None = None
    for n, line in enumerate(lines, 1):
        m = MESSAGE.match(line)
        if m:
            current = Message(m.group(1), line.rstrip(), set(), n)
            if current.id in out.messages:
                out.problems.append(f"{path.relative_to(ROOT).as_posix()}:{n}: duplicate id {current.id}")
            out.messages[current.id] = current
        elif line.startswith((" ", "\t")) and current and line.strip():
            current.source += "\n" + line.rstrip()
        else:
            current = None
            if line.strip() and not line.startswith("#"):
                out.problems.append(f"{path.relative_to(ROOT).as_posix()}:{n}: not a message or a comment: {line[:60]}")
    for msg in out.messages.values():
        msg.placeholders = set(PLACEHOLDER.findall(msg.source.split("=", 1)[1]))
    return out


def code_ids(root: Path) -> tuple[set[str], set[str]]:
    """The ids the code names, and the prefixes it builds ids from."""
    used: set[str] = set()
    built: set[str] = set()
    for rel, suffixes in CODE:
        base = root / rel
        files = [base] if base.is_file() else [p for p in base.rglob("*") if p.suffix in suffixes and "node_modules" not in p.parts and "target" not in p.parts]
        for p in files:
            text = p.read_text(encoding="utf-8", errors="replace")
            if p.suffix == ".rs":
                if p.name != "text.rs":  # the macro's own example
                    used.update(RUST_USED.findall(text))
                continue
            used.update(USED.findall(text))
            used.update(re.findall(r'data-i18n(?:-aria)?="([a-z0-9-]+)"', text))
            built.update(BUILT.findall(text))
    if root == ROOT:  # the pinned otd crate's `label!` calls are code too (tools/otd.py)
        import otd

        for p in (otd.root() / "src").rglob("*.rs"):
            used.update(RUST_USED.findall(p.read_text(encoding="utf-8")))
    return used, built


def check(root: Path = ROOT) -> list[str]:
    locales = root / "locales"
    problems: list[str] = []
    english: dict[str, Message] = {}
    english_files: dict[str, File] = {}
    for p in sorted((locales / SOURCE).glob("*.ftl")):
        f = parse(p)
        problems += f.problems
        english_files[p.name] = f
        for k, v in f.messages.items():
            if k in english:
                problems.append(f"{p.relative_to(root).as_posix()}:{v.line}: id {k} is also in another English file")
            english[k] = v

    import i18n_extract  # the engine's English comes from the Rust source; engine.ftl must match it

    problems += i18n_extract.problems(root)
    problems += check_input_labels(root, english)
    used, built = code_ids(root)
    for k in sorted(used - english.keys()):
        problems.append(f"missing: {k} is used in the code but not in locales/{SOURCE}")
    skip = DERIVED + tuple(built)
    for k in sorted(english.keys() - used):
        if not k.startswith(skip):
            problems.append(f"unused: {k} is in locales/{SOURCE} but the code does not use it")

    for other in sorted(p for p in locales.iterdir() if p.name != SOURCE and not p.name.startswith(".")):
        problems.append(f"{other.relative_to(root).as_posix()}: English only for now (ADR 0026): only locales/{SOURCE} ships")
    return problems


# The input table: each `spec(…)`-style constructor call gives an input's key and its English label.
INPUT_SPEC = re.compile(r'\b(?:spec|only|only_in|pushed|self_propelled|lpg|tug|atb)\(\s*"([a-z0-9_]+)",\s*"((?:[^"\\]|\\.)*)"')


def check_input_labels(root: Path, english: dict[str, "Message"]) -> list[str]:
    """`inputs.ftl` holds each input of the input table with its label: a stale file fails here, fast.
    The Rust test `crates/ovd-format/tests/i18n_inputs.rs` checks the rest (choices, groups, notes)."""
    path = root / "crates/ovd-format/src/vdesign.rs"
    if not path.exists():
        return []
    src = path.read_text(encoding="utf-8")
    start = src.find("pub const INPUTS")
    if start < 0:
        return ["crates/ovd-format/src/vdesign.rs: no input table (`pub const INPUTS`) found"]
    table = {f"input-{k.replace('_', '-')}": label for k, label in INPUT_SPEC.findall(src[start:src.index("];", start)])}
    rerun = "run OVD_WRITE_TEXT=1 cargo test -p ovd-format --test i18n_inputs"
    out = []
    for k, label in table.items():
        value = english[k].source.split("=", 1)[1].strip() if k in english else None
        if value is None:
            out.append(f"locales/en/inputs.ftl is stale: {k} is not in it ({rerun})")
        elif value != label:
            out.append(f"locales/en/inputs.ftl is stale: {k} is {value!r}, the code says {label!r} ({rerun})")
    for k in english:
        if k.startswith("input-") and not k.endswith("-default-by") and k not in table:
            out.append(f"locales/en/inputs.ftl is stale: {k} is not an input ({rerun})")
    return out


def main(argv: list[str]) -> int:
    problems = check()
    for p in problems:
        print(p)
    print(f"i18n check: {len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
