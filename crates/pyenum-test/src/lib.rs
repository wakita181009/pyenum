//! cdylib fixture for the Python pytest suite.
//!
//! Registers one derived enum per supported Python base plus edge-case
//! fixtures, and helper `#[pyfunction]`s that exercise the conversion
//! boundary. Built on-demand by `tests/conftest.py` via `maturin develop`.

use pyenum::{PyEnum, PyModuleExt};
use pyo3::prelude::*;
use pyo3::types::PyModule;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[pyfunction]
fn color_roundtrip(c: Color) -> Color {
    c
}

#[pymodule]
fn pyenum_test<'py>(m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_enum::<Color>()?;
    m.add_function(wrap_pyfunction!(color_roundtrip, m)?)?;
    Ok(())
}
