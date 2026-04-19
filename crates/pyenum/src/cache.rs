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
        Ok::<_, PyErr>(built.unbind())
    })?;
    Ok(class.bind(py).clone())
}
