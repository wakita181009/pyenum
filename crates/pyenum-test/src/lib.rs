//! cdylib fixture for the Python pytest suite.
//!
//! Registers one derived enum per supported Python base plus a handful of
//! `#[pyfunction]`s that exercise the conversion boundary. Built on-demand
//! by `tests/conftest.py` via `maturin develop`.

use core::sync::atomic::{AtomicUsize, Ordering};

use pyenum::{PyEnum, PyEnumTrait};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyType};

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

/// Per-enum construction counter — incremented every time
/// [`add_enum_counted`] routes through [`PyEnumTrait::py_enum_class`].
///
/// Module init drives a single `add_enum_counted::<T>` call per enum, so a
/// correctly cached class leaves each counter at exactly `1` after module
/// initialisation. Re-entrant calls made by derive-generated conversion code
/// (`to_py_member` / `from_py_member`) go straight to
/// [`PyEnumTrait::py_enum_class`] and do *not* touch this counter, so the
/// cache-hit fast path remains observable from pytest as "count stayed at 1".
static COLOR_COUNT: AtomicUsize = AtomicUsize::new(0);
static HTTP_STATUS_COUNT: AtomicUsize = AtomicUsize::new(0);
static GREETING_COUNT: AtomicUsize = AtomicUsize::new(0);
static LANGUAGE_COUNT: AtomicUsize = AtomicUsize::new(0);
static PERMISSION_COUNT: AtomicUsize = AtomicUsize::new(0);
static BITPERMS_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Associate a Rust enum type `T` with the counter registered for it during
/// module init.
fn counter_for<T: PyEnumTrait>() -> &'static AtomicUsize {
    match T::SPEC.name {
        "Color" => &COLOR_COUNT,
        "HttpStatus" => &HTTP_STATUS_COUNT,
        "Greeting" => &GREETING_COUNT,
        "Language" => &LANGUAGE_COUNT,
        "Permission" => &PERMISSION_COUNT,
        "BitPerms" => &BITPERMS_COUNT,
        other => panic!("pyenum_test: no counter registered for `{other}`"),
    }
}

/// Resolve and register the Python class for `T`, counting each trip through
/// [`PyEnumTrait::py_enum_class`].
///
/// The derive's cache keeps this to exactly one actual construction; the
/// counter therefore records how many times module init crossed the
/// `py_enum_class` boundary, which should be `1` after the `#[pymodule]`
/// function returns.
fn add_enum_counted<T: PyEnumTrait>(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    let class = T::py_enum_class(py)?;
    counter_for::<T>().fetch_add(1, Ordering::SeqCst);
    module.add(T::SPEC.name, class)
}

/// Read the construction counter for `cls`.
///
/// Looks up the counter by matching `cls` against the Python class object
/// cached for each known Rust enum. Raises `TypeError` if `cls` is not one of
/// the classes registered by this module.
#[pyfunction]
#[pyo3(name = "_construction_count")]
fn construction_count(cls: &Bound<'_, PyType>) -> PyResult<usize> {
    let py = cls.py();
    let candidates: [(Bound<'_, PyType>, &AtomicUsize); 6] = [
        (Color::py_enum_class(py)?, &COLOR_COUNT),
        (HttpStatus::py_enum_class(py)?, &HTTP_STATUS_COUNT),
        (Greeting::py_enum_class(py)?, &GREETING_COUNT),
        (Language::py_enum_class(py)?, &LANGUAGE_COUNT),
        (Permission::py_enum_class(py)?, &PERMISSION_COUNT),
        (BitPerms::py_enum_class(py)?, &BITPERMS_COUNT),
    ];
    for (registered, counter) in &candidates {
        if cls.is(registered) {
            return Ok(counter.load(Ordering::SeqCst));
        }
    }
    Err(PyTypeError::new_err(
        "_construction_count: class is not registered by pyenum_test",
    ))
}

#[pymodule]
fn pyenum_test(m: &Bound<'_, PyModule>) -> PyResult<()> {
    add_enum_counted::<Color>(m)?;
    add_enum_counted::<HttpStatus>(m)?;
    add_enum_counted::<Greeting>(m)?;
    add_enum_counted::<Language>(m)?;
    add_enum_counted::<Permission>(m)?;
    add_enum_counted::<BitPerms>(m)?;
    m.add_function(wrap_pyfunction!(color_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(http_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(greeting_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(language_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(permission_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(bitperms_roundtrip, m)?)?;
    m.add_function(wrap_pyfunction!(construction_count, m)?)?;
    Ok(())
}
