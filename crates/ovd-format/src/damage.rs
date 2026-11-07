//! The damage a gas hull is run through (spec 0012). The case loop, the damage data a Rule Set gives it and the
//! Permeability Tables are in `ovd_core::gas_damage` (issue #935: the arrangement search calls them); this module
//! runs the cases on worker threads, counts and cancels them, and renders the `damage` block of the results.

use arc_rules::model::RuleSet;
use ovd_core::damage_cases::PermeabilityTable;

pub use ovd_core::gas_damage::{
    GasBoxRule, GasRun, IGC_DAMAGE_TABLES, IgcDamageData, gas_box_damage_plan, gas_damage_plan,
    igc_damage_data, permeability_table,
};
use ovd_core::gas_damage::{GasDamageRun, afloat, run_case};

/// The Permeability Table `table_id` of the highest built-in version of set `set_id` that holds it. A run reads
/// the liquid permeabilities this way whatever version of the set the design pins: the older versions hold no
/// table, and the values were in code before.
pub fn built_in_permeability(set_id: &str, table_id: &str) -> Result<PermeabilityTable, String> {
    let lib = crate::rule_set::built_in();
    let mut sets: Vec<&RuleSet> = lib
        .sets()
        .iter()
        .filter(|s| s.id == set_id && s.tables.iter().any(|t| t.id == table_id))
        .collect();
    sets.sort_by(|a, b| {
        if crate::rule_set::is_newer(&a.version, &b.version) {
            core::cmp::Ordering::Greater
        } else if crate::rule_set::is_newer(&b.version, &a.version) {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Equal
        }
    });
    let set = sets
        .last()
        .ok_or_else(|| format!("no built-in `{set_id}` holds table `{table_id}`"))?;
    permeability_table(set, table_id)
}

/// Issue #759 (spec 0012): the IGC damage cases of a gas hull on every loading condition, final stage of
/// flooding by lost buoyancy, with the criteria of IGC 2.7.1.2 and 2.7.2.1. See
/// [`ovd_core::gas_damage`] for the cases.
///
/// The second value is the `damage` block of the results (unhashed, D21): every case with its condition, its
/// floating position, its residual stability and the deck-edge test (D11), and for each condition the governing
/// case with its righting lever curve over the 20 degree range (D12).
pub fn gas_damage_verdicts(
    set: &RuleSet,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
) -> Result<(Vec<ovd_core::measured::DamageVerdict>, serde_json::Value), String> {
    let plan = gas_damage_plan(set, spaces, hull)?;
    run_gas_damage(&plan, spaces, hull, conditions, density, &Gauge::none())
        .map_err(|Cancelled| "cancelled".to_string())
}

/// Issue #763 (spec 0012 D2, D3): the damage cases of ADN 9.3.1.15 or ES-TRIN 28.03 on every loading condition of
/// a gas hull, final stage by lost buoyancy and the intermediate stages by added mass (D15 to D18). The block is
/// shaped as the IGC one is ([`gas_damage_verdicts`]).
pub fn gas_box_damage_verdicts(
    rule: GasBoxRule,
    set: &RuleSet,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
) -> Result<(Vec<ovd_core::measured::DamageVerdict>, serde_json::Value), String> {
    let plan = gas_box_damage_plan(rule, set, spaces, hull)?;
    run_gas_damage(&plan, spaces, hull, conditions, density, &Gauge::none())
        .map_err(|Cancelled| "cancelled".to_string())
}

/// The damage cases of `run` on every loading condition of a gas hull, the cases of each condition on worker
/// threads (issue #921). The results come back in case order and the governing case is chosen over them in that
/// order, so the output is the serial one, bit for bit.
pub(crate) fn run_gas_damage(
    run: &GasRun,
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
    gauge: &Gauge,
) -> Result<(Vec<ovd_core::measured::DamageVerdict>, serde_json::Value), Cancelled> {
    let workers = damage_workers(run.cases.len());
    let mut out = GasDamageRun::default();
    for c in conditions {
        let Some(state) = afloat(run, c, hull, density) else {
            continue;
        };
        let runs = run_ordered(&run.cases, workers, gauge, |case| {
            run_case(run, spaces, hull, c, &state, density, case)
        })?;
        out.push_condition(run, hull, c, &state, density, runs);
    }
    let block = damage_block(run, &out);
    Ok((out.verdicts, block))
}

fn floats_text(f: ovd_core::damage_stability::Floats) -> &'static str {
    use ovd_core::damage_stability::Floats;
    match f {
        Floats::Yes => "yes",
        Floats::Sinks => "sinks",
        Floats::NoEquilibrium => "no equilibrium",
    }
}

/// The `damage` block of the results from a typed run: every case, and the governing case of each condition.
pub(crate) fn damage_block(run: &GasRun, out: &GasDamageRun) -> serde_json::Value {
    use ovd_core::damage_stability::INTERMEDIATE_FRACTIONS;
    use serde_json::json;
    let num = crate::eval::num;
    let rows: Vec<serde_json::Value> = out
        .cases
        .iter()
        .zip(&out.verdicts)
        .map(|(rec, v)| {
            let o = &v.outcome;
            json!({
                "condition": rec.condition,
                "name": rec.name,
                "flooded": rec.flooded,
                "breached_tanks": rec.breached_tanks,
                "floats": floats_text(o.floats),
                "heel_deg": num(o.heel_deg),
                "trim_deg": num(o.trim_deg),
                "draught_mid_m": num(o.draught_mid_m),
                "range_end_deg": num(o.end_deg),
                "max_gz_m": num(o.max_gz_m),
                "area_m_rad": num(o.area_m_rad),
                "deck_edge_immersed": rec.deck_edge_immersed,
                "ok": o.ok,
                "stages": v.stages.iter().zip(INTERMEDIATE_FRACTIONS).map(|(st, fr)| json!({
                    "fraction": num(fr),
                    "floats": floats_text(st.floats),
                    "heel_deg": num(st.heel_deg),
                    "max_gz_m": num(st.max_gz_m),
                    "area_m_rad": num(st.area_m_rad),
                    "ok": st.ok,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let governing: Vec<serde_json::Value> = out
        .governing
        .iter()
        .map(|g| {
            let o = &out.verdicts[g.case_index].outcome;
            // What the damage drawing shows (issue #761): each Flooded Volume's shape, and the damaged
            // waterplane.
            let geometry: Vec<serde_json::Value> = g
                .flooded
                .iter()
                .map(|v| match &v.shape {
                    ovd_core::compartments::SpaceShape::Prism { section } => json!({
                        "name": v.name,
                        "x_m": [num(v.x_aft_m), num(v.x_fwd_m)],
                        "section_m": section.iter().map(|&(y, z)| json!([num(y), num(z)])).collect::<Vec<_>>(),
                    }),
                    ovd_core::compartments::SpaceShape::Tank {
                        y,
                        z,
                        outer_diameter,
                        ..
                    } => json!({
                        "name": v.name,
                        "x_m": [num(v.x_aft_m), num(v.x_fwd_m)],
                        "tank": { "y_m": num(*y), "z_m": num(*z), "diameter_m": num(*outer_diameter) },
                    }),
                })
                .collect();
            let mut entry = json!({
                "condition": g.condition,
                "case": rows[g.case_index]["name"],
                "ok": o.ok,
                "floats": rows[g.case_index]["floats"],
                "heel_deg": num(o.heel_deg),
                "trim_deg": num(o.trim_deg),
                "draught_mid_m": num(o.draught_mid_m),
                "waterplane": { "heel_deg": num(o.heel_deg), "trim_deg": num(o.trim_deg), "c_m": num(o.waterplane_c_m) },
                "flooded": geometry,
                "gz_after_damage": g.gz_after_damage.iter().map(|&(h, v)| json!([num(h), num(v)])).collect::<Vec<_>>(),
            });
            // Issue #928: where the damage is, for the damage drawing.
            entry["x_m"] = json!([num(g.x_m.0), num(g.x_m.1)]);
            if let Some((y0, y1)) = g.y_m {
                entry["y_m"] = json!([num(y0), num(y1)]);
            }
            entry
        })
        .collect();
    json!({
        "rule_set": run.set_key,
        "method": run.method,
        "cases": rows,
        "governing": governing,
    })
}

/// What the damage stages of a gas hull give (issue #923): the IGC list and block, the ADN and ES-TRIN lists, their
/// `damage_by_rule` blocks, and the number of cases of the stages (counted even when the run skips them).
#[derive(Default)]
pub(crate) struct GasStages {
    pub igc: Vec<ovd_core::measured::DamageVerdict>,
    pub igc_block: serde_json::Value,
    pub adn: Vec<ovd_core::measured::DamageVerdict>,
    pub es_trin: Vec<ovd_core::measured::DamageVerdict>,
    pub by_rule: serde_json::Map<String, serde_json::Value>,
    pub cases: usize,
}

/// The three damage stages of a gas hull on its IS Code conditions: IGC 2.7, ADN 9.3.1.15 and ES-TRIN 28.03 (spec
/// 0012), each when its Rule Set is given. With `control.run` false no case is worked out, but the cases are
/// counted (`cases`). The hook of `control` is called on this thread, from the first call with 0 done, and a
/// Break from it gives `Err(Cancelled)`.
pub(crate) fn gas_damage_stages(
    sets: (Option<&RuleSet>, Option<&RuleSet>, Option<&RuleSet>),
    spaces: &[ovd_core::compartments::Space],
    hull: &ovd_core::hydrostatics::Hull,
    conditions: &[ovd_core::loading_conditions::LoadedCondition],
    density: f64,
    control: &DamageControl,
) -> Result<GasStages, Cancelled> {
    let (igc, adn, es_trin) = sets;
    let mut out = GasStages {
        igc_block: serde_json::Value::Null,
        ..GasStages::default()
    };
    if conditions.is_empty() {
        return Ok(out);
    }
    // A stage whose cases cannot be built (a Rule Set version without the tables) gives nothing, as before.
    let mut plans: Vec<(Option<GasBoxRule>, &RuleSet, GasRun)> = Vec::new();
    if let Some(set) = igc
        && let Ok(plan) = gas_damage_plan(set, spaces, hull)
    {
        plans.push((None, set, plan));
    }
    for (rule, set) in [(GasBoxRule::AdnGas, adn), (GasBoxRule::EsTrin, es_trin)] {
        if let Some(set) = set
            && let Ok(plan) = gas_box_damage_plan(rule, set, spaces, hull)
        {
            plans.push((Some(rule), set, plan));
        }
    }
    out.cases = plans
        .iter()
        .map(|(_, _, p)| p.case_count(hull, conditions, density))
        .sum();
    if !control.run {
        return Ok(out);
    }
    let gauge = Gauge::new(control.progress.clone(), out.cases);
    gauge.report();
    for (rule, set, plan) in &plans {
        let (v, block) = run_gas_damage(plan, spaces, hull, conditions, density, &gauge)?;
        match rule {
            None => (out.igc, out.igc_block) = (v, block),
            Some(r) => {
                match r {
                    GasBoxRule::AdnGas => out.adn = v,
                    GasBoxRule::EsTrin => out.es_trin = v,
                }
                out.by_rule.insert(set.id.clone(), block);
            }
        }
    }
    Ok(out)
}

/// How the `damage_by_rule` block names the method of the ES-TRIN 28.03 damage cases on a liquid bulk hull.
pub(crate) const ES_TRIN_METHOD: &str = "lost buoyancy, final stage; intermediate stages by added mass at 25, 50 and 75 % (ES-TRIN 2025/1 28.03(3) to (6)); laden departure";
/// How the `damage_by_rule` block names the method of the NORMAM 5.23.8 damage cases.
pub(crate) const NORMAM_METHOD: &str =
    "lost buoyancy, final stage (NORMAM-202/DPC 5.23.8, 5.23.9); laden departure";

thread_local! {
    static DAMAGE_RUNS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
}

/// How many damage runs the calling thread has started (a run that works cases out, not one that only counts
/// them). A test reads it before and after a call to show that the call ran no damage (issue #924). Per thread,
/// so tests that run side by side do not count each other's runs.
pub fn damage_runs_started() -> usize {
    DAMAGE_RUNS.with(core::cell::Cell::get)
}

/// How many worker threads the damage cases may use; 0 means one per core.
static DAMAGE_WORKERS: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

/// For tests only (issue #921), hence hidden from the docs: a process-wide knob, so a test that sets it must be
/// the only one in its test binary. Sets the number of worker threads for the damage cases: 1 runs them
/// serially, 0 (the default) uses one per core. The results are the same bits whatever the number. A build for
/// WebAssembly, or without the `parallel` feature, is always serial.
#[doc(hidden)]
pub fn set_damage_workers(n: usize) {
    DAMAGE_WORKERS.store(n, core::sync::atomic::Ordering::Relaxed);
}

/// The worker count for `cases` cases: at least 1, at most one per case.
pub(crate) fn damage_workers(cases: usize) -> usize {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        let set = DAMAGE_WORKERS.load(core::sync::atomic::Ordering::Relaxed);
        let n = if set == 0 {
            std::thread::available_parallelism().map_or(1, |n| n.get())
        } else {
            set
        };
        n.min(cases).max(1)
    }
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        let _ = (cases, &DAMAGE_WORKERS);
        1
    }
}

/// Issue #923: whether a run works out damage stability. A run that skips it still counts the cases
/// (`damage_cases` in the results), for a time estimate.
#[derive(Clone)]
pub struct DamageControl {
    /// Work out the damage cases. Default: true. False gives `damage_status` `not run`.
    pub run: bool,
    /// Called with the cases done and the total as the cases finish. See [`DamageProgress`].
    pub progress: Option<ProgressHook>,
}

impl Default for DamageControl {
    fn default() -> Self {
        Self {
            run: true,
            progress: None,
        }
    }
}

impl core::fmt::Debug for DamageControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DamageControl")
            .field("run", &self.run)
            .field("progress", &self.progress.as_ref().map(|_| "hook"))
            .finish()
    }
}

/// How far a damage run is: `done` of `total` cases, over every damage Rule Set of the run (issue #923).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageProgress {
    pub done: usize,
    pub total: usize,
}

/// The progress hook. It is called on the thread that called the run, never on a worker thread (a Python
/// callback, and the Ctrl-C of the Python main thread, need that): first with `done` 0, then whenever the count
/// of finished cases changes, at most about every 5 ms, and last when the run ends. Damage runs in one pass
/// only: the passes that size the default propeller or settle the rooms of a deckhouse stern skip it (it feeds
/// neither), so the count never falls. It returns
/// [`ControlFlow::Break`] to cancel: no new case starts, the cases under way finish, and the run stops with
/// [`DAMAGE_CANCELLED`]. Not `Send`: it stays on the calling thread.
pub type ProgressHook = std::rc::Rc<dyn Fn(DamageProgress) -> ControlFlow<()>>;

/// The message of a run that the progress hook cancelled (the `Raised` kind is `Cancelled`).
pub const DAMAGE_CANCELLED: &str = "damage stability was cancelled";

pub use core::ops::ControlFlow;

/// A damage run was cancelled by its progress hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cancelled;

/// Counts the cases done over the whole run, holds the cancel flag, and calls the hook (on the calling thread
/// only: the worker threads touch the two atomics, never the hook).
pub(crate) struct Gauge {
    hook: Option<ProgressHook>,
    total: usize,
    done: core::sync::atomic::AtomicUsize,
    cancelled: core::sync::atomic::AtomicBool,
}

impl Gauge {
    /// No hook, no count: the run works as it did before issue #923.
    pub(crate) fn none() -> Self {
        Self::new(None, 0)
    }

    pub(crate) fn new(hook: Option<ProgressHook>, total: usize) -> Self {
        if total > 0 {
            DAMAGE_RUNS.with(|c| c.set(c.get() + 1));
        }
        Self {
            hook,
            total,
            done: core::sync::atomic::AtomicUsize::new(0),
            cancelled: core::sync::atomic::AtomicBool::new(false),
        }
    }

    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    pub(crate) fn has_hook(&self) -> bool {
        self.hook.is_some()
    }

    pub(crate) fn done(&self) -> usize {
        self.done.load(core::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(core::sync::atomic::Ordering::Relaxed)
    }

    /// [`Gauge::report`] for the thread that waits on workers: if the hook panics, the cancel flag is set before
    /// the panic goes on, so the workers stop at their next case and do not run every case that is left.
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    pub(crate) fn report_guarded(&self) {
        if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.report()))
        {
            self.cancelled
                .store(true, core::sync::atomic::Ordering::Relaxed);
            std::panic::resume_unwind(panic);
        }
    }

    /// Call the hook with the count now; a Break sets the cancel flag. Calling thread only. After a cancel the
    /// hook is not called again.
    pub(crate) fn report(&self) {
        if self.is_cancelled() {
            return;
        }
        if let Some(h) = &self.hook
            && h(DamageProgress {
                done: self.done(),
                total: self.total,
            })
            .is_break()
        {
            self.cancelled
                .store(true, core::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Run `f` on every item and return the results in the order of the items. With `workers` above 1 the items
/// are shared between scoped threads (std only), each taking the next item not yet taken. `f` is pure in its
/// item, so the order of the results does not depend on which worker ran which item. This is the one place a
/// per-case progress call or a cancel check belongs (issue #923): after `f` returns, or before a worker takes
/// its next item.
pub(crate) fn run_ordered<T, R, F>(
    items: &[T],
    workers: usize,
    gauge: &Gauge,
    f: F,
) -> Result<Vec<R>, Cancelled>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    use core::sync::atomic::Ordering;
    if gauge.is_cancelled() {
        return Err(Cancelled);
    }
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    if workers > 1 && items.len() > 1 {
        use std::sync::atomic::AtomicUsize;
        let next = AtomicUsize::new(0);
        // The workers see the two atomics and nothing else of the gauge: the hook stays on this thread.
        let (next, f, done, cancelled) = (&next, &f, &gauge.done, &gauge.cancelled);
        let mut slots: Vec<Option<R>> = items.iter().map(|_| None).collect();
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(move || {
                        let mut out: Vec<(usize, R)> = Vec::new();
                        while !cancelled.load(Ordering::Relaxed) {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(item) = items.get(i) else { break };
                            out.push((i, f(item)));
                            done.fetch_add(1, Ordering::Relaxed);
                        }
                        out
                    })
                })
                .collect();
            // This thread polls the counter while the workers run, and calls the hook (issue #923).
            if gauge.has_hook() {
                let mut last = gauge.done();
                while !handles.iter().all(|h| h.is_finished()) {
                    if gauge.done() != last {
                        last = gauge.done();
                        gauge.report_guarded();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                if gauge.done() != last {
                    gauge.report_guarded();
                }
            }
            for h in handles {
                match h.join() {
                    Ok(out) => {
                        for (i, r) in out {
                            slots[i] = Some(r);
                        }
                    }
                    Err(panic) => std::panic::resume_unwind(panic),
                }
            }
        });
        if gauge.is_cancelled() {
            return Err(Cancelled);
        }
        return Ok(slots
            .into_iter()
            .map(|r| r.expect("every case ran once"))
            .collect());
    }
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    let _ = workers;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(f(item));
        gauge.done.fetch_add(1, Ordering::Relaxed);
        gauge.report();
        if gauge.is_cancelled() {
            return Err(Cancelled);
        }
    }
    Ok(out)
}

#[cfg(all(test, feature = "parallel", not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Issue #923: a hook that panics stops the workers at their next case; they do not run every case left.
    #[test]
    fn a_panicking_hook_stops_the_workers() {
        let ran = AtomicUsize::new(0);
        let items: Vec<usize> = (0..400).collect();
        let hook: ProgressHook = std::rc::Rc::new(|p| {
            if p.done >= 2 {
                panic!("hook failed");
            }
            ControlFlow::Continue(())
        });
        let gauge = Gauge::new(Some(hook), items.len());
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_ordered(&items, 4, &gauge, |_| {
                ran.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(std::time::Duration::from_millis(5));
            })
        }));
        assert!(caught.is_err(), "the panic did not reach the caller");
        assert!(gauge.is_cancelled());
        let n = ran.load(Ordering::Relaxed);
        assert!(n < 100, "{n} of 400 cases ran after the hook panicked");
    }

    /// Issue #961: a cancel stops the workers at their next case. The result is `Cancelled` either way (the flag is
    /// read after the scope), so the count of cases started is what shows the early stop.
    #[test]
    fn a_cancel_stops_the_workers_early() {
        let started = AtomicUsize::new(0);
        let items: Vec<usize> = (0..200).collect();
        let at_break = std::rc::Rc::new(std::cell::Cell::new(None));
        let seen = at_break.clone();
        let hook: ProgressHook = std::rc::Rc::new(move |p| {
            if p.done >= 1 {
                seen.set(Some(seen.get().unwrap_or(p.done)));
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        });
        let gauge = Gauge::new(Some(hook), items.len());
        let out = run_ordered(&items, 4, &gauge, |_| {
            started.fetch_add(1, Ordering::Relaxed);
            std::thread::sleep(std::time::Duration::from_millis(2));
        });
        assert!(out.is_err());
        let (n, done) = (started.load(Ordering::Relaxed), at_break.get().unwrap());
        // Cases under way finish (one per worker), and the poll is up to 5 ms apart: about 10 more starts.
        assert!(
            n <= done + 4 + 20,
            "{n} cases started; {done} were done at the break"
        );
        assert!(n < 50, "{n} of 200 cases started after the cancel");
    }
}
