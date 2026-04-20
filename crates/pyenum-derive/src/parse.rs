//! Parse a `#[derive(PyEnum)]` input into an internal IR.
//!
//! The IR (`DeriveSpec`, `VariantSpec`, `VariantValue`) is fed to
//! [`crate::codegen`]. Parsing performs shape-level validation only
//! (unit-variant enforcement, generics/lifetime rejection, attribute surface
//! shape). Identity-level rejection (reserved names, base/value mismatch)
//! happens in [`crate::validate`].

use std::fmt;
use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, ExprLit, ExprUnary, Fields, Ident, Lit, LitInt,
    LitStr, Result, UnOp, Variant,
};

/// Resolved `#[pyenum(...)]` attribute + enum metadata.
pub(crate) struct DeriveSpec {
    /// The Rust enum identifier.
    pub rust_ident: Ident,
    /// Python class name (defaults to `rust_ident.to_string()`).
    pub python_name: String,
    /// Python base class selector.
    pub base: BaseSelector,
    /// Declaration-order variants.
    pub variants: Vec<VariantSpec>,
    /// `#[pyenum(module = "...")]` — written into `__module__` for pickle.
    /// `None` when unset; the runtime crate emits no `module=` kwarg.
    pub python_module: Option<String>,
    /// `#[pyenum(qualname = "...")]` — written into `__qualname__`. `None`
    /// means CPython uses the class name as the default qualname.
    pub python_qualname: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BaseSelector {
    Enum,
    IntEnum,
    StrEnum,
    Flag,
    IntFlag,
}

impl BaseSelector {
    pub(crate) fn tokens(self) -> TokenStream {
        let name = Ident::new(self.into(), Span::call_site());
        quote!(::pyenum::__private::PyEnumBase::#name)
    }
}

/// Sole source of truth for base-name strings; every other conversion
/// (tokens, `Display`, `FromStr`) derives from this `From` impl.
impl From<BaseSelector> for &'static str {
    fn from(value: BaseSelector) -> Self {
        match value {
            BaseSelector::Enum => "Enum",
            BaseSelector::IntEnum => "IntEnum",
            BaseSelector::StrEnum => "StrEnum",
            BaseSelector::Flag => "Flag",
            BaseSelector::IntFlag => "IntFlag",
        }
    }
}

impl fmt::Display for BaseSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).into())
    }
}

impl FromStr for BaseSelector {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Enum" => Ok(BaseSelector::Enum),
            "IntEnum" => Ok(BaseSelector::IntEnum),
            "StrEnum" => Ok(BaseSelector::StrEnum),
            "Flag" => Ok(BaseSelector::Flag),
            "IntFlag" => Ok(BaseSelector::IntFlag),
            other => Err(format!(
                "unknown pyenum base `{other}`; expected one of `Enum`, `IntEnum`, `StrEnum`, `Flag`, `IntFlag`"
            )),
        }
    }
}

pub(crate) struct VariantSpec {
    pub rust_ident: Ident,
    pub value: VariantValue,
    #[allow(dead_code)]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub(crate) enum VariantValue {
    /// Explicit integer literal from a Rust discriminant.
    Int(i64),
    /// Explicit string literal from `#[pyenum(value = "...")]`.
    Str(String),
    /// No discriminant — defer to CPython's `enum.auto()`.
    Auto,
}

/// Parse a `TokenStream` from `#[proc_macro_derive]` into a [`DeriveSpec`].
pub(crate) fn parse_derive_input(input: TokenStream) -> Result<DeriveSpec> {
    let derive: DeriveInput = syn::parse2(input)?;
    parse(derive)
}

fn parse(input: DeriveInput) -> Result<DeriveSpec> {
    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            input.generics.span(),
            "#[derive(PyEnum)] cannot be applied to a generic or \
             lifetime-parameterised enum",
        ));
    }

    let data_enum = match input.data {
        Data::Enum(data) => data,
        Data::Struct(s) => {
            return Err(Error::new(
                s.struct_token.span,
                "#[derive(PyEnum)] can only be applied to enums, not structs",
            ));
        }
        Data::Union(u) => {
            return Err(Error::new(
                u.union_token.span,
                "#[derive(PyEnum)] can only be applied to enums, not unions",
            ));
        }
    };

    if data_enum.variants.is_empty() {
        return Err(Error::new(
            input.ident.span(),
            "#[derive(PyEnum)] requires at least one variant",
        ));
    }

    let EnumAttrs {
        python_name,
        base,
        python_module,
        python_qualname,
    } = parse_pyenum_attr(&input.attrs, &input.ident)?;

    let mut variants = Vec::with_capacity(data_enum.variants.len());
    for variant in data_enum.variants {
        variants.push(parse_variant(variant)?);
    }

    Ok(DeriveSpec {
        rust_ident: input.ident,
        python_name,
        base,
        variants,
        python_module,
        python_qualname,
    })
}

/// Enum-level `#[pyenum(...)]` attribute payload.
struct EnumAttrs {
    python_name: String,
    base: BaseSelector,
    python_module: Option<String>,
    python_qualname: Option<String>,
}

fn parse_variant(variant: Variant) -> Result<VariantSpec> {
    let span = variant.span();

    match variant.fields {
        Fields::Unit => {}
        Fields::Unnamed(_) | Fields::Named(_) => {
            return Err(Error::new(
                variant.ident.span(),
                format!(
                    "variant `{}` has fields; Python enum members must be \
                     unit variants",
                    variant.ident
                ),
            ));
        }
    }

    let explicit_str = parse_variant_attr(&variant.attrs, &variant.ident)?;

    let value = match (explicit_str, variant.discriminant) {
        (Some(_), Some((_, expr))) => {
            return Err(Error::new(
                expr.span(),
                format!(
                    "variant `{}` has both an `#[pyenum(value = ...)]` \
                     attribute and a Rust discriminant; specify only one",
                    variant.ident
                ),
            ));
        }
        (Some(s), None) => VariantValue::Str(s),
        (None, None) => VariantValue::Auto,
        (None, Some((_, expr))) => literal_from_expr(&expr, &variant.ident)?,
    };

    Ok(VariantSpec {
        rust_ident: variant.ident,
        value,
        span,
    })
}

fn parse_variant_attr(attrs: &[Attribute], variant_ident: &Ident) -> Result<Option<String>> {
    let mut value: Option<String> = None;

    for attr in attrs {
        if !attr.path().is_ident("pyenum") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("value") {
                if value.is_some() {
                    return Err(meta.error(format!(
                        "duplicate `value` in #[pyenum(...)] on variant `{variant_ident}`"
                    )));
                }
                let lit: LitStr = meta.value()?.parse()?;
                value = Some(lit.value());
                return Ok(());
            }
            let key = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "(unknown)".to_string());
            Err(meta.error(format!(
                "unknown key `{key}` in #[pyenum(...)] on variant \
                 `{variant_ident}`; expected: value"
            )))
        })?;
    }

    Ok(value)
}

fn literal_from_expr(expr: &Expr, variant_ident: &Ident) -> Result<VariantValue> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(int), ..
        }) => parse_int_literal(int),
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr: inner,
            ..
        }) => {
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(int), ..
            }) = inner.as_ref()
            {
                let positive = parse_int_literal(int)?;
                if let VariantValue::Int(v) = positive {
                    return Ok(VariantValue::Int(-v));
                }
            }
            Err(Error::new(
                expr.span(),
                format!(
                    "variant `{variant_ident}` has an unsupported \
                     discriminant expression; v1 only accepts integer \
                     literals"
                ),
            ))
        }
        _ => Err(Error::new(
            expr.span(),
            format!(
                "variant `{variant_ident}` has an unsupported discriminant \
                 expression; v1 only accepts integer literals"
            ),
        )),
    }
}

fn parse_int_literal(int: &LitInt) -> Result<VariantValue> {
    int.base10_parse::<i64>()
        .map(VariantValue::Int)
        .map_err(|e| Error::new(int.span(), format!("invalid integer literal: {e}")))
}

/// Walk `#[pyenum(...)]` attributes and resolve the enum-level keys. Unknown
/// keys and duplicates are rejected here.
fn parse_pyenum_attr(attrs: &[Attribute], enum_ident: &Ident) -> Result<EnumAttrs> {
    let mut base: Option<BaseSelector> = None;
    let mut python_name: Option<String> = None;
    let mut python_module: Option<String> = None;
    let mut python_qualname: Option<String> = None;

    for attr in attrs {
        if !attr.path().is_ident("pyenum") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("base") {
                if base.is_some() {
                    return Err(meta.error("duplicate `base` in #[pyenum(...)]"));
                }
                let value: LitStr = meta.value()?.parse()?;
                base = Some(
                    value
                        .value()
                        .parse::<BaseSelector>()
                        .map_err(|err| Error::new(value.span(), err))?,
                );
                return Ok(());
            }
            if meta.path.is_ident("name") {
                if python_name.is_some() {
                    return Err(meta.error("duplicate `name` in #[pyenum(...)]"));
                }
                let value: LitStr = meta.value()?.parse()?;
                python_name = Some(value.value());
                return Ok(());
            }
            if meta.path.is_ident("module") {
                if python_module.is_some() {
                    return Err(meta.error("duplicate `module` in #[pyenum(...)]"));
                }
                let value: LitStr = meta.value()?.parse()?;
                python_module = Some(value.value());
                return Ok(());
            }
            if meta.path.is_ident("qualname") {
                if python_qualname.is_some() {
                    return Err(meta.error("duplicate `qualname` in #[pyenum(...)]"));
                }
                let value: LitStr = meta.value()?.parse()?;
                python_qualname = Some(value.value());
                return Ok(());
            }
            let key = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "(unknown)".to_string());
            Err(meta.error(format!(
                "unknown key `{key}` in #[pyenum(...)]; expected one of: base, name, module, qualname"
            )))
        })?;
    }

    Ok(EnumAttrs {
        python_name: python_name.unwrap_or_else(|| enum_ident.to_string()),
        base: base.unwrap_or(BaseSelector::Enum),
        python_module,
        python_qualname,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn parse_err(input: TokenStream) -> String {
        match parse_derive_input(input) {
            Ok(_) => panic!("expected parse error"),
            Err(e) => e.to_string(),
        }
    }

    fn parse_ok(input: TokenStream) -> DeriveSpec {
        match parse_derive_input(input) {
            Ok(spec) => spec,
            Err(e) => panic!("expected successful parse: {e}"),
        }
    }

    #[test]
    fn rejects_struct() {
        let msg = parse_err(quote! {
            struct NotAnEnum { field: i32 }
        });
        assert!(msg.contains("can only be applied to enums, not structs"));
    }

    #[test]
    fn rejects_union() {
        let msg = parse_err(quote! {
            union NotAnEnum { a: u32, b: u32 }
        });
        assert!(msg.contains("can only be applied to enums, not unions"));
    }

    #[test]
    fn rejects_generic_enum() {
        let msg = parse_err(quote! {
            enum Color<T> { Red(T), Green, Blue }
        });
        assert!(msg.contains("generic or lifetime-parameterised enum"));
    }

    #[test]
    fn rejects_empty_enum() {
        let msg = parse_err(quote! {
            enum Nothing {}
        });
        assert!(msg.contains("requires at least one variant"));
    }

    #[test]
    fn rejects_tuple_variant() {
        let msg = parse_err(quote! {
            enum Color { Red, Rgb(u8, u8, u8) }
        });
        assert!(msg.contains("has fields"));
    }

    #[test]
    fn rejects_struct_variant() {
        let msg = parse_err(quote! {
            enum Color { Red, Rgb { r: u8, g: u8, b: u8 } }
        });
        assert!(msg.contains("has fields"));
    }

    #[test]
    fn rejects_unknown_top_level_key() {
        let msg = parse_err(quote! {
            #[pyenum(unknown = "x")]
            enum Color { Red, Green }
        });
        assert!(msg.contains("unknown key `unknown`"));
    }

    #[test]
    fn rejects_unknown_base() {
        let msg = parse_err(quote! {
            #[pyenum(base = "Bogus")]
            enum Color { Red, Green }
        });
        assert!(msg.contains("unknown pyenum base `Bogus`"));
    }

    #[test]
    fn rejects_duplicate_base() {
        let msg = parse_err(quote! {
            #[pyenum(base = "Enum", base = "IntEnum")]
            enum Color { Red, Green }
        });
        assert!(msg.contains("duplicate `base`"));
    }

    #[test]
    fn rejects_duplicate_name() {
        let msg = parse_err(quote! {
            #[pyenum(name = "A", name = "B")]
            enum Color { Red, Green }
        });
        assert!(msg.contains("duplicate `name`"));
    }

    #[test]
    fn accepts_name_override() {
        let spec = parse_ok(quote! {
            #[pyenum(name = "MyColor")]
            enum Color { Red, Green }
        });
        assert_eq!(spec.python_name, "MyColor");
        assert_eq!(spec.base, BaseSelector::Enum);
    }

    #[test]
    fn accepts_all_base_selectors() {
        for (literal, expected) in [
            ("Enum", BaseSelector::Enum),
            ("IntEnum", BaseSelector::IntEnum),
            ("StrEnum", BaseSelector::StrEnum),
            ("Flag", BaseSelector::Flag),
            ("IntFlag", BaseSelector::IntFlag),
        ] {
            let lit_ts: TokenStream = format!("#[pyenum(base = \"{literal}\")] enum E {{ A }}")
                .parse()
                .unwrap();
            let spec = parse_ok(lit_ts);
            assert_eq!(spec.base, expected, "for literal {literal}");
        }
    }

    #[test]
    fn accepts_negative_discriminant() {
        let spec = parse_ok(quote! {
            #[pyenum(base = "IntEnum")]
            enum Signed { Low = -5, Zero = 0, High = 5 }
        });
        let values: Vec<_> = spec
            .variants
            .iter()
            .map(|v| match &v.value {
                VariantValue::Int(i) => *i,
                _ => panic!("expected Int"),
            })
            .collect();
        assert_eq!(values, vec![-5, 0, 5]);
    }

    #[test]
    fn rejects_non_literal_discriminant() {
        let msg = parse_err(quote! {
            enum Math { Pi = 3 + 1 }
        });
        assert!(msg.contains("unsupported discriminant expression"));
    }

    #[test]
    fn rejects_negative_non_literal_discriminant() {
        let msg = parse_err(quote! {
            enum Math { X = -foo }
        });
        assert!(msg.contains("unsupported discriminant expression"));
    }

    #[test]
    fn rejects_oversized_integer_literal() {
        let msg = parse_err(quote! {
            enum Big { Huge = 99999999999999999999999 }
        });
        assert!(msg.contains("invalid integer literal"));
    }

    #[test]
    fn rejects_value_and_discriminant() {
        let msg = parse_err(quote! {
            enum Mixed {
                #[pyenum(value = "red")]
                Red = 1,
            }
        });
        assert!(msg.contains("both an `#[pyenum(value = ...)]` attribute and a Rust discriminant"));
    }

    #[test]
    fn rejects_duplicate_variant_value() {
        let msg = parse_err(quote! {
            enum Dup {
                #[pyenum(value = "a", value = "b")]
                X,
            }
        });
        assert!(msg.contains("duplicate `value`"));
    }

    #[test]
    fn rejects_unknown_variant_key() {
        let msg = parse_err(quote! {
            enum Bad {
                #[pyenum(bogus = "x")]
                X,
            }
        });
        assert!(msg.contains("unknown key `bogus`"));
    }

    #[test]
    fn skips_non_pyenum_attrs_on_enum_and_variant() {
        let spec = parse_ok(quote! {
            #[derive(Debug)]
            #[some_other_attr]
            enum Color {
                #[serde(rename = "red")]
                Red,
                Green,
            }
        });
        assert_eq!(spec.variants.len(), 2);
    }

    #[test]
    fn variant_value_auto_by_default() {
        let spec = parse_ok(quote! {
            enum Color { Red, Green }
        });
        for v in &spec.variants {
            assert!(matches!(v.value, VariantValue::Auto));
        }
    }

    #[test]
    fn variant_value_str_from_attr() {
        let spec = parse_ok(quote! {
            #[pyenum(base = "StrEnum")]
            enum Color {
                #[pyenum(value = "crimson")]
                Red,
                Green,
            }
        });
        match &spec.variants[0].value {
            VariantValue::Str(s) => assert_eq!(s, "crimson"),
            other => panic!("expected Str, got {other:?}"),
        }
        assert!(matches!(spec.variants[1].value, VariantValue::Auto));
    }

    #[test]
    fn base_selector_tokens_are_distinct() {
        let all = [
            BaseSelector::Enum,
            BaseSelector::IntEnum,
            BaseSelector::StrEnum,
            BaseSelector::Flag,
            BaseSelector::IntFlag,
        ];
        let rendered: Vec<String> = all.iter().map(|b| b.tokens().to_string()).collect();
        for (i, a) in rendered.iter().enumerate() {
            for b in rendered.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }
}
