//! Damage stability on demand (issue #924): the run of one Option off the main thread, and the runs
//! the session keeps.
//!
//! The app evaluates without damage, so an edit stays quick. A user asks for damage; a [`Job`] holds a
//! copy of the Option, so the app's lock is free while the cases run. The results are kept with a
//! fingerprint of what they depend on (see `DesignSession::option_fingerprint`) and shown only while it
//! still matches. This module holds no calculation: the engine runs the cases.

use ovd_format::damage::{ControlFlow, DAMAGE_CANCELLED, DamageControl};
use ovd_format::rule_set::Library;
use ovd_format::{DesignOption, RunOutcome, VesselRun, run_option_with_vessel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The damage control of an evaluation that does not run damage.
pub fn off() -> DamageControl {
    DamageControl {
        run: false,
        progress: None,
    }
}

/// How a damage run ended.
#[derive(Debug, Clone, PartialEq)]
pub enum End {
    /// The run finished; the outcome holds the results. The Vessel Model of the run's own pick goes with them
    /// (issue #1041), so the 3D view builds the screened model without a second search. `None` when the run
    /// gave none, or a worker's hand-back could not be read.
    Done(RunOutcome, Option<Box<VesselRun>>),
    /// The user (or an edit) cancelled it. Nothing is kept.
    Cancelled,
    /// The engine refused the Option; the text says why.
    Failed(String),
}

/// One damage run: a copy of the Option as the engine runs it, and what the results depend on.
pub struct Job {
    /// The Option's name when the run was asked for.
    pub option: String,
    /// See `DesignSession::option_fingerprint`.
    pub fingerprint: String,
    pub(crate) run_option: DesignOption,
    pub(crate) cancel: Arc<AtomicBool>,
}

impl Job {
    pub(crate) fn new(
        option: String,
        fingerprint: String,
        run_option: DesignOption,
        cancel: Arc<AtomicBool>,
    ) -> Job {
        Job {
            option,
            fingerprint,
            run_option,
            cancel,
        }
    }

    /// Ask the run to stop: no new case starts, and the run ends as [`End::Cancelled`].
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    /// A handle that cancels this run from another thread.
    pub fn canceller(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }

    /// Run the damage cases. `on_progress(done, total)` is called on this thread, first with 0 done and
    /// last with every case done, and it counts only (a slow call slows the run).
    pub fn run(&self, on_progress: impl FnMut(usize, usize) + 'static) -> End {
        let on_progress = RefCell::new(on_progress);
        let cancel = self.cancel.clone();
        let control = DamageControl {
            run: true,
            progress: Some(Rc::new(move |p| {
                (on_progress.borrow_mut())(p.done, p.total);
                if cancel.load(Ordering::SeqCst) {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            })),
        };
        match run_option_with_vessel(&self.run_option, &Library::built_in(), &[], &control) {
            (RunOutcome::Failed(m), _) if m.contains(DAMAGE_CANCELLED) => End::Cancelled,
            (RunOutcome::Failed(m), _) => End::Failed(m),
            (RunOutcome::NotAvailable(m), _) => End::Failed(m),
            (done @ RunOutcome::Results { .. }, model) => {
                if self.cancel.load(Ordering::SeqCst) {
                    End::Cancelled
                } else {
                    End::Done(done, model.map(Box::new))
                }
            }
        }
    }
}

/// The run an Option last had: the results and the fingerprint they were run for.
#[derive(Debug, Clone)]
pub(crate) struct Kept {
    pub fingerprint: String,
    pub outcome: RunOutcome,
    /// The Vessel Model of the run's pick, for the 3D view (issue #1041); `None` when the run gave none.
    pub model: Option<Box<VesselRun>>,
}

/// The run under way, so that an edit of its Option can cancel it.
pub(crate) struct Running {
    pub option: String,
    pub fingerprint: String,
    pub cancel: Arc<AtomicBool>,
}
