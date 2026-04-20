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
///
/// Triggers a one-time class construction on the first call per interpreter
/// per enum type (subsequent calls reuse the cached class). Prefer the
/// extension-method form [`PyModuleExt::add_enum`] inside `#[pymodule]`
/// bodies; this free function exists for call sites that want an explicit
/// type parameter.
///
/// If `module` already has an attribute with the same name, it is silently
/// replaced — consistent with `PyModule::add` and `m.add_class::<T>()?`.
/// Avoid registering two enums whose `#[pyenum(name = "...")]` resolves to
/// the same string within a single module.
///
/// ```rust,ignore
/// use pyenum::{PyEnum, add_enum};
/// use pyo3::prelude::*;
///
/// #[derive(Clone, Copy, PyEnum)]
/// pub enum Color { Red, Green, Blue }
///
/// #[pymodule]
/// fn demo(m: &Bound<'_, PyModule>) -> PyResult<()> {
///     add_enum::<Color>(m)
/// }
/// ```
pub fn add_enum<T: PyEnum>(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    let class = T::py_enum_class(py)?;
    module.add(T::SPEC.name, class)
}

/// Extension method for `Bound<'_, PyModule>` — the idiomatic way to
/// register a derived enum inside a `#[pymodule]` body.
///
/// ```rust,ignore
/// use pyenum::{PyEnum, PyModuleExt};
/// use pyo3::prelude::*;
///
/// #[derive(Clone, Copy, PyEnum)]
/// pub enum Color { Red, Green, Blue }
///
/// #[pymodule]
/// fn demo(m: &Bound<'_, PyModule>) -> PyResult<()> {
///     m.add_enum::<Color>()
/// }
/// ```
pub trait PyModuleExt {
    /// Register the Python class for `T` on `self` under `T::SPEC.name`.
    fn add_enum<T: PyEnum>(&self) -> PyResult<()>;
}

impl PyModuleExt for Bound<'_, PyModule> {
    fn add_enum<T: PyEnum>(&self) -> PyResult<()> {
        add_enum::<T>(self)
    }
}
