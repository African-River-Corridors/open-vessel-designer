# Spec 0017: A setup wizard and a guided workspace for a first Design

Status: draft, built on the maintainer's instruction of 2026-11-19. The decisions in section 3 were made
without a decision round; the maintainer reviews them after the build.
Related: spec 0002 (first desktop app), issue #924 (damage stability is a manual run), issue #925 (the
Service Area is required and has no default), issue #1034 (a river-only Option needs both river limits),
`docs/design/style-guide.md`.

Concept and preliminary design only, never approval (ADR 0011).

## 1. Problem

A vessel designer who opens the app for the first time wants one thing: a first Design of their own vessel,
with an answer they can read. We walked that path on main with a 140 × 29 m coastal LNG barge. This is what
the designer meets.

1. **A vessel they did not ask for.** The app opens on a starter Design: a 76.5 × 11.4 m pushed barge that
   carries bauxite. It shows an error and "No results to draw", because the starter has no Service Area.
2. **The six required inputs are in three places.** Cargo, Formation, length and beam are in the Required
   group. Hull Form is in its own group. The Service Area is in the Rules group, near the bottom of about
   160 inputs.
3. **A leftover from the starter spoils the first answer.** The starter types a moulded depth of 3.5 m. The
   designer changes the cargo and the size, the 3.5 m stays, and the first result is a 140 m hull 3.5 m
   deep with a slenderness warning.
4. **Words for another tool.** The Service Area error tells the designer to run a command-line program.
5. **No sign of work.** The first result of a gas barge takes several seconds. Nothing says "calculating".
6. **No headline.** The answer is spread over tables, tabs and a compare grid. Nothing says, in one place,
   "this barge carries 18,633 m³ and meets every Rule that was checked".
7. **One step is easy to miss.** Damage stability changes the answer of a gas barge (the tank size, and so
   the cargo), but it runs only when the designer presses a button in a message line.
8. **All 160 inputs look equally important.** The designer cannot tell the ten that matter from the 150 that
   the Defaults Profile fills well enough. Help text is only a tooltip.

## 2. Aim

A designer who has never seen the app makes a first Design of their own vessel in under five minutes, with
no manual, and can say what the answer is and what to change next.

The worked example is the barge of the reviewer guide: LNG, self-propelled, raked box, 140.0 × 29.0 m,
Service Area coastal and river, then moulded depth 12.0 m, maximum speed 12 kn, largest tank diameter
12.0 m, two tanks along, minimum freeboard 1.81 m. That is 11 typed inputs.

Nothing in this spec changes a calculation, a Rule, a default or the file format. Every number the new
screens show already exists in the evaluation. A Design made by the wizard is the same Design, input for
input and source for source, as the one `ovd fill --as-new` makes from a short file with the same typed
inputs.

## 3. Decisions

- **D1. The app starts with a Start page, after the first-run notice.** The first-run notice keeps its
  words, its button and its behaviour (style guide). After it, when the app was not started with a file to
  open and has no Design to restore, the Start page opens. It has three choices and one link:
  - **New Design, step by step** (the wizard; the first and largest choice).
  - **Start from an example** (D6).
  - **Open a Design file** (the existing Open).
  - A link, **"Skip: open the blank starter Design"**, which gives exactly what the app gives today.
  The **New** button and its shortcut open the same Start page. The existing question before unsaved work is
  discarded stays, and it is asked before the Start page replaces the Design, not before the page opens:
  cancelling the Start page leaves the current Design untouched.
- **D2. The wizard asks only for what has no default, in the order a designer thinks.** Five short steps and
  a review. The choices of every step come from the input schema (`input_schema`), never from a list in the
  app, so a new Cargo or Formation appears with no change to the wizard. The labels are the schema's labels.
  1. **Cargo.** One choice per Cargo. A Cargo in development keeps its "(work in development)" label.
  2. **Service Area.** River only, or coastal and river, each with one sentence that says what it brings
     (which families of Rules are checked). For river only the step also asks for the draught limit and the
     air-draft limit, because a river-only Option needs both numbers (#1034), with the note that a Vessel
     Class chosen in step 4 can give them instead.
  3. **Formation.** One choice per Formation.
  4. **Hull.** How the vessel is defined (typed dimensions, or a standard Vessel Class from a Class Scheme,
     with the same pickers as the Input panel), then the Hull Form, the length overall and the beam. With a
     Vessel Class the length, beam and Formation come from the class, as they do today.
  5. **Your own limits (optional).** "Leave any of these empty and the tool chooses." Moulded depth and
     maximum speed for every Design. For a Cargo carried in tanks: the largest tank diameter and the tanks
     along the hull. For coastal and river: the minimum freeboard, with a note that the default is the inland
     safety clearance and that the load-line table value appears in the Rule checks after the first result.
     The step shows only inputs that apply to the Cargo, the Formation and the Service Area chosen.
  6. **Review.** The typed choices in one list, the count of inputs left to the Defaults Profile, the Option
     name (made from the Cargo and the size, for example "LNG 140x29"; the designer may edit it), and
     **Create Design**.
  Back and Next keep every answer. Each step can be reached again from the review.
- **D3. The wizard checks as the designer types, with the schema's own limits.** A value outside a hard
  limit blocks Next and says the limit. A value outside a usual range shows the same warning the Input panel
  shows, and does not block. The wizard does not copy a limit into the app.
- **D4. Create makes the Design in one step.** One new session call takes the typed choices and makes a
  Design with one Option that holds them, marked typed, and every other input from the Defaults Profile,
  marked default. It never goes through the starter Design, so no example value of the starter (its moulded
  depth of 3.5 m, its Cargo) can stay behind. It is the same on the desktop and on the web. The equality
  with `ovd fill --as-new` (section 2) is its test.
- **D5. After Create the app shows the answer, and runs damage stability by itself once.** The app
  evaluates, opens the Summary view (D7), and, when the Design is one for which the app today says "Damage
  stability has not been run", starts that run with the existing progress line and Cancel. This is the only
  automatic damage run: after it, #924 stands and damage stability runs when the designer asks. Cancel
  leaves the unscreened answer, marked as it is today.
- **D6. One example ships with the app: the coastal LNG barge of the reviewer guide.** "Start from an
  example" lists it with one line of text. It opens the wizard at the review step with the 11 typed inputs
  of section 2 filled in, so the designer sees what an example is made of and can change any step before
  Create. The example is data in one place in the core, behind the same session call as D4, so the desktop
  and the web cannot differ. More examples can be added later as data.
- **D7. A Summary view is the first view.** It answers three questions with numbers that already exist.
  - **Does it work?** One verdict line: every evaluated blocking Rule is met; or the Rules that are not, by
    title; or "not calculated", with the input to fix and a link to it. Then the scope sentence the outputs
    already carry (which Rule Sets were checked, how many blocking Rules were not evaluated). Then the state
    of damage stability: run, with the count of cases; or not run, with the button.
  - **What did I get?** The headline numbers: cargo volume and cargo mass, and what governs the cargo; for a
    Cargo in tanks, the tanks across and along and the tank diameter; laden draught against the deepest
    draught allowed; freeboard; lightship and displacement; installed power and service speed; air draft.
  - **What should I look at next?** The warning checks that are not met, in the words the Checks panel
    uses, each with a link to its place; and the count of typed and default inputs, with a link to the
    Input panel.
  The Summary view opens after Create and when a Design is opened. The other views do not change.
- **D8. The Required group holds all six required inputs.** Hull Form and the Service Area move into the
  Required group in the schema (their names, values and place in the file do not change). The river limits
  stay shown beside the Service Area for a river-only Option.
- **D9. The Input panel shows the essentials first.** A switch at the top: **Essentials** or **All
  inputs**. Essentials shows the required inputs, every input the designer typed, and the inputs the schema
  marks as essential for this Cargo and Formation (a new flag in the schema, set in the core next to each
  input, so `ovd schema` shows it too). The first list of essential inputs: moulded depth, minimum
  freeboard, maximum speed, design speed, power source, propulsor type, the largest tank diameter, the tanks
  across and along, the fill ratio, the hold covering of a dry-bulk hold, the draught limit, the air-draft
  limit, the lock length and the lock width. All inputs is the panel as it is today. The app remembers the
  choice per user. A Design made by the wizard opens on Essentials; so does the first start.
- **D10. The Input panel has a filter box.** Typing part of a label ("freeboard") shows the matching inputs
  of every group, in both modes, and opens their groups. Clearing the box restores the panel.
- **D11. Help is on the page, not only in a tooltip.** Each input has a small "about this input" control
  that opens the text the tooltip holds today (the default and where it comes from, the usual range, the
  basis) under the field. The tooltip stays.
- **D12. The app speaks about the app.** No message in the app tells the designer to run a command-line
  program. The Service Area error reads, in the app, "Choose a Service Area: river only, or coastal and
  river." The command line keeps its own hint. The check covers every message the app can show.
- **D13. The app says when it is working.** From the moment an edit is sent until the new evaluation is
  drawn, the header shows "Calculating…". The views keep their "out of date" mark as they do today.
- **D14. English first.** Every new text has its English message. The other Display Languages fall back to
  English until their drafts are written; that is a follow-up, not part of this build.

## 4. Not changed

The file format; every calculation, Rule, default and results hash; the first-run notice; the Outputs,
Checks, Warnings and Compare panels; the drawings; the command line and the Python API, except that
`ovd schema` shows the new `essential` flag and the new group of two inputs.

## 5. Testing

- **The session call (D4, D6).** Rust tests in the shell crate: for the 11 typed inputs of section 2, the
  Design it makes equals, input for input and source for source, the Design `fill --as-new` makes from the
  short file of the same inputs, and has the same results hash. The same for a dry-bulk pushed barge and for
  a river-only Option with both limits. A choice the schema refuses gives the schema's error and no Design.
  The example of D6 is those 11 inputs.
- **The wizard (D1 to D5).** Unit tests of the wizard's logic (the steps it shows for a Cargo and a Service
  Area, the limits, the Option name, the choices it sends), run by `npm run test:unit`. Web end-to-end tests,
  in their own new file: the Start page opens after the notice; the skip link gives the starter Design of
  today; the wizard makes the barge of section 2 with 11 typed inputs and no other typed input; the Summary
  view opens; damage stability runs once by itself; the cargo volume on the page equals the cargo volume
  `ovd run --json` gives for the same file. The existing end-to-end tests pass, with a shared helper that
  takes the skip link where they need the starter Design.
- **The Summary view (D7).** Unit tests of the function that turns an evaluation into the Summary's lines:
  a feasible gas barge, an Option with a failed blocking Rule, an Option that is not calculated, damage run
  and not run. A web end-to-end test in its own file: every number on the Summary equals the same number in
  the Outputs panel.
- **The Input panel (D8 to D11).** A Rust test that the six required inputs are in the Required group and
  that the `essential` flag is in the schema. The generated English input text is regenerated. Unit tests of
  the function that says which inputs Essentials shows, and of the filter. A web end-to-end test in its own
  file: Essentials hides a default input that is not essential and shows it after the designer types it;
  the filter finds "freeboard"; the help opens.
- **Plain words (D12, D13).** A test that fails when a message the app can show holds the name of the
  command-line program followed by a sub-command. A web end-to-end test that "Calculating…" appears after an
  edit and goes when the result is drawn.
- **No result moves.** `tools/check_fixture_hashes.py` reports no change, and the design-impact report shows
  no moved design, on every pull request of this spec.

New end-to-end tests go in their own files, one per ticket (`web-smoke.<issue>.test.mjs`), and the web
end-to-end script runs every `web-smoke*.test.mjs` file.

## 6. Phasing and tickets

Three lanes, two tickets each. Each ticket is one pull request.

- **Lane A, the wizard.** A1: the session call that makes a Design from typed choices, and the example
  (D4, D6), with this spec. A2: the Start page and the wizard, and the one automatic damage run (D1 to D3,
  D5). A2 needs A1.
- **Lane B, the answer.** B1: the Summary view (D7). B2: plain words and "Calculating…" (D12, D13).
- **Lane C, the Input panel.** C1: the Required group and the `essential` flag in the schema (D8, the core
  half of D9). C2: Essentials and All inputs, the filter box, and help on the page (D9 to D11). C2 needs C1.

File ownership: lane A owns the new wizard files, the session call and the changes to New in the shell
component; lane B owns the Summary view, the views' tab list and the message line; lane C owns the Input
panel components and the schema. A lane that must touch another lane's file keeps the change to the fewest
lines and says so in its pull request.

## 7. Not in scope

Translations of the new texts; more examples than one; a wizard for a second Option in an open Design; a
change to when damage stability runs, beyond the one run of D5; a bill of quantities view; any change to a
calculation, a Rule or a default; a release.
