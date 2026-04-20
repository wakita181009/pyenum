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
                         but the enum base is `{base}`, which requires \
                         integer values",
                        variant.rust_ident,
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
    match spec.base {
        BaseSelector::StrEnum => check_duplicate_str_values(spec),
        BaseSelector::Enum | BaseSelector::IntEnum | BaseSelector::Flag | BaseSelector::IntFlag => {
            check_duplicate_int_values(spec)
        }
    }
}

fn check_duplicate_str_values(spec: &DeriveSpec) -> Result<()> {
    let mut seen: HashMap<String, String> = HashMap::new();
    for variant in &spec.variants {
        let variant_name = variant.rust_ident.to_string();
        let (key, is_auto) = match &variant.value {
            VariantValue::Str(s) => (s.clone(), false),
            VariantValue::Auto => (variant_name.to_lowercase(), true),
            // Integer discriminants are rejected by `check_base_value_compatibility`.
            VariantValue::Int(_) => continue,
        };
        if let Some(prev) = seen.get(&key) {
            let detail = if is_auto {
                format!(
                    "variant `{variant_name}` auto-lowercases to `{key:?}`, \
                     which was already used by `{prev}`; Python would make \
                     the second variant an alias of the first and break \
                     Rust-side round-trip identity (add an explicit \
                     `#[pyenum(value = \"...\")]` to disambiguate)"
                )
            } else {
                format!(
                    "variant `{variant_name}` has value `{key:?}`, which \
                     was already used by `{prev}`; Python would make the \
                     second variant an alias of the first and break \
                     Rust-side round-trip identity"
                )
            };
            return Err(syn::Error::new(variant.rust_ident.span(), detail));
        }
        seen.insert(key, variant_name);
    }
    Ok(())
}

fn check_duplicate_int_values(spec: &DeriveSpec) -> Result<()> {
    let mut seen: HashMap<i64, String> = HashMap::new();
    // Python's `auto()` for integer-shaped bases is driven by the *last*
    // assigned value (for Flag/IntFlag: `bit_length(last) + 1`; for
    // Enum/IntEnum: `last + 1`). Track it as we walk the variants in
    // declaration order so we can diagnose auto↔explicit collisions the
    // same way Python's functional `enum.*` constructor resolves them.
    let mut last: Option<i64> = None;
    for variant in &spec.variants {
        let variant_name = variant.rust_ident.to_string();
        let (value, is_auto) = match &variant.value {
            VariantValue::Int(x) => (*x, false),
            VariantValue::Auto => (auto_int_for_base(spec.base, last), true),
            // String values are rejected by `check_base_value_compatibility`.
            VariantValue::Str(_) => continue,
        };
        if let Some(prev) = seen.get(&value) {
            let detail = if is_auto {
                format!(
                    "variant `{variant_name}`'s `auto()` resolves to \
                     `{value}`, which was already used by `{prev}`; Python \
                     would make the second variant an alias of the first \
                     and break Rust-side round-trip identity (give one \
                     variant an explicit discriminant to disambiguate)"
                )
            } else {
                format!(
                    "variant `{variant_name}` has discriminant `{value}`, \
                     which was already used by `{prev}`; Python would make \
                     the second variant an alias of the first and break \
                     Rust-side round-trip identity"
                )
            };
            return Err(syn::Error::new(variant.rust_ident.span(), detail));
        }
        seen.insert(value, variant_name);
        last = Some(value);
    }
    Ok(())
}

/// Mirror CPython's `_generate_next_value_` for integer-shaped enum bases.
///
/// * `Enum` / `IntEnum`: `last + 1` (start = 1 when no prior value exists).
/// * `Flag` / `IntFlag`: `1 << last.bit_length()` — the next power of two
///   strictly above the last assigned value (1 when no prior value exists).
///
/// Negative `last` values fall back to `1`; CPython's Flag machinery rejects
/// them at class-construction time, so the exact Rust-side return value here
/// only matters for collision detection against other explicit values.
fn auto_int_for_base(base: BaseSelector, last: Option<i64>) -> i64 {
    match base {
        BaseSelector::Enum | BaseSelector::IntEnum => last.map_or(1, |n| n.saturating_add(1)),
        BaseSelector::Flag | BaseSelector::IntFlag => match last {
            None => 1,
            Some(n) if n <= 0 => 1,
            Some(n) => {
                let bit_len = u64::BITS - (n as u64).leading_zeros();
                if bit_len >= i64::BITS - 1 {
                    i64::MAX
                } else {
                    1_i64 << bit_len
                }
            }
        },
        BaseSelector::StrEnum => unreachable!("string-shaped base routed to int path"),
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
            python_module: None,
            python_qualname: None,
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
    fn rejects_auto_colliding_with_explicit_intenum() {
        // `A` auto-resolves to 1; `B = 1` collides and would be aliased by
        // CPython's `enum` machinery.
        let s = spec(
            BaseSelector::IntEnum,
            vec![("A", VariantValue::Auto), ("B", VariantValue::Int(1))],
        );
        let err = run_err(s);
        assert!(err.contains("`B`"), "{err}");
        assert!(err.contains("already used by `A`"), "{err}");
        assert!(err.contains("alias"), "{err}");
    }

    #[test]
    fn rejects_explicit_colliding_with_auto_intenum() {
        // `A = 5`, `B` auto -> 6, `C = 6` collides with `B`.
        let s = spec(
            BaseSelector::IntEnum,
            vec![
                ("A", VariantValue::Int(5)),
                ("B", VariantValue::Auto),
                ("C", VariantValue::Int(6)),
            ],
        );
        let err = run_err(s);
        assert!(err.contains("`C`"), "{err}");
        assert!(err.contains("already used by `B`"), "{err}");
    }

    #[test]
    fn rejects_auto_colliding_with_explicit_flag() {
        // `Read` auto -> 1; `Write = 1` collides.
        let s = spec(
            BaseSelector::Flag,
            vec![
                ("Read", VariantValue::Auto),
                ("Write", VariantValue::Int(1)),
            ],
        );
        let err = run_err(s);
        assert!(err.contains("`Write`"), "{err}");
        assert!(err.contains("already used by `Read`"), "{err}");
    }

    #[test]
    fn rejects_auto_resolving_to_prior_explicit_flag() {
        // `A = 4`, `B = 2`, `C` auto. Python's Flag auto doubles the last
        // value's high bit: bit_length(2) = 2, so C -> 2^2 = 4, collides with A.
        let s = spec(
            BaseSelector::Flag,
            vec![
                ("A", VariantValue::Int(4)),
                ("B", VariantValue::Int(2)),
                ("C", VariantValue::Auto),
            ],
        );
        let err = run_err(s);
        assert!(err.contains("`C`"), "{err}");
        assert!(err.contains("already used by `A`"), "{err}");
    }

    #[test]
    fn accepts_all_auto_intflag_power_of_two() {
        // Classic Flag pattern: every variant auto -> 1, 2, 4, 8. All distinct.
        let s = spec(
            BaseSelector::IntFlag,
            vec![
                ("Read", VariantValue::Auto),
                ("Write", VariantValue::Auto),
                ("Exec", VariantValue::Auto),
                ("Admin", VariantValue::Auto),
            ],
        );
        assert!(run(s).is_ok());
    }

    #[test]
    fn accepts_explicit_then_auto_intenum() {
        // `A = 10`, `B` auto -> 11, `C` auto -> 12. No collision.
        let s = spec(
            BaseSelector::IntEnum,
            vec![
                ("A", VariantValue::Int(10)),
                ("B", VariantValue::Auto),
                ("C", VariantValue::Auto),
            ],
        );
        assert!(run(s).is_ok());
    }

    #[test]
    fn base_selector_converts_to_python_enum_names() {
        assert_eq!(<&'static str>::from(BaseSelector::Enum), "Enum");
        assert_eq!(<&'static str>::from(BaseSelector::IntEnum), "IntEnum");
        assert_eq!(<&'static str>::from(BaseSelector::StrEnum), "StrEnum");
        assert_eq!(<&'static str>::from(BaseSelector::Flag), "Flag");
        assert_eq!(<&'static str>::from(BaseSelector::IntFlag), "IntFlag");
        assert_eq!(BaseSelector::IntFlag.to_string(), "IntFlag");
    }

    #[test]
    fn base_selector_parses_from_str() {
        use std::str::FromStr;
        assert_eq!(
            BaseSelector::from_str("IntEnum").unwrap(),
            BaseSelector::IntEnum
        );
        let err = BaseSelector::from_str("Nope").unwrap_err();
        assert!(err.contains("unknown pyenum base `Nope`"), "{err}");
        assert!(err.contains("expected one of"), "{err}");
    }
}
