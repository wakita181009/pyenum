//! Identity-level validation that runs over the parsed IR.
//!
//! Currently enforces the reserved-name check. Future work: base/value
//! literal mismatch, attribute-surface tightening, and stacking with other
//! PyO3 attributes such as `#[pyclass]`.

use crate::parse::DeriveSpec;
use crate::reserved::{ReservedKind, is_reserved};
use syn::Result;

/// Runs every identity-level check. Returns the spec unchanged on success,
/// or the first diagnostic error encountered.
pub(crate) fn run(spec: DeriveSpec) -> Result<DeriveSpec> {
    for variant in &spec.variants {
        if let Some(kind) = is_reserved(&variant.rust_ident.to_string()) {
            let category = match kind {
                ReservedKind::PythonKeyword => "a Python keyword",
                ReservedKind::EnumReservedMember => "an `enum`-reserved member name",
                ReservedKind::EnumSpecialMethod => "an `enum` special method name",
            };
            return Err(syn::Error::new(
                variant.rust_ident.span(),
                format!(
                    "variant `{}` collides with {category}; \
                     rename the Rust variant (future `#[pyenum(rename = \
                     \"...\")]` may offer an opt-out path)",
                    variant.rust_ident
                ),
            ));
        }
    }
    Ok(spec)
}
