"""An independent oracle for the deckhouse opening Rules of the IGC Code (issue #913).

It never calls the engine and shares no code with it (standard library only). It works out each verdict
from the text of the IGC Code, resolution MSC.370(93), 3.2.4.1 and 3.2.4.2, and from IGC 1.2.17 and 1.2.31,
not from the Rule Set data:

- 3.2.4.1: entrances, air inlets and openings to accommodation spaces, service spaces and machinery spaces
  and control stations shall not face the cargo area. They shall be located on the end bulkhead not facing
  the cargo area and/or on the outboard side of the superstructure or deckhouse at a distance of at least
  4 % of the length (L) of the ship but not less than 3 m from the end of the superstructure or deckhouse
  facing the cargo area. This distance, however, need not exceed 5 m.
- 3.2.4.2: windows and sidescuttles facing the cargo area, and on the sides of the superstructure or deckhouse
  within the distance of 3.2.4.1, shall be of the fixed type. Wheelhouse windows may be non-fixed, and
  wheelhouse doors may be located within the above limits so long as they are designed in such a manner that
  the wheelhouse can be made rapidly and efficiently gas and vapour tight.
- 1.2.17: the wheelhouse (the main navigating equipment) is a control station. So an entrance of the
  wheelhouse must not face the cargo area, gas-tight or not: the 3.2.4.2 relief is for the side zone only (the
  maintainer's strict reading, issue #913).
- 1.2.31: L is the load line length, ICLL Regulation 3(1)(a).

A case (`kind: openings`) gives L and a list of openings, each with a kind, a space, a face, its distance from
the deckhouse front, and (for a window or door) whether it opens and whether it is designed gas-tight. The
oracle gives one verdict for each Rule, the worst of the openings the Rule covers; a Rule that covers none of
them has no entry (the engine has no applicable row for it). The distance at the zone limit meets it (at
least 4 % of L, not less than 3 m, at most 5 m).

    python3 evaluation/oracle/openings_oracle.py CASE.yaml     # print the verdicts of one case
"""
from __future__ import annotations

import sys
from pathlib import Path

CARGO_FACE = "igc/3.2.4.1/no-opening-faces-cargo"
SIDE_ENTRANCE = "igc/3.2.4.1/side-opening-setback-placed"
AIR_INLET = "igc/3.2.4.1/air-inlet-setback-placed"
WINDOWS = "igc/3.2.4.2/windows-fixed"
WHEELHOUSE = "igc/3.2.4.2/wheelhouse-gas-tight"

#: The spaces of 3.2.4.1, and the wheelhouse that 1.2.17 makes one of them.
SPACES = ("accommodation", "service", "machinery", "control_station")


def zone_m(l_m: float) -> float:
    """The distance of 3.2.4.1: at least 4 % of L, not less than 3 m, and it need not exceed 5 m."""
    return min(5.0, max(3.0, 0.04 * l_m))


def _meets(distance: float, limit: float) -> bool:
    return distance >= limit


def covered(rule: str, o: dict) -> bool:
    """Whether the clause reaches this opening."""
    kind, space, face = o["kind"], o["space"], o["face"]
    opens = bool(o.get("openable", kind != "window"))
    tight = bool(o.get("gas_tight", False))
    if rule == CARGO_FACE:
        return kind in ("entrance", "air_inlet", "air_intake") and (space in SPACES or space == "wheelhouse")
    if rule == SIDE_ENTRANCE:
        return kind == "entrance" and space in SPACES and face == "side"
    if rule == AIR_INLET:
        return kind in ("air_inlet", "air_intake") and face == "side"
    if rule == WINDOWS:
        return kind == "window" and space in SPACES and face in ("front", "side") and opens
    if rule == WHEELHOUSE:
        return space == "wheelhouse" and kind in ("entrance", "window") and face == "side" and opens and not tight
    raise KeyError(rule)


def verdicts(i: dict) -> dict:
    """The expected verdict of every Rule that covers an opening of the case, by rule id."""
    limit = zone_m(i["l_icll_m"])
    out = {}
    for rule in (CARGO_FACE, SIDE_ENTRANCE, AIR_INLET, WINDOWS, WHEELHOUSE):
        hit = [o for o in i["openings"] if covered(rule, o)]
        if not hit:
            continue
        if rule == CARGO_FACE:
            bad = [o for o in hit if o["face"] == "front"]
            out[rule] = {"rule": "IGC 3.2.4.1", "what": "no entrance, air inlet or air intake faces the cargo area",
                         "unit": "-", "verdict": "fail" if bad else "pass", "value": not bad, "limit": True}
            continue
        # The distance from the deckhouse front: nought on the front itself.
        d = min(0.0 if o["face"] == "front" else float(o["distance_from_front_m"]) for o in hit)
        text = "IGC 3.2.4.1" if rule in (SIDE_ENTRANCE, AIR_INLET) else "IGC 3.2.4.2"
        what = {SIDE_ENTRANCE: "side entrance from the deckhouse front",
                AIR_INLET: "side air inlet or intake from the deckhouse front",
                WINDOWS: "openable window facing the cargo area or in the side zone, from the deckhouse front",
                WHEELHOUSE: "wheelhouse side door or window not designed gas-tight, from the deckhouse front"}[rule]
        out[rule] = {"rule": text, "what": what, "unit": "m",
                     "verdict": "pass" if _meets(d, limit) else "fail",
                     "value": float(f"{d:.10g}"), "limit": float(f"{limit:.10g}"),
                     "margin": float(f"{d - limit:.10g}")}
    return out


def main(argv: list[str]) -> int:
    import yaml
    case = yaml.safe_load(Path(argv[1]).read_text(encoding="utf-8"))
    for rule_id, e in verdicts(case["inputs"]).items():
        print(f"{rule_id}: {e['verdict']} (value {e['value']}, limit {e['limit']})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
