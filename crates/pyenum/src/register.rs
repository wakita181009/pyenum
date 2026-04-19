//! Register derived enums into a `#[pymodule]`.
//!
//! The only supported registration path is an explicit library-provided
//! helper — either the free [`add_enum`] function or the
//! [`PyModuleExt::add_enum`] extension method. No hidden global registry,
//! no replacement module macro.

use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::trait_def::PyEnum;

/// Register the Python class for `T` onto `module` under `T::SPEC.name`.
pub fn add_enum<T: PyEnum>(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    let class = T::py_enum_class(py)?;
    module.add(T::SPEC.name, class)
}

/// Extension-method form of [`add_enum`] — `m.add_enum::<T>()?` inside a
/// `#[pymodule]` is the idiomatic call site.
pub trait PyModuleExt {
    fn add_enum<T: PyEnum>(&self) -> PyResult<()>;
}

impl PyModuleExt for Bound<'_, PyModule> {
    fn add_enum<T: PyEnum>(&self) -> PyResult<()> {
        add_enum::<T>(self)
    }
}
