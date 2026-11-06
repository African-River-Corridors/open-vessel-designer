"""One-call barge simulation for the simulator.

    simulate(params: dict, damage: bool = False) -> dict      # JSON in, JSON out
    simulate_damage(params, progress=None, cancel=None) -> dict   # the damage stage, with progress and cancel

One path (issue #937): the dict becomes one Option the way `ovd fill --as-new` builds it (a key left out
takes the Defaults Profile value; a required input left out is an error), and the Option is run the way
`ovd run` runs it. The result is `run`'s results. The calculation runs in the Rust core
(`ovd._native`); this module only passes the parameters through. Pure calculation, no I/O.
"""
from __future__ import annotations

from ._bridge import call as _call, call_with_progress as _call_with_progress

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


def simulate_damage(params: dict, progress=None, cancel=None) -> dict:
    """Run the damage stage: the same Option as ``simulate(params)``, with the damage cases worked out.

    The result equals ``simulate(params, damage=True)``, and ``damage_status`` is ``"run"``. The call is
    stateless: it builds the Option again, so the part that ``simulate(params)`` does runs again.

    ``progress(done, total)`` is called as cases finish: first with ``done`` 0, last with ``done == total``.
    ``cancel()`` returning true stops the run, and the call *returns* the results of a run without damage
    (``damage_status`` is ``"not run"``). Both are called on the thread that called ``simulate_damage``
    (the engine's worker threads never call Python), one at a time; keep them short. A ``threading.Event``
    is passed as ``event.is_set``. An exception raised by either, or Ctrl-C (``KeyboardInterrupt``),
    stops the run and is raised from this call. A hull with no damage Rule Set gives
    ``"not applicable"`` and never calls ``progress``.
    """
    for name, f in (("progress", progress), ("cancel", cancel)):
        if f is not None and not callable(f):
            raise TypeError(f"`{name}` must be a callable or None, not {type(f).__name__}")
    return _call_with_progress("simulate_damage", {"params": params}, progress, cancel)
