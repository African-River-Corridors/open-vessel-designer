# Spec 0016: Type C pressure holding time, and the design pressure and insulation it sets

Status: **agreed with the maintainer, 2026-11-02.** Round 1: D1–D6 (D1, D3 and D6 amended after review and re-confirmed). Round 2: D7–D15, all as recommended except D7 (the passage gas use from the powering model is credited). Confirm round (2026-11-02): the D7 credit basis, no credit without a gas consumer, and D14 on IGC 15.1.3.1 always. Issue: #823.
Related: #317 (tank checks), #793 (boil-off destination on an ATB), spec 0007 (powering, gas fuel), spec 0015
(report).

Concept and preliminary design only, never approval (ADR 0011).

## 1. Problem

An external review of the 140 × 29 m gas example asked for the cargo tank pressure holding time to be checked
against the operation profile, because the holding time is what sets the tank design pressure and the
insulation thickness.

OVD cannot do this today:

1. **No pressure model.** `crates/ovd-core/src/gas_fuel.rs` computes the heat leak through the insulation at
   the IGC 7.2 upper ambient air temperature (45 °C) and divides it by the latent heat to give a boil-off rate.
   Nothing follows the tank pressure when the boil-off is not taken away.
2. **No operation profile.** There is no input for how long the tank holds its cargo or how much boil-off is
   surely used.
3. **Defaults stand in for design.** The design pressure (0.4 MPa on the example, or the IGC 4.23.1.2 minimum
   when `design_pressure_igc_minimum` is set) and the insulation thickness (330 mm) are defaults. Neither is
   derived from what the vessel must do.
4. **The surplus is only flagged.** The check `boil-off within gas use` warns that boil-off exceeds gas use;
   it does not say for how long the tank can hold it.
5. **Hidden from the user.** The boil-off rate, the gas use and the insulation assumed are not shown together
   anywhere in the app.

## 2. Rules read

Text read from the rule-source library. Pressures in the IGC Code are gauge: MARVS
(1.2.34) and P0 (4.1.2).

- **IGC Code (MSC.370(93)) 7.1.1**: tank pressure and temperature shall be maintained within the design range
  by reliquefaction, thermal oxidation, pressure accumulation or liquid cargo cooling. **7.1.3**: venting to
  maintain pressure is not acceptable except in emergency. **7.2**: upper ambient design temperatures, sea
  32 °C, air 45 °C. **7.5** (pressure accumulation): "The containment system insulation, design pressure or
  both shall be adequate to provide for a suitable margin for the operating time and temperatures involved."
  The Code sets **no fixed holding time** for cargo tanks.
- **IGC 4.13.2.1**: P0 is not less than the MARVS; on a Type C tank the relief setting is taken as the design
  pressure unless typed lower.
- **IGC 4.23.1.2**: the Type C minimum design vapour pressure: a floor on design pressure, independent of
  holding time.
- **IGC 15.1.1, 15.1.3.1, 15.5**: the filling limit (98 %) applies at the **reference temperature**; with
  pressure accumulation only, that is the saturation temperature at the relief setting. The **loading limit**
  is LL = FL · ρR / ρL. A tank loaded to LL is at FL when it reaches the relief setting, so it cannot go
  liquid-full first.
- **IGC 16.1 and 16.2**: methane (LNG) is the cargo whose vapour or boil-off may be used as fuel, under the
  IGC Code's own use-of-cargo-as-fuel chapter. A cargo tank whose boil-off fuels the vessel is therefore treated
  under IGC; IGF 6.9.1.1 is applied only to separate fuel tanks. (The IGF scope paragraph was not read.)
- **IGC 15.1.3**: the reference temperature is .1 the saturation temperature at the PRV set pressure when no
  chapter 7 pressure/temperature control is provided, or .2 the greatest cargo temperature at the end of
  loading, during transport or at unloading when one is provided (see D14).
- **IGF Code (MSC.391(95)) 6.9.1.1**: for liquefied gas **fuel** tanks, the method "shall be capable of
  maintaining tank pressure below the set pressure of the tank pressure relief valves for a period of 15 days
  assuming full tank at normal service pressure and the ship in idle condition, i.e. only power for domestic
  load is generated." Separate fuel tanks only.
- **ADN 2025 7.2.4.16.17** (inland, refrigerated liquefied gases): the holding time is determined from the heat
  transmission coefficient, the safety-valve set pressure, the initial filling conditions, the ambient
  temperatures of 9.3.1.24.2 and, when boil-off is used, "the minimum guaranteed use of the boil-off vapours
  (that is the amount of boil-off vapours used under any service conditions)". It "is at least three times the
  expected duration of the journey", with a minimum of 15 days for journeys of up to 5 days, and of 30 days
  plus two days per day beyond 10 for journeys over 10 days. **ADN 9.3.1.24.2** sets the inland ambient
  temperatures (air +30 °C, water +20 °C).
- **ABS Rules for Building and Classing Steel Barges, 5-2-5 item 27** (liquefied gas tank barges): "Holding
  time calculations are to be submitted."
- **BV NR467 Pt D Ch 9 Sec 7 [5.1.1]** repeats IGC 7.5, then Note 1: the operating time is "not less than
  21 days". Scope not confirmed.
- **BV NR217 Pt D Ch 3 Sec 2 [5.2.1]** (inland, refrigerated cargo after failure of refrigeration): at least
  52 hours without the safety valves opening. Scope not confirmed.

## 3. Decisions (design tree)

Round 1 (2026-11-02), as answered. Round 2 is open; recommendations in bold.

| # | Decision | Options | Decision and why |
|---|---|---|---|
| D1 | Which tanks | (a) cargo tanks and separate LNG fuel tanks, each against its own rule; (b) cargo only; (c) fuel only | **(a).** Cargo tanks (including those whose boil-off fuels the vessel, IGC 16) against the design basis and ADN where it applies; separate fuel tanks against IGF 6.9.1.1 as well |
| D2 | Required holding time, cargo tanks | (a) from the typed profile only; (b) a default, overridable; (c) always typed | **(b): a default design basis of 30 days, 10 of them idle**, overridable. IGC 7.5 sets no figure, so the default is printed as "design basis (default)", never as compliance |
| D3 | Profile shape | (a) legs; (b) one idle period | **The 30-day default with 10 days idle**, editable. The leg states are fixed by D9. With D7, a passage leg credits the gas the engines burn and an idle leg credits none, so the split matters: 20 days of passage, then 10 idle: the idle block last is the worst case, as the tank is warmest when the gas use stops |
| D4 | Pressure model | (a) homogeneous; (b) homogeneous with a typed stratification margin; (c) stratified | **(a).** See section 4 for the balance. The app and report state that the model ignores thermal stratification, which can make the real pressure rise faster |
| D5 | Solve mode | (a) check, then solve later; (b) check only; (c) solve from the start | **(c).** The cases are in D11 |
| D6 | Test oracle | (a) vendor calculation plus NeqSim; (b) NeqSim only; (c) papers | **(b)**, plus the hand case and the published-case search of section 7 |
| D7 | Gas-use credit | (a) only a minimum guaranteed use, typed with its source, default zero; (b) the passage gas use from the powering model | **(b), the maintainer's decision.** For the design basis row, a passage leg credits the **calm-water** gas use the powering model gives at a **typed passage speed** and the **lightest draught in the leg** (its end), the conservative choice; an idle leg credits zero unless a use is typed. **No passage credit** unless the Option has a gas consumer that takes the boil-off; it is zero for a barge with none until #793 settles the ATB case. Rules that limit the credit keep their own: the ADN row credits only a typed minimum guaranteed use (default zero, as 7.2.4.16.17 reads), and the IGF row credits domestic load only (typed, default zero). The app and report show which credit each row used. This brings the loop through draught and resistance back into the solve (D12) |
| D8 | Which rules apply by Service Area | (a) the design basis always; ADN 7.2.4.16.17 as well when the Service Area includes inland waterways (required time = the larger, at ADN ambient for the ADN check); IGF 15 days for separate fuel tanks; BV NR467 and NR217 as named presets, never applied by default; (b) the design basis only | **(a)**. The journey duration for ADN is a typed input; with none typed, the ADN check is "not evaluated" |
| D9 | Leg states | (a) laden passage and idle only; the tank is closed and starts at the loading limit; loading, discharge and ballast are out of scope; (b) all five states with mass changes | **(a)**: a closed tank that starts full has no meaning for loading or discharge legs |
| D10 | Start state | (a) cargo tanks: loaded to LL at the saturation state of 0 MPa g (atmospheric), typed override; separate fuel tanks: the typed normal service pressure that IGF 6.9.1.1 names, and with none typed the IGF row is "not evaluated"; (b) a typed storage temperature for both | **(a)**: no storage-temperature input exists; atmospheric loading is the usual LNG cargo case; IGF starts from "normal service pressure", which is usually above atmospheric, so it must be typed |
| D11 | Solve cases | (a) pressure untyped, insulation typed or default: solve the lowest design pressure; pressure typed, insulation untyped: solve the thinnest insulation; both typed: check only; `design_pressure_igc_minimum = true` keeps today's meaning (the design pressure is the IGC minimum) and counts as typed, so the holding time is checked, not solved; (b) solve pressure only | **(a).** Every example without typed values moves (section 5) |
| D12 | Solve numerics | (a) search in steps of 0.005 MPa (pressure) and 10 mm (insulation) between the IGC 4.23.1.2 minimum and a typed upper bound (default 1.0 MPa g; insulation 50 to 600 mm); iterate the wall, the steel radius in the heat leak, LL, and the tank mass, draught and passage gas use (D7) to a fixed point: stop when the solved value repeats within one step; if it flips between two neighbouring steps, take the higher and confirm it meets the requirement after a last pass; fail as "not converged" after 20 iterations; no solution in range is "infeasible", a fail row, and the typed or default values are kept | **(a)** |
| D13 | Heat leak through time | (a) constant at the start-state cargo temperature; (b) recomputed each step as the cargo warms | **(b)**: the leak falls as the cargo warms; (a) is simpler and slightly conservative. Recommend (b) because it costs little in the stepping. Ambient by rule: IGC 7.2 (air 45 °C, sea 32 °C) for the design basis and IGF; ADN 9.3.1.24.2 (air 30 °C, water 20 °C) for the ADN row |
| D14 | Reference temperature for LL | (a) IGC 15.1.3.1 (saturation at the PRV setting) when no use is credited; 15.1.3.2 (the greatest cargo temperature in the profile) when a use is credited, as that use is a chapter 7 control; (b) 15.1.3.1 always | **(b), confirmed.** IGC 7.1.1 lists reliquefaction, thermal oxidation, pressure accumulation and liquid cooling; burning gas in the engines (chapter 16) is not tied back to chapter 7, so 15.1.3.2 is not used. LL is always set at the saturation temperature at the relief setting |
| D15 | Published worked case | (a) the model ticket stays open until a published LNG Type C pressure-time case with its heat leak is found; (b) the maintainer accepts the hand case and the NeqSim cases in its place | **(b)** with a standing ticket to add a published case when one is found: no such case was found in the first search, and (a) could block the work indefinitely |

## 4. Model

**State.** A closed tank of geometric volume V holding a mass m of methane in one saturation state
(temperature T, quality x). Pressures are converted between gauge (the rules) and absolute (the saturation
table) at 0.101325 MPa.

**Step.** Over a time step Δt with heat leak Q and a withdrawal ṁ (the gas use credited for the leg and the row, D7):

- U(t + Δt) = U(t) + Q · Δt − ṁ · h_out · Δt
- m(t + Δt) = m(t) − ṁ · Δt
- h_out is the saturated vapour enthalpy when vapour is burned, or the saturated liquid enthalpy when liquid
  is drawn to a forced vaporiser (an input with the gas use; default vapour).
- The new state is the (T, x) whose specific internal energy is U/m and whose specific volume is V/m, solved
  together on the saturation table.
- When the typed use exceeds the vapour available in a step, vapour withdrawal is capped at what is available
  and the result says so. Withdrawal also stops when the pressure falls to the start pressure (a floor), so the
  tank is never drawn below atmospheric.

**Holding time.** The time from the start state to the relief setting. The loading limit LL = FL · ρR / ρL is
computed at the solved or typed relief setting, so the tank is at FL when the relief setting is reached.
Liquid-full is a guard: a typed fill above LL is refused with a message.

**Properties.** The methane saturation table already in the library (`type-c/cargo/NIST_saturation_methane.tsv`)
covers −170 to −100 °C, about 2.6 MPa absolute. The code reads pressure, densities, internal energies and
enthalpies from it; the constant in `gas_fuel.rs` that covers only −165 to −160 °C is replaced by the table.
Outside the table the functions return an error.

## 5. Effects on the engine

- New `no_std` functions for the saturation properties (section 4) and for the stepping and holding time.
- The solve (D11, D12) runs before the tank wall, tank mass, heat leak and boil-off, which then follow the
  solved values.
- New Rule rows, `unverified` until a second reader checks them:
  - `igf/6.9.1.1/fuel-tank-holding-time` (separate fuel tanks, 15 days, idle, domestic load only);
  - `adn/7.2.4.16.17/holding-time` (inland Service Areas, three times the typed journey, with the minimums);
  - `ovd/holding-time/design-basis`: the profile's holding time against the design basis. It prints
    "design basis (default)" or "design basis (typed)", never "complies"; IGC 7.5 sets no figure.
- **Design impact.** On every Option that solves (every example without typed values): the design pressure,
  the tank wall, the tank mass, the loading limit and so the **cargo capacity and deadweight**, the lightship,
  the draught and the cost move. Stored results of files that type both the design pressure and the insulation
  must not move.

## 6. App and report

- **App, Stability & Propulsion group, gas vessels only**: a new **Gas** view. It shows together: the
  insulation assumed (type, thickness, conductivity, each with its source), the heat leak, the boil-off rate
  (kg/h and % of cargo per day), the gas use by environment, the minimum guaranteed use credited, the balance,
  the profile, the pressure against time with the relief setting and the holding time, and the design
  pressure with its source (typed, default or solved).
- **Report (spec 0015)**: the gas module (#835) gains a holding-time subsection in A3 and A6 with the same
  content; the stratification limit goes to A14.

## 7. Testing

- **Properties.** Values match the table at its rows; a test fails outside the table.
- **Hand case.** One property-window case computed by hand from the table rows (start and end states at two
  table rows, no withdrawal), run in a constant-heat-leak test mode: the holding time equals
  (U_end − U_start) / Q to the table's precision. Checks the state solve, not only the stepping.
- **NeqSim reference cases** (D6). At least three Type C methane cases (two fills, two heat leaks, one with a
  withdrawal), produced by a script in `tools/` that pins the NeqSim version and equation of state; the stored
  output carries its hash. **The tolerance is set before the first comparison**, as the larger of the spread between
  NeqSim's equation of state and the NIST table at the same states and OVD's own step and interpolation error
  (measured by halving the step), and is never loosened without the maintainer. These cases check properties and stepping only; they share the homogeneous assumption and the
  heat leak.
- **Published case.** The issue asks for a published method with a worked example. None was found that gives
  an LNG Type C pressure-time curve with its heat leak. D15 decides what happens.
- **Monotonic.** More insulation never shortens the holding time; a higher design pressure never shortens it.
- **Solve.** The solved value meets the requirement and the next step down does not; never below the IGC
  4.23.1.2 minimum; out of range gives "infeasible".
- **Convergence.** A case built to flip between two steps returns the higher and meets the requirement; a case
  forced past 20 iterations returns "not converged" and its row fails.
- **Loading limit.** With no withdrawal, a tank loaded to LL is within 0.1 percentage point of FL at the relief
  setting; with withdrawal, at or below FL. A typed fill above LL is refused.
- **Profiles.** A profile with an unknown state or a negative duration or use is refused with a message.
- **Withdrawal.** A use above the vapour available is capped, and the result says so; the pressure never falls
  below the start pressure.

## 8. Phasing and tickets (filed after this spec merges)

1. **Saturation properties and pressure basis**: the table functions (P, ρ, u, h by T; gauge and absolute).
   Accept: section 7 "Properties"; outside the table the functions return an error.
2. **NeqSim reference cases**: the script, the tolerance from the equation-of-state spread, the stored cases.
   Blocked by 1 (the spread is measured against the table).
3. **Operation profile input**: the 30/10 design basis, leg states (D9), the passage speed per leg and the
   link to the powering model's gas use (D7), the typed minimum guaranteed use for the ADN row and the domestic
   load for the IGF row, the journey duration for ADN, the presets (D8), the `.vdesign` fields with their sources. Accept: old files load
   with the default, labelled default; bad profiles are refused (section 7).
4. **Holding-time model**: the stepping, the withdrawal cap and floor, the heat leak by step and ambient by
   rule (D13; ambient becomes an input to `tank_heat_leak_kw`), the loading limit at the reference temperature
   (D14), the liquid-full guard, the per-leg result. Accept: the hand case, the NeqSim cases within the set
   tolerance, monotonic, loading limit, withdrawal. Blocked by 1, 2, 3.
5. **Solve and Rule rows**: D11, D12 with the fixed point through tank mass, draught and passage gas use, the
   three Rule rows, design impact with the changelog fragment. Blocked by 3 (the powering link) and 4.
6. **App Gas view** (section 6). Blocked by 4, 5.
7. **Report subsection** (section 6). Blocked by 5 and #835.

## 9. Not in scope

- Stratified or non-equilibrium models (D4).
- Real LNG composition: methane properties stand in; the app and report say so.
- Loading, discharge, ballast legs and heel management (D9).
- Reliquefaction, thermal oxidation and gas combustion unit sizing.
- Tank types other than Type C.
