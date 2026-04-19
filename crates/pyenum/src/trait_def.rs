//! Core trait + metadata types emitted by `#[derive(PyEnum)]`.

use pyo3::prelude::*;
use pyo3::types::PyType;

/// Python enum base type selector.
///
/// Chosen per derive via `#[pyenum(base = "...")]`; defaults to
/// [`PyEnumBase::Enum`]. The variant name matches the Python class name
/// exactly — no translation layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyEnumBase {
    Enum,
    IntEnum,
    StrEnum,
    Flag,
    IntFlag,
}

impl PyEnumBase {
    /// The Python attribute name on the `enum` module that exposes this base.
    pub const fn class_name(self) -> &'static str {
        match self {
            PyEnumBase::Enum => "Enum",
            PyEnumBase::IntEnum => "IntEnum",
            PyEnumBase::StrEnum => "StrEnum",
            PyEnumBase::Flag => "Flag",
            PyEnumBase::IntFlag => "IntFlag",
        }
    }
}

/// A single variant's declared value, ready to be materialised into a
/// Python-side `(name, value)` tuple for the functional `enum.*` constructor.
///
/// `Auto` defers value resolution to CPython's `enum.auto()` so behaviour
/// tracks whatever rules the host Python version enforces for the chosen
/// base.
#[derive(Debug, Clone, Copy)]
pub enum VariantLiteral {
    /// Explicit integer literal (from a Rust discriminant).
    Int(i64),
    /// Explicit string literal.
    ///
    /// Reserved for future use; the v1 derive never emits this variant.
    Str(&'static str),
    /// Defer to Python's `enum.auto()` per-base semantics.
    Auto,
}

/// Static metadata emitted by `#[derive(PyEnum)]` for each derived enum.
#[derive(Debug, Clone, Copy)]
pub struct PyEnumSpec {
    /// Python class name (defaults to the Rust enum's identifier;
    /// overridable via `#[pyenum(name = "...")]`).
    pub name: &'static str,
    /// Chosen Python enum base class.
    pub base: PyEnumBase,
    /// Ordered variants, preserving declaration order.
    pub variants: &'static [(&'static str, VariantLiteral)],
}

/// Bridge between a `#[derive(PyEnum)]` Rust type and its cached Python class.
///
/// User code never implements this trait by hand — the derive is the only
/// supported path.
pub trait PyEnum: Sized + Copy + 'static {
    /// Static metadata describing the derived type.
    const SPEC: PyEnumSpec;

    /// Returns the cached Python class object, constructing it (exactly once
    /// per interpreter) on first call.
    fn py_enum_class(py: Python) -> PyResult<Bound<PyType>>;

    /// Returns the Python enum member corresponding to `self`.
    fn to_py_member<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;

    /// Extracts `Self` from a Python object that must be a member of the
    /// cached class. Raises `TypeError` otherwise.
    fn from_py_member(obj: &Bound<PyAny>) -> PyResult<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_name_matches_python_enum_module_attrs() {
        assert_eq!(PyEnumBase::Enum.class_name(), "Enum");
        assert_eq!(PyEnumBase::IntEnum.class_name(), "IntEnum");
        assert_eq!(PyEnumBase::StrEnum.class_name(), "StrEnum");
        assert_eq!(PyEnumBase::Flag.class_name(), "Flag");
        assert_eq!(PyEnumBase::IntFlag.class_name(), "IntFlag");
    }
}
