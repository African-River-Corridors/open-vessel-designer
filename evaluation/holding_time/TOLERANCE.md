# Tolerance for the holding-time reference cases

Set before any comparison of the OVD holding-time model with `neqsim_cases.json` (issue #868, spec 0016
sections 4 and 7, decision D6). The OVD model does not exist yet (#870), so no comparison has been run.

## The rule

The tolerance on a compared quantity is the **larger** of:

1. the spread between NeqSim's equation of state and the NIST table (`catalogue/methane_saturation/nist_methane.tsv`)
   at the same states, and
2. OVD's own step and interpolation error: the change in the result when the step is halved.

It is **never loosened without the maintainer**. A new, larger measured value is a finding to report, not a
reason to edit this file.

## What was measured now (item 1)

Equation of state: GERG-2008 in NeqSim 3.27.0. States: the table rows from -162 C to -133 C, which cover every
state the four cases visit. NeqSim's energy zero differs from the table's, so one offset is fixed at the
saturated liquid at -160 C (910.80 kJ/kg) and applied to all internal energies and enthalpies.

| Quantity | Largest difference |
|---|---|
| Saturation pressure | 0.0278 % |
| Liquid density | 0.0148 % |
| Vapour density | 0.0259 % |
| Liquid internal energy | 0.025 kJ/kg |
| Vapour internal energy | 0.135 kJ/kg |
| Liquid enthalpy | 0.027 kJ/kg |
| Vapour enthalpy | 0.157 kJ/kg |
| Holding time, same stepping on the table (straight line between rows) against NeqSim | 0.0150 % (cases A to D: 0.0116 to 0.0150 %) |

The holding-time row is the figure for the compared output. The cases share the homogeneous assumption and the
heat leak with the model under test, so this spread comes from properties and stepping only.

## Item 2: how H4 (#870) will measure OVD's step error

For each case, H4 runs the OVD model at the case step (3600 s) and at half of it (1800 s), and takes the
percentage change in holding time, and the largest change in pressure at the stored trajectory times. The
interpolation error is inside that figure, since OVD reads the table by straight lines between rows. H4 states
the measured value in its PR, beside this file's current floor.

The stored reference carries its own step effect: halving the step in the NeqSim run moves the holding time by up
to 0.0437 % (case D, the withdrawal case). That is part of the reference value, so the tolerance is not set
below it.

## Current value

Holding time: **the larger of 0.0437 % and the step error H4 measures**. Pressure on the stored trajectory:
the larger of 0.0278 % and the step error H4 measures. Both are floors. H4 raises them only by stating the
measurement here, in a PR the maintainer approves.
