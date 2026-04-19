//! Identity-level validation that runs over the parsed IR.
//!
//! Enforces:
//!
//! 1. Reserved-name rejection (`name`, `value`, `__init__`, …).
//! 2. Base/value compatibility — integer discriminants on string-shaped
//!    bases (and vice versa) are rejected at compile time.
//! 3. Duplicate-value rejection — two variants that would hit the same
//!    Python value would become aliases and break Rust-side variant
//!    identity on round-trip. Rust itself already forbids duplicate
//!    integer discriminants, but explicit `#[pyenum(value = "...")]`
//!    strings (and auto-lowercased `StrEnum` names) can still collide.

use std::collections::HashMap;

use crate::parse::{BaseSelector, DeriveSpec, VariantValue};
use crate::reserved::{ReservedKind, is_reserved};
use syn::Result;

/// Runs every identity-level check. Returns the spec unchanged on success,
/// or the first diagnostic error encountered.
pub(crate) fn run(spec: DeriveSpec) -> Result<DeriveSpec> {
    check_reserved_names(&spec)?;
    check_base_value_compatibility(&spec)?;
    check_duplicate_values(&spec)?;
    Ok(spec)
}

fn check_reserved_names(spec: &DeriveSpec) -> Result<()> {
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
    Ok(())
}

fn check_base_value_compatibility(spec: &DeriveSpec) -> Result<()> {
    let base = spec.base;
    for variant in &spec.variants {
        match (&variant.value, base) {
            // String value on an integer-shaped base.
            (
                VariantValue::Str(_),
                BaseSelector::IntEnum | BaseSelector::Flag | BaseSelector::IntFlag,
            ) => {
                return Err(syn::Error::new(
                    variant.rust_ident.span(),
                    format!(
                        "variant `{}` has a string `#[pyenum(value = ...)]` \
                         but the enum base is `{}`, which requires integer \
                         values",
                        variant.rust_ident,
                        base_display(base),
                    ),
                ));
            }
            // Integer discriminant on the string-shaped base.
            (VariantValue::Int(_), BaseSelector::StrEnum) => {
                return Err(syn::Error::new(
                    variant.rust_ident.span(),
                    format!(
                        "variant `{}` has an integer discriminant but the \
                         enum base is `StrEnum`, which requires string \
                         values (use `#[pyenum(value = \"...\")]` or omit \
                         the discriminant for auto-lowercased names)",
                        variant.rust_ident,
                    ),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_duplicate_values(spec: &DeriveSpec) -> Result<()> {
    let mut seen_ints: HashMap<i64, String> = HashMap::new();
    let mut seen_strs: HashMap<String, String> = HashMap::new();

    for variant in &spec.variants {
        let variant_name = variant.rust_ident.to_string();
        match &variant.value {
            VariantValue::Int(v) => {
                if let Some(prev) = seen_ints.get(v) {
                    return Err(syn::Error::new(
                        variant.rust_ident.span(),
                        format!(
                            "variant `{}` has discriminant `{}`, which was \
                             already used by `{}`; Python would make the \
                             second variant an alias of the first and \
                             break Rust-side round-trip identity",
                            variant_name, v, prev,
                        ),
                    ));
                }
                seen_ints.insert(*v, variant_name);
            }
            VariantValue::Str(s) => {
                let normalized = s.clone();
                if let Some(prev) = seen_strs.get(&normalized) {
                    return Err(syn::Error::new(
                        variant.rust_ident.span(),
                        format!(
                            "variant `{}` has value `{:?}`, which was \
                             already used by `{}`; Python would make the \
                             second variant an alias of the first and \
                             break Rust-side round-trip identity",
                            variant_name, s, prev,
                        ),
                    ));
                }
                seen_strs.insert(normalized, variant_name);
            }
            VariantValue::Auto => {
                // For `StrEnum`, Python's `auto()` lowercases the variant
                // name. Check for collisions against explicit string
                // values and other auto-lowercased names in the same
                // enum. Other bases resolve `auto()` to deterministic
                // sequences (ints / powers-of-two) that cannot collide
                // with peer auto values, and Rust forbids explicit
                // duplicate integer discriminants, so we skip the
                // non-StrEnum case.
                if spec.base == BaseSelector::StrEnum {
                    let lowered = variant_name.to_lowercase();
                    if let Some(prev) = seen_strs.get(&lowered) {
                        return Err(syn::Error::new(
                            variant.rust_ident.span(),
                            format!(
                                "variant `{}` auto-lowercases to `{:?}`, \
                                 which was already used by `{}`; Python \
                                 would make the second variant an alias \
                                 of the first and break Rust-side \
                                 round-trip identity (add an explicit \
                                 `#[pyenum(value = \"...\")]` to \
                                 disambiguate)",
                                variant_name, lowered, prev,
                            ),
                        ));
                    }
                    seen_strs.insert(lowered, variant_name);
                }
            }
        }
    }
    Ok(())
}

fn base_display(base: BaseSelector) -> &'static str {
    match base {
        BaseSelector::Enum => "Enum",
        BaseSelector::IntEnum => "IntEnum",
        BaseSelector::StrEnum => "StrEnum",
        BaseSelector::Flag => "Flag",
        BaseSelector::IntFlag => "IntFlag",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{DeriveSpec, VariantSpec, VariantValue};
    use proc_macro2::Span;
    use syn::Ident;

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    fn run_err(s: DeriveSpec) -> String {
        match run(s) {
            Ok(_) => panic!("expected validation error"),
            Err(e) => e.to_string(),
        }
    }

    fn spec(base: BaseSelector, variants: Vec<(&str, VariantValue)>) -> DeriveSpec {
        DeriveSpec {
            rust_ident: ident("TestEnum"),
            python_name: "TestEnum".to_string(),
            base,
            variants: variants
                .into_iter()
                .map(|(name, value)| VariantSpec {
                    rust_ident: ident(name),
                    value,
                    span: Span::call_site(),
                })
                .collect(),
        }
    }

    #[test]
    fn rejects_python_keyword_variant() {
        // `class` is a Python keyword.
        let s = spec(BaseSelector::Enum, vec![("class", VariantValue::Auto)]);
        let err = run_err(s);
        assert!(err.contains("collides with a Python keyword"), "{err}");
    }

    #[test]
    fn rejects_enum_reserved_member_variant() {
        // `name` is reserved by `enum.Enum`.
        let s = spec(BaseSelector::Enum, vec![("name", VariantValue::Auto)]);
        let err = run_err(s);
        assert!(
            err.contains("collides with an `enum`-reserved member name"),
            "{err}"
        );
    }

    #[test]
    fn rejects_enum_special_method_variant() {
        // `__init__` is an enum special method.
        let s = spec(BaseSelector::Enum, vec![("__init__", VariantValue::Auto)]);
        let err = run_err(s);
        assert!(
            err.contains("collides with an `enum` special method name"),
            "{err}"
        );
    }

    #[test]
    fn rejects_duplicate_int_discriminants() {
        // Rust itself rejects duplicate discriminants at the surface level,
        // but the validator is defense-in-depth — invoke it directly.
        let s = spec(
            BaseSelector::IntEnum,
            vec![("A", VariantValue::Int(1)), ("B", VariantValue::Int(1))],
        );
        let err = run_err(s);
        assert!(err.contains("already used by `A`"), "{err}");
        assert!(err.contains("alias"), "{err}");
    }

    #[test]
    fn rejects_duplicate_str_values() {
        let s = spec(
            BaseSelector::StrEnum,
            vec![
                ("A", VariantValue::Str("red".into())),
                ("B", VariantValue::Str("red".into())),
            ],
        );
        let err = run_err(s);
        assert!(err.contains("already used by `A`"), "{err}");
    }

    #[test]
    fn str_value_on_intenum_is_rejected() {
        let s = spec(
            BaseSelector::IntEnum,
            vec![("A", VariantValue::Str("x".into()))],
        );
        let err = run_err(s);
        assert!(err.contains("IntEnum"), "{err}");
        assert!(err.contains("requires integer values"), "{err}");
    }

    #[test]
    fn str_value_on_flag_is_rejected() {
        let s = spec(
            BaseSelector::Flag,
            vec![("A", VariantValue::Str("x".into()))],
        );
        let err = run_err(s);
        assert!(err.contains("Flag"), "{err}");
    }

    #[test]
    fn str_value_on_intflag_is_rejected() {
        let s = spec(
            BaseSelector::IntFlag,
            vec![("A", VariantValue::Str("x".into()))],
        );
        let err = run_err(s);
        assert!(err.contains("IntFlag"), "{err}");
    }

    #[test]
    fn int_discriminant_on_strenum_is_rejected() {
        let s = spec(BaseSelector::StrEnum, vec![("A", VariantValue::Int(1))]);
        let err = run_err(s);
        assert!(err.contains("StrEnum"), "{err}");
    }

    #[test]
    fn auto_strenum_collides_with_lowercased_peer() {
        // `RED` auto-lowercases to `"red"`, collides with explicit value.
        let s = spec(
            BaseSelector::StrEnum,
            vec![
                ("red", VariantValue::Str("red".into())),
                ("RED", VariantValue::Auto),
            ],
        );
        let err = run_err(s);
        assert!(err.contains("auto-lowercases"), "{err}");
    }

    #[test]
    fn accepts_well_formed_spec() {
        let s = spec(
            BaseSelector::IntEnum,
            vec![
                ("A", VariantValue::Int(1)),
                ("B", VariantValue::Int(2)),
                ("C", VariantValue::Auto),
            ],
        );
        assert!(run(s).is_ok());
    }

    #[test]
    fn base_display_covers_every_variant() {
        for b in [
            BaseSelector::Enum,
            BaseSelector::IntEnum,
            BaseSelector::StrEnum,
            BaseSelector::Flag,
            BaseSelector::IntFlag,
        ] {
            let name = base_display(b);
            assert!(!name.is_empty());
        }
    }
}
