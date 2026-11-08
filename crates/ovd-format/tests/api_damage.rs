//! Seam: the `api` entry's damage stage, `simulate_damage` (issue #940). One build path: the
//! stage gives what `simulate` with `damage: true` gives, on the whole value, damage blocks included
//! (the results hash leaves them out, so it cannot prove this). A progress hook sees the cases rise to
//! the total; a hook that breaks gives the results with `damage_status` `not run`.
//!
//! Label: property. Each test compares paths to one answer; no number is pinned.

use ovd_format::damage::{ControlFlow, ProgressHook};
use ovd_format::{eval, simulate_damage_with};
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const EXAMPLE: &str =
    include_str!("../../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign");
const DRY_BULK: &str = include_str!("fixtures/dry_bulk_box_hold.vdesign");

fn typed(file: &str) -> Map<String, Value> {
    let root: Value = serde_json::from_str(file).expect("JSON");
    let mut p: Map<String, Value> = root["options"][0]["inputs"]
        .as_object()
        .expect("inputs")
        .iter()
        .filter(|(_, i)| i["source"] == "typed")
        .map(|(k, i)| (k.clone(), i["value"].clone()))
        .collect();
    p.entry("service_area").or_insert(json!("river-only"));
    p
}

fn ok(v: Result<Value, ovd_format::EvalError>) -> Value {
    let v = v.expect("a call");
    assert!(v.get("ok").is_some(), "{v}");
    v["ok"].clone()
}

fn simulate(p: &Map<String, Value>, damage: bool) -> Value {
    let case = json!({ "fn": "simulate", "args": { "params": p, "damage": damage } });
    ok(eval("api", &case))
}

fn text(v: &Value) -> String {
    serde_json::to_string(v).unwrap()
}

type Calls = Rc<RefCell<Vec<(usize, usize)>>>;

/// A hook that records `(done, total)`, and breaks after `break_after` calls if given.
fn hook(break_after: Option<usize>) -> (ProgressHook, Calls) {
    let calls: Calls = Rc::new(RefCell::new(Vec::new()));
    let log = calls.clone();
    let h: ProgressHook = Rc::new(move |p| {
        log.borrow_mut().push((p.done, p.total));
        match break_after {
            Some(n) if log.borrow().len() >= n => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    });
    (h, calls)
}

fn stage(p: &Map<String, Value>, h: Option<ProgressHook>) -> Value {
    ok(simulate_damage_with(&json!({ "params": p }), h, &|| false))
}

#[test]
fn the_json_seam_stage_equals_simulate_with_damage_on_the_whole_value() {
    let p = typed(EXAMPLE);
    let case = json!({ "fn": "simulate_damage", "args": { "params": p } });
    let staged = ok(eval("api", &case));
    assert_eq!(staged["damage_status"], "run");
    assert_eq!(text(&staged), text(&simulate(&p, true)));
    // With a hook, and without one: the same value.
    let (h, _) = hook(None);
    assert_eq!(text(&stage(&p, Some(h))), text(&staged));
}

#[test]
fn the_hook_sees_done_reach_total() {
    let p = typed(EXAMPLE);
    let (h, calls) = hook(None);
    let r = stage(&p, Some(h));
    let total = r["damage_cases"].as_u64().unwrap() as usize;
    let calls = calls.borrow();
    assert!(total > 3);
    // Issue #935: the search runs damage on each candidate that passes stability, then the answer's run follows.
    // The count is cumulative over those runs: `done` and the fraction done never fall, `total` is planned (the
    // search does not know how many runs it will make) and shrinks to the real total at the end, and the last call
    // has every case done. The answer's own
    // cases (`damage_cases`) are the last run's share of it.
    assert_eq!(calls[0].0, 0);
    assert!(calls.windows(2).all(|w| w[0].0 <= w[1].0), "done fell");
    // The total is planned (the runs the search expects), so it may shrink at the end: the fraction done never falls.
    assert!(
        calls
            .windows(2)
            .all(|w| w[0].0 as f64 / w[0].1 as f64 <= w[1].0 as f64 / w[1].1 as f64 + 1e-12),
        "the fraction fell"
    );
    let last = *calls.last().unwrap();
    assert_eq!(last.0, last.1);
    assert!(
        last.1 >= total,
        "{last:?} against {total} cases in the answer"
    );
}

#[test]
fn a_hook_that_breaks_gives_the_results_with_damage_not_run() {
    let p = typed(EXAMPLE);
    let (h, calls) = hook(Some(3));
    let r = stage(&p, Some(h));
    assert_eq!(r["damage_status"], "not run");
    assert_eq!(text(&r), text(&simulate(&p, false)));
    let last = calls.borrow().last().copied().unwrap();
    assert!(last.0 < last.1, "{last:?}");
}

#[test]
fn a_hull_with_no_damage_rule_set_is_not_applicable_and_the_hook_is_not_called() {
    let p = typed(DRY_BULK);
    let (h, calls) = hook(None);
    let r = stage(&p, Some(h));
    assert_eq!(r["damage_status"], "not applicable");
    assert_eq!(text(&r), text(&simulate(&p, true)));
    assert!(calls.borrow().is_empty());
}

/// Issue #935: what a pass of a run is for, and whether the number of tanks along was typed, are the run's own
/// (`RunRules`), not parameters: a parameter map that carries the names of the old internal keys is refused as an
/// unknown input, and cannot reach the results.
#[test]
fn parameters_cannot_set_the_role_of_a_pass_or_the_choice_of_tanks_along() {
    let p = typed(EXAMPLE);
    for key in ["__settle_discard", "__settle_last", "__tanks_in_line_typed"] {
        let mut with = p.clone();
        with.insert(key.to_string(), json!(true));
        let case = json!({ "fn": "simulate", "args": { "params": with, "damage": false } });
        let err = eval("api", &case).expect_err("the key is refused");
        assert!(
            err.to_string().contains(&format!("unknown input `{key}`")),
            "{err}"
        );
    }
}
