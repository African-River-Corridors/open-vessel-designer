# Tolerance for the holding-time reference cases

Set before any comparison of the OVD holding-time model with `neqsim_cases.json` (issue #868, spec 0016
sections 4 and 7, decision D6). The OVD model does not exist yet (#870), so no comparison has been run.

## The rule

The tolerance on a compared quantity is the **larger** of:

1. the spread between NeqSim's equation of state and the NIST table (`catalogue/methane_saturation/nist_methane.tsv`)
   at the same states, and
2. OVD's own step error: the change in the result when OVD's step is halved, measured by H4 (#870).

It is **never loosened without the maintainer**. A larger measured value is a finding to report, not a
reason to edit this file.

## Item 1, measured now

Equation of state: GERG-2008 in NeqSim 3.27.0. States: the table rows from -162 C to -134 C, which cover every
state the four cases visit. NeqSim's energy zero differs from the table's, so one offset is fixed at the
saturated liquid at -160 C (910.80 kJ/kg) and applied to all internal energies and enthalpies.

Largest difference, NeqSim against the table:

| Quantity | At the table rows | Half way between rows (NIST value is the straight line) |
|---|---|---|
| Saturation pressure | 0.0278 % | 0.0401 % |
| Liquid density | 0.0144 % | 0.0133 % |
| Vapour density | 0.0259 % | 0.0736 % |
| Liquid internal energy | 0.025 kJ/kg | 0.024 kJ/kg |
| Vapour internal energy | 0.135 kJ/kg | 0.134 kJ/kg |
| Liquid enthalpy | 0.025 kJ/kg | 0.023 kJ/kg |
| Vapour enthalpy | 0.157 kJ/kg | 0.157 kJ/kg |

The second column is the one that matters for OVD, which reads the table by straight lines between rows: it
holds the equation-of-state spread and the interpolation error together.

The same stepping run on the table (straight line between rows, 60 s step) against the NeqSim run, for the
four cases:

- Holding time: 0.013 to 0.034 % (largest 0.0340 %, cases A and C).
- Pressure at the stored hourly times: up to 0.0632 % (case B).

## Item 2, how H4 (#870) will measure it

For each case, H4 runs the OVD model at its step and at half of it, and takes the percentage change in holding
time and the largest change in pressure at the stored times. H4 states the measured value in its PR, beside the
values above. Nothing is set for it here.

The reference itself is converged in step: halving 120 s to 60 s moves no holding time at the stored digits
(cases A to C have a constant heat leak, so their energy balance is exact in time; case D differs by under
5e-7 h).

## Current value

Holding time: the larger of **0.0340 %** and the step error H4 measures. Pressure at the stored times: the larger
of **0.0632 %** and the step error H4 measures. These floors move only with the maintainer's approval.

## Item 2, measured by H4 (#870)

OVD steps at 600 s with a second-order (predictor-corrector) step. Halving it to 300 s changes the holding time
by 0 % in cases A to C (constant heat leak) and 0.0070 % in case D, and the pressure at the stored times by 0 %
and 0.0195 %. Both are below the floors above, so the floors stand.

OVD against NeqSim at 600 s: holding time within 0.018 % in every case, inside its floor of 0.0340 % (about
30 times inside it in cases A to C). **Holding time is gated** against that floor. Pressure at the stored times
is 0.077 to 0.110 %, above its floor of 0.0632 %: **reported, not gated** (maintainer, 2026-11-05; see issue
#TBD-pressure-gap). The floor is unchanged.

holding-time-tolerance-percent: 0.0340
pressure-tolerance-percent: 0.0632
