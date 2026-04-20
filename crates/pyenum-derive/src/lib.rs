//! `#[derive(PyEnum)]` procedural macro.
//!
//! Parsing, validation, and codegen live in the submodules and are wired
//! together by [`derive_pyenum`]. The generated code only names items
//! re-exported from `pyenum::__private`, so the output is stable across
//! internal refactors of the runtime crate.
//!
//! See the [`pyenum`](https://docs.rs/pyenum) crate docs for usage,
//! worked examples, and scope.

use proc_macro::TokenStream;

mod codegen;
mod parse;
mod reserved;
mod validate;

/// Derive a [`pyenum::PyEnum`](../pyenum/trait.PyEnumTrait.html)
/// implementation for a unit-variant Rust enum.
///
/// The derive also emits PyO3 0.28 `IntoPyObject<'py>` (for `T` and `&T`)
/// and `FromPyObject<'a, 'py>` impls, so the Rust enum can appear directly
/// in `#[pyfunction]`, `#[pymethods]`, and `#[pyclass]` field signatures.
///
/// # Attributes
///
/// Enum-level (all optional):
///
/// * `#[pyenum(base = "Enum" | "IntEnum" | "StrEnum" | "Flag" | "IntFlag")]`
///   — select the Python base class. Defaults to `"Enum"`.
/// * `#[pyenum(name = "...")]` — override the Python class name. Defaults
///   to the Rust enum identifier.
///
/// Variant-level (optional, mutually exclusive with a Rust discriminant):
///
/// * `#[pyenum(value = "...")]` — explicit Python string value. Valid on
///   `StrEnum` (and `Enum`). Without it, `StrEnum` variants default to
///   Python's `auto()` semantics, which lowercase the variant name.
///
/// # Example
///
/// ```rust,ignore
/// use pyenum::{PyEnum, PyModuleExt};
/// use pyo3::prelude::*;
///
/// #[derive(Clone, Copy, PyEnum)]
/// #[pyenum(base = "IntEnum")]
/// pub enum HttpStatus {
///     Ok = 200,
///     NotFound = 404,
/// }
///
/// #[pyfunction]
/// fn classify(s: HttpStatus) -> HttpStatus { s }
///
/// #[pymodule]
/// fn demo(m: &Bound<'_, PyModule>) -> PyResult<()> {
///     m.add_enum::<HttpStatus>()?;
///     m.add_function(wrap_pyfunction!(classify, m)?)
/// }
/// ```
///
/// # Rejected at compile time
///
/// The macro emits spanned `compile_error!` diagnostics for:
///
/// * Tuple / struct variants, generics, lifetimes, and empty enums.
/// * Variant names colliding with Python keywords, dunders, or
///   `enum.Enum`-reserved attributes.
/// * Base/value shape mismatches (e.g. `#[pyenum(value = "x")]` on
///   `IntEnum`, or an integer discriminant on `StrEnum`).
/// * Duplicate resolved values — includes the `auto()` vs. explicit
///   collision on every integer-shaped base (`enum { A, B = 1 }` with
///   base `IntEnum`, `enum { Read, Write = 1 }` with base `Flag`, etc.)
///   and auto-lowercased name collisions on `StrEnum` (`Hello` + `HELLO`).
/// * Duplicate or unknown `#[pyenum(...)]` keys.
#[proc_macro_derive(PyEnum, attributes(pyenum))]
pub fn derive_pyenum(input: TokenStream) -> TokenStream {
    let spec = match parse::parse_derive_input(input.into()) {
        Ok(spec) => spec,
        Err(err) => return err.to_compile_error().into(),
    };
    let spec = match validate::run(spec) {
        Ok(spec) => spec,
        Err(err) => return err.to_compile_error().into(),
    };
    codegen::emit(&spec).into()
}
