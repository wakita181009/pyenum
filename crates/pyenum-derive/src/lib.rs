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
/// Attributes (all optional):
///
/// * `#[pyenum(base = "Enum" | "IntEnum" | "StrEnum" | "Flag" | "IntFlag")]`
///   — select the Python base class. Defaults to `"Enum"`.
/// * `#[pyenum(name = "...")]` — override the Python class name. Defaults
///   to the Rust enum identifier.
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
