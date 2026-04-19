//! `#[derive(PyEnum)]` procedural macro.
//!
//! This is the entry-point crate. Parsing, validation, and codegen live in
//! the submodules and are wired together by [`derive_pyenum`] below. The
//! generated code references only items re-exported from `pyenum::__private`
//! so the output is feature-flag-free — the runtime crate's `compat` module
//! resolves those names per the active `pyo3-0_XX` feature.

use proc_macro::TokenStream;

mod codegen;
mod parse;
mod reserved;
mod validate;

/// Derive a [`pyenum::PyEnum`] implementation for a unit-variant Rust enum.
///
/// Enum-level attributes (all optional):
///
/// * `#[pyenum(base = "Enum" | "IntEnum" | "StrEnum" | "Flag" | "IntFlag")]`
///   — select the Python base class. Defaults to `"Enum"`.
/// * `#[pyenum(name = "...")]` — override the Python class name. Defaults
///   to the Rust enum identifier.
///
/// Variant-level attributes (all optional, mutually exclusive with a Rust
/// discriminant on the same variant):
///
/// * `#[pyenum(value = "...")]` — explicit Python string value. Only
///   valid when the enum base is `StrEnum` or `Enum`. Without this
///   attribute (and without a Rust discriminant), `StrEnum` variants
///   default to Python's `auto()` semantics, which lowercase the variant
///   name.
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
