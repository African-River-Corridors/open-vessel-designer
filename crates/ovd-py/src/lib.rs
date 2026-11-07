//! Python bindings for the `open-vessel-designer` wheel (`ovd._native`).
//!
//! One function does the work: `eval(entry_point, case_json) -> result_json`, the Rust core's seam
//! for every entry point. `simulate_damage` is the one other: the same run with damage on, plus a
//! progress callable and a cancel callable (issue #940). The Python package turns its public calls into that JSON and the result
//! back into its dataclasses (`ovd/_bridge.py`), so the maths lives only in the Rust
//! core. This crate is a workspace member but not a default member, so
//! `cargo build` and `cargo test` never need Python. The core never depends on PyO3: only this
//! crate does.

use ovd_format::EvalError;
use ovd_format::damage::{ControlFlow, DamageProgress, ProgressHook};
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// The version of the compiled core, the same as the Python package version.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Run one case through the Rust core. `case_json` is `{"fn": ..., "args": {...}}`; the result is
/// `{"ok": ...}` or `{"error": {"type": ..., "message": ...}}`, as `ovd_format::eval` writes them. Input
/// the core does not accept raises `TypeError`; an unknown entry point raises `RuntimeError`.
#[pyfunction]
fn eval(py: Python<'_>, entry_point: &str, case_json: &str) -> PyResult<String> {
    let input = ovd_format::parse_case(case_json)
        .map_err(|e| PyValueError::new_err(format!("case is not JSON: {e}")))?;
    // The core holds no Python object, so other Python threads run while it calculates (issue #940).
    // A plain call has no callback, so Ctrl-C waits for it to return.
    match py.detach(|| ovd_format::eval(entry_point, &input)) {
        Ok(v) => Ok(v.to_string()),
        Err(e) => Err(error_of(e)),
    }
}

fn error_of(e: EvalError) -> PyErr {
    match e {
        EvalError::BadInput(m) => PyTypeError::new_err(m),
        e => PyRuntimeError::new_err(e.to_string()),
    }
}

/// The damage stage of `simulate` (issue #940): `case_json` is `{"fn": "simulate_damage", "args":
/// {"params": ...}}`. The GIL is released while the core runs. The core calls the progress hook on this
/// thread only, never on a worker (the main thread is the one that sees Ctrl-C), so the hook takes the
/// GIL, calls `progress(done, total)` then `cancel()`, and asks Python for a pending signal. A true
/// `cancel()`, an exception from either callable, or a signal stops the run. A true `cancel()` gives the
/// results with `damage_status` `not run`; an exception or a signal is raised after the run stops.
#[pyfunction]
#[pyo3(signature = (case_json, progress=None, cancel=None))]
fn simulate_damage(
    py: Python<'_>,
    case_json: &str,
    progress: Option<Py<PyAny>>,
    cancel: Option<Py<PyAny>>,
) -> PyResult<String> {
    let input = ovd_format::parse_case(case_json)
        .map_err(|e| PyValueError::new_err(format!("case is not JSON: {e}")))?;
    let args = input.get("args").cloned().unwrap_or_default();
    let (answer, failure) = py.detach(move || {
        let failure: Rc<RefCell<Option<PyErr>>> = Rc::new(RefCell::new(None));
        let kept = failure.clone();
        let hook: ProgressHook = Rc::new(move |p: DamageProgress| {
            Python::attach(|py| {
                let step = || -> PyResult<bool> {
                    if let Some(f) = &progress {
                        f.call1(py, (p.done, p.total))?;
                    }
                    if let Some(f) = &cancel
                        && f.call0(py)?.bind(py).is_truthy()?
                    {
                        return Ok(true);
                    }
                    py.check_signals()?;
                    Ok(false)
                };
                match step() {
                    Ok(false) => ControlFlow::Continue(()),
                    Ok(true) => ControlFlow::Break(()),
                    Err(e) => {
                        *kept.borrow_mut() = Some(e);
                        ControlFlow::Break(())
                    }
                }
            })
        });
        let seen = failure.clone();
        let answer =
            ovd_format::simulate_damage_with(&args, Some(hook), &|| seen.borrow().is_some());
        let failure = failure.borrow_mut().take();
        (answer, failure)
    });
    if let Some(e) = failure {
        return Err(e);
    }
    match answer {
        Ok(v) => Ok(v.to_string()),
        Err(e) => Err(error_of(e)),
    }
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_damage, m)?)?;
    Ok(())
}
