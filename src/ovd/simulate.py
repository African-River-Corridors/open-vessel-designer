"""One-call barge simulation for the simulator.

    simulate(params: dict, damage: bool = False) -> dict      # JSON in, JSON out

One path (issue #937): the dict becomes one Option the way `ovd fill --as-new` builds it (a key left out
takes the Defaults Profile value; a required input left out is an error), and the Option is run the way
`ovd run` runs it. The result is `run`'s results. The calculation runs in the Rust core
(`ovd._native`); this module only passes the parameters through. Pure calculation, no I/O.
"""
from __future__ import annotations

from ._bridge import call as _call

# 0.1.0 `simulate` exposed these names (it imported them), and user scripts import them from here.
# Nothing in this module uses them: the calculation runs in the Rust core.
from .core.barge_capacity import BargeCapacityInputs, compute as capacity  # noqa: F401
from .core.cargo import CARGO_DATABASE  # noqa: F401
from .core.gas_arrangement import DEFAULT_DEPTHS_M, RULESETS, fit as gas_fit  # noqa: F401
from .core.gas_stability import assess as assess_stability, criteria_set_for  # noqa: F401
from .core.resistance import (  # noqa: F401
    EnvironmentInputs, ResistanceInputs, TugInputs, compute as resistance)

RHO = 1.0  # fresh water, t/m3
# Tank filling limit by ruleset (defect 7): IGC 15.3 default 98 %, for the IGC rule set.
FILL_BY_RULESET = {"IGC_2G": 0.98}

VESSELS = {
    # id: (label, hold_type for the core, allowed cargo phases)
    "box_barge": ("Box-hold dry bulk barge", "box", {"dry_bulk"}),
    "hopper_barge": ("Sloped-hopper dry bulk barge", "sloped_hopper", {"dry_bulk"}),
    "gas_barge": ("Gas barge, IMO Type C tanks", None, {"liquid_gas"}),
}


def cargo_options() -> list[dict]:
    """Every cargo the simulator offers: id, name, phase and density."""
    return _call("simulate", "cargo_options", {})


def simulate(p: dict, damage: bool = False) -> dict:
    """Run the one-call simulation on the parameters ``p`` and return the result as a dict.

    ``p`` gives the inputs of one Option, by their glossary names or by the engine names of the old
    dict (``loa_m``, ``vessel``, ``channel`` and so on); see ``docs/python-api.md``. A key left out
    takes the Defaults Profile value. An input that is refused (unknown, not used, out of range, a
    required one left out) raises ``TypeError`` naming every one. ``damage=True`` also runs the damage
    stability cases; by default they are not run and ``damage_status`` is ``"not run"``.
    """
    return _call("api", "simulate", {"params": p, "damage": damage})
