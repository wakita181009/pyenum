//! Core trait + metadata types emitted by `#[derive(PyEnum)]`.
//!
//! User code never implements these by hand — the derive is the only
//! supported entry point. The types are public so the derive's output can
//! name them, and so generic helpers like [`crate::add_enum`] can be bound
//! over `T: PyEnum`.

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
/// base (sequential ints for `Enum`/`IntEnum`, powers of two for
/// `Flag`/`IntFlag`, lowercased name for `StrEnum`).
#[derive(Debug, Clone, Copy)]
pub enum VariantLiteral {
    /// Explicit integer literal from a Rust discriminant (`Variant = 42`).
    Int(i64),
    /// Explicit string literal from `#[pyenum(value = "...")]`.
    Str(&'static str),
    /// Defer to Python's `enum.auto()`.
    Auto,
}

/// Static metadata emitted by `#[derive(PyEnum)]` for each derived enum.
///
/// Stored as `const SPEC: PyEnumSpec` on every `impl PyEnum`, driving class
/// construction ([`crate::add_enum`]) and conversion error messages.
#[derive(Debug, Clone, Copy)]
pub struct PyEnumSpec {
    /// Python class name — defaults to the Rust enum identifier, overridable
    /// via `#[pyenum(name = "...")]`.
    pub name: &'static str,
    /// Chosen Python enum base class.
    pub base: PyEnumBase,
    /// Variants in declaration order.
    pub variants: &'static [(&'static str, VariantLiteral)],
}

/// Bridge between a `#[derive(PyEnum)]` Rust type and its cached Python class.
///
/// Implemented only by the derive. Downstream code interacts via the free
/// helper [`crate::add_enum`] or PyO3's `IntoPyObject` / `FromPyObject`
/// conversions (also emitted by the derive).
pub trait PyEnum: Sized + Copy + 'static {
    /// Static metadata describing the derived type.
    const SPEC: PyEnumSpec;

    /// Returns the cached Python class object, constructing it (exactly once
    /// per interpreter) on first call.
    fn py_enum_class(py: Python) -> PyResult<Bound<PyType>>;

    /// Returns the Python enum member corresponding to `self`.
    ///
    /// Resolves the cached per-variant `Py<PyAny>` and rebinds it to `py`;
    /// no Python-side attribute lookup on the steady-state path.
    fn to_py_member<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;

    /// Extracts `Self` from a Python object that must be a member of the
    /// cached class. Raises `TypeError` with the enum name in the message
    /// for any other object.
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
