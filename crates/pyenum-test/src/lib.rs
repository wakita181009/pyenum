//! cdylib fixture for the Python pytest suite.
//!
//! Registers one derived enum per supported Python base plus a handful of
//! `#[pyfunction]`s that exercise the conversion boundary. Built on-demand
//! by `tests/conftest.py` via `maturin develop`.

use pyenum::{PyEnum, PyModuleExt};
use pyo3::prelude::*;
use pyo3::types::PyModule;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
#[pyenum(base = "IntEnum")]
pub enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    Teapot = 418,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Greeting {
    Hello,
    World,
    Bye,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Language {
    #[pyenum(value = "Rust")]
    Rust,
    #[pyenum(value = "Python")]
    Python,
    #[pyenum(value = "TypeScript")]
    TypeScript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
#[pyenum(base = "Flag")]
pub enum Permission {
    Read = 1,
    Write = 2,
    Execute = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
#[pyenum(base = "IntFlag")]
pub enum BitPerms {
    Read = 1,
    Write = 2,
    Execute = 4,
    Admin = 8,
}

#[pyfunction]
fn color_roundtrip(c: Color) -> Color {
    c
}

#[pyfunction]
fn http_roundtrip(s: HttpStatus) -> HttpStatus {
    s
}

#[pyfunction]
fn greeting_roundtrip(g: Greeting) -> Greeting {
    g
}

#[pyfunction]
fn language_roundtrip(l: Language) -> Language {
    l
}

#[pyfunction]
fn permission_roundtrip(p: Permission) -> Permission {
    p
}

#[pyfunction]
fn bitperms_roundtrip(p: BitPerms) -> BitPerms {
    p
}

#[pymodule]
fn pyenum_test(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_enum::<Color>()?;
    m.add_enum::<HttpStatus>()?;
    m.add_enum::<Greeting>()?;
    m.add_enum::<Language>()?;
    m.add_enum::<Permission>()?;
    m.add_enum::<BitPerms>()?;
    m.add_function(wrap_pyfunction!(color_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(http_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(greeting_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(language_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(permission_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(bitperms_roundtrip, m)?)?;
    Ok(())
}
