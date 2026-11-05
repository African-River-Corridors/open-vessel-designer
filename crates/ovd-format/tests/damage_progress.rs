//! Seam: the progress hook of a damage run (issue #923), for the Python callback and its Ctrl-C (#940).
//!
//! Label: property - the 140 x 29 m coastal LNG reference design with 1 and with 4 worker threads. The hook
//! is called on the thread that called the run, never on a worker, first with 0 cases done, then with a count
//! that never falls within a pass (a run repeats its passes for the propeller sizing and the deckhouse rooms,
//! and each pass counts from 0 again), last with every case done; the total is the case count the results give. A hook that
//! breaks stops the run: it gives a failure that says the damage was cancelled, and no later call follows.
//! The worker count is a process-wide knob, so this file holds this one test.

use ovd_format::damage::{ControlFlow, DamageControl, DamageProgress, set_damage_workers};
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option_with_damage};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const EXAMPLE: &str =
    include_str!("../../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign");

fn run(control: &DamageControl) -> RunOutcome {
    let mut file: Value = serde_json::from_str(EXAMPLE).unwrap();
    let mut option = file["options"][0].clone();
    let inputs = option["inputs"].as_object_mut().unwrap();
    inputs.retain(|_, v| v["source"] != "default");
    inputs.insert(
        "service_area".into(),
        json!({ "value": "coastal-and-river", "source": "typed" }),
    );
    option.as_object_mut().unwrap().remove("results_hash");
    file["options"] = json!([option]);
    file.as_object_mut().unwrap().remove("defaults_profile");
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    run_option_with_damage(
        &filled.design.unwrap().options[0],
        ovd_format::rule_set::built_in(),
        &[],
        control,
    )
}

type Calls = Rc<RefCell<Vec<(DamageProgress, std::thread::ThreadId)>>>;

/// A hook that records each call and the thread, and breaks once `break_at` cases are done.
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

fn results_of(outcome: RunOutcome) -> Value {
    match outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_hook_runs_on_the_calling_thread_counts_up_and_can_cancel() {
    let me = std::thread::current().id();
    for workers in [1, 4] {
        set_damage_workers(workers);
        // A full run with a hook: the same results as one without.
        let plain = results_of(run(&DamageControl::default()));
        let total = plain["damage_cases"].as_u64().unwrap() as usize;
        assert!(total > 4, "{workers} workers: only {total} cases");
        let (control, calls) = recording(None);
        let hooked = results_of(run(&control));
        assert_eq!(
            serde_json::to_string(&plain).unwrap(),
            serde_json::to_string(&hooked).unwrap(),
            "{workers} workers: a hook changed the results"
        );
        let calls = calls.borrow();
        assert!(
            calls.iter().all(|(_, t)| *t == me),
            "{workers} workers: the hook ran off the calling thread"
        );
        assert_eq!(calls[0].0, DamageProgress { done: 0, total });
        // A run can repeat (the propeller sizing, the deckhouse rooms): each pass counts from 0 again.
        assert!(
            calls
                .windows(2)
                .all(|w| w[1].0.done == 0 || w[0].0.done <= w[1].0.done),
            "{workers} workers: the count fell inside a pass"
        );
        assert!(calls.iter().all(|(p, _)| p.total == total));
        assert_eq!(calls.last().unwrap().0.done, total, "{workers} workers");

        // Cancel after 3 cases: the run fails, says so, and the hook is not called again.
        let (control, calls) = recording(Some(3));
        match run(&control) {
            RunOutcome::Failed(m) => assert!(m.contains("cancelled"), "{m}"),
            other => panic!("{workers} workers: a cancelled run gave {other:?}"),
        }
        let calls = calls.borrow();
        let last = calls.last().unwrap().0;
        assert!(
            last.done >= 3 && last.done < total,
            "{workers} workers: {last:?}"
        );
        assert_eq!(
            calls.iter().filter(|(p, _)| p.done >= 3).count(),
            1,
            "{workers} workers: the hook was called again after it broke"
        );
        // Cancel at the very start (0 done): nothing runs.
        let (control, calls) = recording(Some(0));
        assert!(matches!(run(&control), RunOutcome::Failed(_)));
        assert_eq!(calls.borrow().len(), 1);
    }
    set_damage_workers(0);
}
