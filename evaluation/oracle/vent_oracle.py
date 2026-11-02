"""An independent oracle for the IGC cargo vent exit Rules (issue #849).

It never calls the engine and shares no code with it (standard library only). It works out each limit and
verdict from the text of the IGC Code, resolution MSC.370(93), 8.2.10.3, 8.2.10.4 and 8.2.11.1, not from the
Rule Set data:

- 8.2.10.3: the height of vent exits is not less than B/3 or 6 m, whichever is the greater, above the weather deck.
- 8.2.10.4: and 6 m above working areas and walkways.
- 8.2.11.1: cargo PRV vent exits at a distance at least equal to B or 25 m, whichever is less, from the nearest
  air intake, outlet or opening to accommodation spaces, service spaces and control stations, or other
  non-hazardous areas. (For ships under 90 m in length, smaller distances may be permitted: not applied.)

A case (`kind: vent`) gives the beam and the three numbers a run reads from its results: the exit height above
the deck, the exit height above the working area, and the distance to the nearest opening. The boundary cases
move one number to each side of its limit. The Rules apply to the Service Area `coastal-and-river` only; the
cases are all on it.

    python3 evaluation/oracle/vent_oracle.py CASE.yaml      # print the verdicts of one case
"""
from __future__ import annotations

import sys
from pathlib import Path

DECK = "igc/8.2.10.3/vent-exit-height"
WORKING = "igc/8.2.10.4/vent-exit-above-working-areas"
DISTANCE = "igc/8.2.11.1/vent-exit-distance"

#: The measured values, by the Rule they are read for, and the input that holds each.
MOVES = {DECK: "vent_exit_above_deck_m", WORKING: "vent_exit_above_working_area_m",
         DISTANCE: "vent_distance_to_openings_m"}

#: Step from the limit in metres: (big, small). Smaller than the 0.01 m step of a rule mutant of a constant
#: limit, so a mutant lies across a case.
STEPS = {"m": (0.005, 0.0001)}


def _entry(rule: str, what: str, value: float, limit: float) -> dict:
    ok = value >= limit
    return {"rule": rule, "what": what, "unit": "m", "compare": "ge",
            "verdict": "pass" if ok else "fail", "value": float(f"{value:.10g}"),
            "limit": float(f"{limit:.10g}"), "margin": float(f"{value - limit:.10g}")}


def verdicts(i: dict) -> dict:
    """The expected verdict of every Rule that applies to the case, by rule id."""
    b = i["beam_m"]
    out = {}
    out[DECK] = _entry("IGC 8.2.10.3", "cargo vent exit above the weather deck",
                       i["vent_exit_above_deck_m"], max(b / 3.0, 6.0))
    out[WORKING] = _entry("IGC 8.2.10.4", "cargo vent exit above working areas and walkways",
                          i["vent_exit_above_working_area_m"], 6.0)
    out[DISTANCE] = _entry("IGC 8.2.11.1", "cargo vent exit from the nearest opening",
                           i["vent_distance_to_openings_m"], min(b, 25.0))
    return out


class Unreachable(Exception):
    pass


def boundary_case(base: dict, rule_id: str, side: str, magnitude: str) -> dict:
    """A base case moved to one side of one Rule's limit, with the oracle's verdicts for every Rule."""
    i = base["inputs"]
    entry = verdicts(i)[rule_id]
    big, small = STEPS["m"]
    step = big if magnitude == "big" else small
    target = entry["limit"] + (step if side == "pass" else -step)
    name = MOVES[rule_id]
    inputs = dict(i)
    inputs[name] = float(f"{target:.12g}")
    expected = verdicts(inputs)
    slug = rule_id.replace("/", "_")
    return {
        "id": f"{base['id']}__{slug}__{side}_{magnitude}",
        "title": f"{base['title']}, {rule_id} just {'inside' if side == 'pass' else 'across'} its limit ({magnitude} step)",
        "kind": "vent",
        "inputs": inputs,
        "expected": {k: {kk: vv for kk, vv in v.items() if kk != "compare"} for k, v in expected.items()},
        "mutant": {"parent": base["id"], "criterion": rule_id, "side": side, "magnitude": magnitude,
                   "moved": name, "target_value": float(f"{target:.10g}"), "grid": False},
        "provenance": {
            "verdict_basis": "derived",
            "source": "evaluation/oracle/vent_oracle.py from the parent case",
            "edition": base["provenance"]["edition"], "locator": base["provenance"]["locator"],
            "licence": base["provenance"]["licence"],
            "confidence": "high for the verdicts; a mutant is not an external published example",
        },
    }


def generate(bases: list[dict]) -> tuple[list[dict], list[dict]]:
    """(boundary cases, skipped) of the vent base cases."""
    docs = []
    for base in bases:
        for rule_id in verdicts(base["inputs"]):
            for side in ("pass", "fail"):
                docs.append(boundary_case(base, rule_id, side, "big"))
    return docs, []


def main(argv: list[str]) -> int:
    import yaml
    case = yaml.safe_load(Path(argv[1]).read_text(encoding="utf-8"))
    for rule_id, e in verdicts(case["inputs"]).items():
        print(f"{rule_id}: {e['verdict']} (value {e['value']}, limit {e['limit']})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
