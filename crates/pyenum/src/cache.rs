//! Per-type, per-interpreter Python-class cache.
//!
//! The derive gives each enum type a dedicated `static CACHE: PyOnceLock<Py<PyType>>`.
//! This module owns the accessor that initialises the cell on first use via
//! [`crate::construct::build_py_enum`] and returns a borrowed `Bound<'py, PyType>`
//! on every subsequent call.

use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyType;

use crate::construct::build_py_enum;
use crate::trait_def::PyEnumSpec;

#[cfg(test)]
use core::sync::atomic::{AtomicUsize, Ordering};

/// Global counter of first-call constructions observed by [`get_or_build`].
///
/// Test-only: incremented exactly once per distinct `cell`, the first time
/// [`PyOnceLock::get_or_try_init`]'s closure actually runs. Subsequent cache
/// hits on the same cell do not increment the counter, so the value reflects
/// the number of cells that have ever been initialised process-wide.
#[cfg(test)]
static CONSTRUCTION_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Read the current value of [`CONSTRUCTION_COUNTER`].
#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn construction_counter() -> usize {
    CONSTRUCTION_COUNTER.load(Ordering::SeqCst)
}

/// Reset [`CONSTRUCTION_COUNTER`] to zero.
///
/// Useful for unit tests that want to assert a delta rather than an absolute
/// value, since other tests in the same binary may have already triggered
/// cache initialisations.
#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn reset_construction_counter() {
    CONSTRUCTION_COUNTER.store(0, Ordering::SeqCst);
}

/// Returns the Python class for `spec`, constructing it on first call under
/// the GIL and caching the result in `cell`.
///
/// Concurrent first-call attempts from multiple Python threads holding the
/// GIL are serialised by [`PyOnceLock`], so exactly one construction runs
/// per interpreter per derived type.
pub fn get_or_build<'py>(
    py: Python<'py>,
    cell: &'static PyOnceLock<Py<PyType>>,
    spec: &PyEnumSpec,
) -> PyResult<Bound<'py, PyType>> {
    let class: &Py<PyType> = cell.get_or_try_init(py, || {
        let built = build_py_enum(py, spec)?;
        #[cfg(test)]
        CONSTRUCTION_COUNTER.fetch_add(1, Ordering::SeqCst);
        Ok::<_, PyErr>(built.unbind())
    })?;
    Ok(class.bind(py).clone())
}
