# Open Vessel Designer (OVD)

An open instrument for the concept and preliminary design of inland and coastal vessels and
tug-barge systems. Give it a hull and a cargo; it gives you capacity, draught, weights, tank
arrangement, stability, resistance and power, and general-arrangement drawings — each result with
its checks, checked against real vessels.

**Website and docs:** [africanrivercorridors.com/ovd](https://www.africanrivercorridors.com/ovd) ·
**Roadmap:** [ROADMAP.md](ROADMAP.md) · **Decisions:** [architecture decision records](docs/adr/0011-scope-concept-and-preliminary-design-never-approval.md)

## Ways to use it

- **Web app**: [open it in your browser](https://www.africanrivercorridors.com/ovd/app), nothing to
  install. Live results as you type, drawings to the Drawing Format Standard (SVG, PDF and DXF),
  stability curves, rule checks and a capital budget. The Rust core runs in your browser as
  WebAssembly, so your designs stay on your computer. See [Run the web app](#run-the-web-app).
- **Command line** (`ovd`): create, check, run and draw `.vdesign` files.
- **Python library** (`ovd`): the calculation core from Python.

All of them use one compiled Rust calculation core (`ovd-core`: pure calculation, no I/O), so they
give the same numbers.

## What it is, and what it is not

**It is** a concept and preliminary design tool for inland and coastal vessels and tug-barge
systems; a calculation core you can read, where results name their checks and sources; calibrated
against real barges and published design points, with the cases in the open; free, with open
formats and documented methods.

**It is not** class approval, a stability booklet or construction approval, and it never will be
([ADR 0011](docs/adr/0011-scope-concept-and-preliminary-design-never-approval.md)). It is not a
substitute for a naval architect, a yard design office or a class surveyor. It is not yet verified:
known engineering findings are under review in the open
([#121](https://github.com/African-River-Corridors/open-vessel-designer/issues/121)).

## What it does

- **Standard vessel classes.** A catalogue of standard inland classes (CEMT, the Dutch 2011 update,
  Chinese GB 38030, US and Paraná sizes) in `catalogue/`, and a typed lookup of the common ones in
  `ovd.core.vessel_standards`.
- **Capacity.** Volume-bound cargo for dry-bulk holds (sloped hopper, box, container) and for
  liquefied-gas barges with IMO Type C tanks (`core.barge_capacity`).
- **Gas tank arrangement.** Protective-location clearances under IGC type 2G (the distance d(Vc) of
  IGC 2.4.1.1 per tank, dome included in Vc, measured plate to plate per 2.4.2), a tank outer diameter cap (an input, 16 m by default) and tank length at most 7
  diameters, and a search over depth and tanks abreast for the most cargo at a draught limit
  (`core.gas_arrangement`). Only IGC rules apply to gas tanks.
- **Hydrostatics and stability.** Equilibrium draught, freeboard, air draught, and intact stability
  for gas barges (`core.hydrostatics`, `core.gas_stability`). The hull is a table of sections: the
  raked bow and the stern form, with a bilge radius, sheer and camber when set. The draught, trim, GM and
  GZ come from these sections, and the block coefficient is a result, not an input. The weights and
  the resistance wetted surface still use a box hull. Known limitation: the 0 to 40 degree and
  30 to 40 degree areas of IS Code 2008 Part A 2.2.1 run to 40 degrees unless you type an angle of
  down-flooding below 40 degrees (the input `downflooding_angle_deg`). The text stops them at that angle
  when it is less. List the deck openings with their heights (the input `downflooding_openings`) and the
  model works the angle out for each loading condition, from the hull's sections at the free-trim
  equilibrium; a typed angle wins. With neither typed nor listed, a hull whose openings immerse before
  40 degrees can pass when it should fail
  ([#282](https://github.com/African-River-Corridors/open-vessel-designer/issues/282),
  [#592](https://github.com/African-River-Corridors/open-vessel-designer/issues/592)).
- **Journey de-rating.** Cargo per trip when the channel, not the hull, limits the draught
  (`core.journey_derating`).
- **Resistance, squat, waves and power** in deep and confined shallow water (`core.resistance`,
  `core.squat`, `core.waves`, `core.electric_energy`).
- **Calibration** against real barges and published design points (`calibration/`).

## Install

Python 3.12 or later.

```
pip install git+https://github.com/African-River-Corridors/open-vessel-designer
```

## Run the web app

The web app is the app: a concept design tool with live results and drawings. It needs no Python and
nothing to install. Open [africanrivercorridors.com/ovd/app](https://www.africanrivercorridors.com/ovd/app).
The first start shows a short notice: results are not verified and there is no warranty.

**The desktop app is paused.** OVD ships the web app and the command line only, until the design and
the engine are more stable (ADR 0026, `docs/adr/0026-web-and-command-line-only-english-only-for-now.md`).
A release holds the `ovd` command line for Windows and macOS and the Python wheels, on the
[releases page](https://github.com/African-River-Corridors/open-vessel-designer/releases).

**Build from source.** You need Git, Node 24 and Rust (install with [rustup](https://rustup.rs)), and
the `wasm-bindgen` command line at the version `crates/ovd-web/Cargo.toml` pins. Then:

```
git clone https://github.com/African-River-Corridors/open-vessel-designer
cd open-vessel-designer
bash tools/build_web.sh   # the WebAssembly session, into app/wasm-pkg
cd app
npm ci
npm run dev               # run the web app
npm run build             # tests, type check and the web build (app/dist-web)
```

## Worked example

To run one Option from a Python dict, the way the app and `ovd run` do, see the [Python API](docs/python-api.md).

Size a liquefied-propane barge on a standard Chinese Class II hull (`examples/quickstart.py`):

```python
from ovd.core.barge_capacity import BargeCapacityInputs, compute
from ovd.core.gas_arrangement import fit
from ovd.core.vessel_standards import get_standard

cls = get_standard("CN_II_motor")
loa, beam = cls.loa_m[1], cls.beam_m

# One capacity run: the core picks a tank arrangement for this hull.
auto = compute(BargeCapacityInputs(loa=loa, beam=beam, depth=6.5, cargo_type="LPG_propane"))

# Search the arrangement for the most cargo per journey at a 4.4 m draught cap.
best = fit(loa, beam, "LPG_propane", 4.4, ruleset="IGC_2G", depths=(6.5,))
```

Output:

```
Class CN-GB50139 CN-II-MOTOR: 90 m x 14.8 m, 2,000 t class deadweight
Single run : 1 tanks abreast, OD 13.70 m, 3,278 t, laden draught 4.29 m
Searched   : 2 tank(s) abreast, OD 6.45 m, 1,406 t (volume-bound), draught 2.57 m
```

The single run fills the tank volume and does not look at the draught cap or the checks. The search
respects the cap and keeps only arrangements that pass every blocking Rule, stability and the
wheelhouse sightline included. The larger tanks are too tender when laden, hide the view from the
raised wheelhouse, or (one 10.70 m tank) fail the light-condition angle of maximum GZ, so the search
carries 1,406 t in two 6.45 m tanks abreast. If no arrangement passed, `feasible` would be false and
`failed_checks` would list the checks the closest one fails. Tank side clearance follows IGC Code
2.4.1.1 (the distance d(Vc), per tank, dome included in Vc), measured plate to plate with the
insulation inside d (2.4.2). A check that is only advisory, a warning, does not stop an arrangement
passing; it stays in the results as a failed check.

## Layout

| Path | Holds |
|---|---|
| `src/ovd/core/` | the calculation core |
| `src/ovd/simulate.py` | one-call JSON-in, JSON-out simulation |
| `catalogue/` | standard vessel classes and reference vessels, with sources |
| `calibration/` | calibration cases, fitted factors and results |
| `crates/` | the Rust core (`ovd-core`), file format and Design session (`ovd-format`), and CLI. Drawings go through [`otd`](https://github.com/African-River-Corridors/open-technical-drawer), pinned by tag |
| `app/` | the web app (Svelte front end over the WebAssembly session) |
| `website/` | the website and docs ([africanrivercorridors.com/ovd](https://www.africanrivercorridors.com/ovd)) |
| `docs/adr/` | architecture decisions |
| `GLOSSARY.md` | the domain language |

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[roadmap](ROADMAP.md). Please follow the [code of conduct](CODE_OF_CONDUCT.md). Report security
issues privately — see [SECURITY.md](SECURITY.md).

## Licence

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
Copyright 2026 African River Corridors Limited.
