"""An independent oracle for the deckhouse opening Rules of the IGC Code (issue #913).

It never calls the engine and shares no code with it (standard library only), and it does not read the Rule
Set file: the scope of each clause is written here from the text of the IGC Code, resolution MSC.370(93),
3.2.4.1 and 3.2.4.2 (`MSC.370-93.txt` lines 1494-1505), 1.2.17 and 1.2.31:

- 3.2.4.1, first sentence: entrances, air inlets and openings to accommodation spaces, service spaces,
  machinery spaces and control stations shall not face the cargo area.
- 3.2.4.1, second sentence: they shall be located on the end bulkhead not facing the cargo area and/or on the
  outboard side at a distance of at least 4 % of L, not less than 3 m, from the end of the superstructure or
  deckhouse facing the cargo area; the distance need not exceed 5 m.
- 3.2.4.2: windows and sidescuttles facing the cargo area, and on the sides within that distance, shall be of
  the fixed type. Wheelhouse windows may be non-fixed, and wheelhouse doors may be located within the above
  limits, so long as they are designed so that the wheelhouse can be made rapidly and efficiently gas and
  vapour tight. A window or door of the wheelhouse that is not so designed has no relief: it is held to the
  same limits as any other (the maintainer's reading, issue #913).
- 1.2.17: the wheelhouse (the main navigating equipment) is a control station. So a wheelhouse entrance may
  not face the cargo area, gas-tight or not: the 3.2.4.2 relief is for the side zone only (strict reading).
- 1.2.31: L is the load line length, ICLL Regulation 3(1)(a).

The oracle goes opening by opening. For each opening it finds the clause sentences that bind it and tests it
against each. A clause is named by the Rule id the engine uses for it; the expected verdict of that id is the
worst of the openings the clause binds, `pass` if none is wrong. An opening the text binds always gives an
entry, so an engine with no Rule for it shows as a mismatch (the engine has no row: `not evaluated`), never as
a missing expectation. An opening the text does not bind (a gas-tight wheelhouse door within the zone, a fixed
window) gives none.

A case (`kind: openings`) gives L and a list of openings, each with a kind, a space, a face, its distance from
the deckhouse front, and for a window or door whether it opens and whether it is designed gas-tight.

    python3 evaluation/oracle/openings_oracle.py CASE.yaml     # print the verdicts of one case
"""
from __future__ import annotations

import sys
from pathlib import Path

# The clauses, named by the id of the Rule that checks each.
FACE = "igc/3.2.4.1/no-opening-faces-cargo"           # 3.2.4.1, sentence 1
SIDE_ENTRANCE = "igc/3.2.4.1/side-opening-setback-placed"  # 3.2.4.1, sentence 2, entrances
AIR_INLET = "igc/3.2.4.1/air-inlet-setback-placed"    # 3.2.4.1, sentence 2, air inlets
WINDOWS = "igc/3.2.4.2/windows-fixed"                 # 3.2.4.2, sentence 1
WHEELHOUSE = "igc/3.2.4.2/wheelhouse-gas-tight"       # 3.2.4.2, sentence 2

#: The spaces the first sentence of 3.2.4.1 names, and the wheelhouse, a control station (1.2.17).
NAMED = ("accommodation", "service", "machinery", "control_station", "wheelhouse")


def zone_m(l_m: float) -> float:
    """At least 4 % of L, not less than 3 m, and it need not exceed 5 m."""
    return min(5.0, max(3.0, 0.04 * l_m))


def _distance(o: dict) -> float:
    """From the end of the deckhouse facing the cargo area: nought for an opening on that end itself."""
    return 0.0 if o["face"] == "front" else float(o["distance_from_front_m"])


def binds(o: dict, limit: float) -> list[tuple[str, bool]]:
    """The clauses that bind one opening, each with whether the opening meets it."""
    kind, space, face = o["kind"], o["space"], o["face"]
    opens = bool(o.get("openable", kind != "window"))
    tight = bool(o.get("gas_tight", False))
    wheelhouse = space == "wheelhouse"
    out: list[tuple[str, bool]] = []
    if space not in NAMED:
        return out
    # 3.2.4.1, sentence 1: entrances, air inlets and openings do not face the cargo area.
    if kind in ("entrance", "air_inlet", "air_intake"):
        out.append((FACE, face != "front"))
    # 3.2.4.1, sentence 2: on a side, at the distance from the end facing the cargo area.
    if face == "side" and kind == "entrance" and not wheelhouse:
        out.append((SIDE_ENTRANCE, _distance(o) >= limit))
    if face == "side" and kind in ("air_inlet", "air_intake"):
        out.append((AIR_INLET, _distance(o) >= limit))
    # 3.2.4.2, sentence 1: windows facing the cargo area and on the sides within the distance are fixed.
    if kind == "window" and opens and face in ("front", "side") and not wheelhouse:
        out.append((WINDOWS, _distance(o) >= limit))
    # 3.2.4.2, sentence 2: a wheelhouse door or window within the limits (the front end, or a side nearer
    # than the distance) only if designed gas-tight. A wheelhouse door on the front already fails sentence 1.
    if wheelhouse and kind in ("entrance", "window") and opens and face in ("front", "side") and not tight:
        out.append((WHEELHOUSE, _distance(o) >= limit))
    return out


def verdicts(i: dict) -> dict:
    """The expected verdict of every clause that binds an opening of the case, by Rule id."""
    limit = zone_m(i["l_icll_m"])
    held: dict[str, list[tuple[dict, bool]]] = {}
    for o in i["openings"]:
        for clause, ok in binds(o, limit):
            held.setdefault(clause, []).append((o, ok))
    out = {}
    what = {FACE: "no entrance, air inlet or air intake faces the cargo area",
            SIDE_ENTRANCE: "side entrance from the deckhouse front",
            AIR_INLET: "side air inlet or intake from the deckhouse front",
            WINDOWS: "openable window facing the cargo area or in the side zone, from the deckhouse front",
            WHEELHOUSE: "wheelhouse door or window within the limits that is not designed gas-tight, from the deckhouse front"}
    for clause, rows in held.items():
        text = "IGC 3.2.4.1" if clause in (FACE, SIDE_ENTRANCE, AIR_INLET) else "IGC 3.2.4.2"
        bad = [o for o, ok in rows if not ok]
        if clause == FACE:
            out[clause] = {"rule": text, "what": what[clause], "unit": "-",
                           "verdict": "fail" if bad else "pass", "value": not bad, "limit": True}
            continue
        d = min(_distance(o) for o, _ in rows)
        out[clause] = {"rule": text, "what": what[clause], "unit": "m",
                       "verdict": "fail" if bad else "pass",
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
