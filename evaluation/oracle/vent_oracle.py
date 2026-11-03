"""An independent oracle for the cargo vent exit Rules: the IGC Code (issue #849) and ADN 9.3.1.22.3 (issue #850).

It never calls the engine and shares no code with it (standard library only). It works out each limit and
verdict from the text of the IGC Code, resolution MSC.370(93), 8.2.10.3, 8.2.10.4 and 8.2.11.1, not from the
Rule Set data:

- 8.2.10.3: the height of vent exits is not less than B/3 or 6 m, whichever is the greater, above the weather deck.
- 8.2.10.4: and 6 m above working areas and walkways.
- 8.2.11.1: cargo PRV vent exits at a distance at least equal to B or 25 m, whichever is less, from the nearest
  air intake, outlet or opening to accommodation spaces, service spaces and control stations, or other
  non-hazardous areas. (For ships under 90 m in length, smaller distances may be permitted: not applied.)

ADN 2025 (ECE/TRANS/351) 9.3.1.22.3, printed p. 523, applies on the Service Area `river-only`: the exhaust outlets of
the pressure relief valves are not less than 2.00 m above the deck, at a distance of not less than 6.00 m from the
accommodation and from the service spaces outside the cargo area. (The reduction of the height when nothing is
within 1.00 m of the outlet is not applied.)

A case (`kind: vent`) gives the beam and the three numbers a run reads from its results: the exit height above
the deck, the exit height above the working area, and the distance to the nearest opening. The boundary cases
move one number to each side of its limit by 0.005 m, exactly onto it (meets), and one float step under it (fails). The IGC Rules apply to the Service Area `coastal-and-river`, the ADN Rules to
`river-only`; a river-only case also gives the distance to the accommodation.

    python3 evaluation/oracle/vent_oracle.py CASE.yaml      # print the verdicts of one case
"""
from __future__ import annotations

import math
import sys
from pathlib import Path

DECK = "igc/8.2.10.3/vent-exit-height"
WORKING = "igc/8.2.10.4/vent-exit-above-working-areas"
DISTANCE = "igc/8.2.11.1/vent-exit-distance"
ADN_HEIGHT = "adn/9.3.1.22.3/relief-valve-outlet-height"
ADN_DISTANCE = "adn/9.3.1.22.3/relief-valve-outlet-distance"

#: The measured values, by the Rule they are read for, and the input that holds each.
MOVES = {DECK: "vent_exit_above_deck_m", WORKING: "vent_exit_above_working_area_m",
         DISTANCE: "vent_distance_to_openings_m", ADN_HEIGHT: "vent_exit_above_deck_m",
         ADN_DISTANCE: "vent_distance_to_accommodation_m"}

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
    if i["service_area"] == "river-only":
        out[ADN_HEIGHT] = _entry("ADN 9.3.1.22.3", "relief valve outlet above the deck",
                                 i["vent_exit_above_deck_m"], 2.0)
        out[ADN_DISTANCE] = _entry("ADN 9.3.1.22.3", "relief valve outlet from the accommodation",
                                   i["vent_distance_to_accommodation_m"], 6.0)
        return out
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
    name = MOVES[rule_id]
    # The limit as the text works it out, not rounded for the file.
    limit = {DECK: max(i["beam_m"] / 3.0, 6.0), WORKING: 6.0, DISTANCE: min(i["beam_m"], 25.0),
             ADN_HEIGHT: 2.0, ADN_DISTANCE: 6.0}[rule_id]
    if magnitude == "exact":
        target = limit
    elif magnitude == "ulp":
        # One float step under the limit: the value a difference that rounds down gives.
        target = math.nextafter(limit, -math.inf)
    else:
        step = big if magnitude == "big" else small
        target = limit + (step if side == "pass" else -step)
    inputs = dict(i)
    inputs[name] = target if magnitude in ("exact", "ulp") else float(f"{target:.12g}")
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
            for side, magnitude in (("pass", "big"), ("fail", "big"), ("pass", "exact"), ("fail", "ulp")):
                docs.append(boundary_case(base, rule_id, side, magnitude))
    return docs, []


def main(argv: list[str]) -> int:
    import yaml
    case = yaml.safe_load(Path(argv[1]).read_text(encoding="utf-8"))
    for rule_id, e in verdicts(case["inputs"]).items():
        print(f"{rule_id}: {e['verdict']} (value {e['value']}, limit {e['limit']})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
