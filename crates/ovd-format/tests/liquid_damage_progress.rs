//! Seam: the damage cases of a liquid bulk hull count, report progress and cancel as the gas ones do (issue
//! #961, after #923).
//!
//! Label: property - the crude integral-tank barge fixtures (ADN, and MARPOL under the coastal stack) with 1 and
//! with 4 worker threads. `damage_cases` of a run that skips damage equals the cases a full run works out; the
//! hook is called on the calling thread, first with 0 done, then with a count that never falls, last with every
//! case done, and the total is that count; a hook that breaks stops the run with the same "cancelled" failure
//! and no results. The worker count is a process-wide knob, so this file holds one test that sets it.

use ovd_format::damage::{ControlFlow, DamageControl, DamageProgress, set_damage_workers};
use ovd_format::{Profile, RunOutcome, check, run_option_with_damage};
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

const FIXTURES: [&str; 2] = [
    "crude_integral_tanks_pushed",
    "crude_integral_tanks_coastal",
];

fn run(name: &str, control: &DamageControl) -> RunOutcome {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap();
    let checked = check(&text, &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    run_option_with_damage(
        &checked.design.unwrap().options[0],
        ovd_format::rule_set::built_in(),
        &[],
        control,
    )
}

fn results_of(outcome: RunOutcome) -> Value {
    match outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// The cases a full run worked out: the ADN block, the MARPOL block and every rule of `damage_by_rule`.
fn cases_worked_out(r: &Value) -> usize {
    let n = |v: &Value| v.as_array().map_or(0, Vec::len);
    let by_rule: usize = r["damage_by_rule"]
        .as_object()
        .map_or(0, |o| o.values().map(|b| n(&b["cases"])).sum());
    n(&r["damage"]) + n(&r["marpol_damage"]) + by_rule
}

fn skip() -> DamageControl {
    DamageControl {
        run: false,
        progress: None,
    }
}

type Calls = Rc<RefCell<Vec<(DamageProgress, std::thread::ThreadId)>>>;

fn recording(break_at: Option<usize>) -> (DamageControl, Calls) {
    let calls: Calls = Rc::new(RefCell::new(Vec::new()));
    let log = calls.clone();
    let control = DamageControl {
        run: true,
        progress: Some(Rc::new(move |p| {
            log.borrow_mut().push((p, std::thread::current().id()));
            match break_at {
                Some(n) if p.done >= n => ControlFlow::Break(()),
                _ => ControlFlow::Continue(()),
            }
        })),
    };
    (control, calls)
}

#[test]
fn liquid_cases_count_report_progress_and_cancel() {
    let me = std::thread::current().id();
    for workers in [1, 4] {
        set_damage_workers(workers);
        for name in FIXTURES {
            let ctx = format!("{name}, {workers} workers");
            let plain = results_of(run(name, &DamageControl::default()));
            let total = cases_worked_out(&plain);
            assert!(total > 4, "{ctx}: only {total} cases");
            assert_eq!(plain["damage_cases"].as_u64(), Some(total as u64), "{ctx}");
            // A run that skips damage still gives the count of a full run.
            let skipped = results_of(run(name, &skip()));
            assert_eq!(skipped["damage_status"], "not run", "{ctx}");
            assert_eq!(
                skipped["damage_cases"].as_u64(),
                Some(total as u64),
                "{ctx}"
            );
            // A hook changes no byte of the results, and `done` reaches `total`.
            let (control, calls) = recording(None);
            let hooked = results_of(run(name, &control));
            assert_eq!(
                serde_json::to_string(&plain).unwrap(),
                serde_json::to_string(&hooked).unwrap(),
                "{ctx}: a hook changed the results"
            );
            let calls = calls.borrow();
            assert!(calls.iter().all(|(_, t)| *t == me), "{ctx}: off thread");
            assert_eq!(calls[0].0, DamageProgress { done: 0, total }, "{ctx}");
            assert!(
                calls.windows(2).all(|w| w[0].0.done <= w[1].0.done),
                "{ctx}: the count fell"
            );
            assert!(calls.iter().all(|(p, _)| p.total == total), "{ctx}");
            assert_eq!(calls.last().unwrap().0.done, total, "{ctx}");

            // Cancel after 3 cases: the run fails, says so, and the hook is not called again.
            let (control, calls) = recording(Some(3));
            match run(name, &control) {
                RunOutcome::Failed(m) => assert!(m.contains("cancelled"), "{ctx}: {m}"),
                other => panic!("{ctx}: a cancelled run gave {other:?}"),
            }
            let calls = calls.borrow();
            let last = calls.last().unwrap().0;
            // Serial: it stops before the end. On workers the 18 cases can all finish before the hook polls.
            assert!(last.done >= 3 && last.done <= total, "{ctx}: {last:?}");
            if workers == 1 {
                assert!(last.done < total, "{ctx}: {last:?}");
            }
            assert_eq!(
                calls.iter().filter(|(p, _)| p.done >= 3).count(),
                1,
                "{ctx}: the hook was called again after it broke"
            );
            // Cancel at the very start: nothing runs.
            let (control, calls) = recording(Some(0));
            assert!(
                matches!(run(name, &control), RunOutcome::Failed(_)),
                "{ctx}"
            );
            assert_eq!(calls.borrow().len(), 1, "{ctx}");
        }
    }
    set_damage_workers(0);
}
