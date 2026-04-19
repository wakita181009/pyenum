//! Parse a `#[derive(PyEnum)]` input into an internal IR.
//!
//! The IR (`DeriveSpec`, `VariantSpec`, `VariantValue`) is fed to
//! [`crate::codegen`]. Parsing performs shape-level validation only
//! (unit-variant enforcement, generics/lifetime rejection, attribute surface
//! shape). Identity-level rejection (reserved names, base/value mismatch)
//! happens in [`crate::validate`].

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, LitInt, LitStr, Result};

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
        use quote::quote;
        match self {
            BaseSelector::Enum => quote!(::pyenum::__private::PyEnumBase::Enum),
            BaseSelector::IntEnum => quote!(::pyenum::__private::PyEnumBase::IntEnum),
            BaseSelector::StrEnum => quote!(::pyenum::__private::PyEnumBase::StrEnum),
            BaseSelector::Flag => quote!(::pyenum::__private::PyEnumBase::Flag),
            BaseSelector::IntFlag => quote!(::pyenum::__private::PyEnumBase::IntFlag),
        }
    }

    fn from_str(value: &LitStr) -> Result<Self> {
        match value.value().as_str() {
            "Enum" => Ok(BaseSelector::Enum),
            "IntEnum" => Ok(BaseSelector::IntEnum),
            "StrEnum" => Ok(BaseSelector::StrEnum),
            "Flag" => Ok(BaseSelector::Flag),
            "IntFlag" => Ok(BaseSelector::IntFlag),
            other => Err(syn::Error::new(
                value.span(),
                format!(
                    "unknown pyenum base `{other}`; expected one of \
                     `Enum`, `IntEnum`, `StrEnum`, `Flag`, `IntFlag`"
                ),
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
    /// Explicit string literal (reserved; not emitted by v1 parser).
    #[allow(dead_code)]
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
        return Err(syn::Error::new(
            input.generics.span(),
            "#[derive(PyEnum)] cannot be applied to a generic or \
             lifetime-parameterised enum",
        ));
    }

    let data_enum = match input.data {
        Data::Enum(data) => data,
        Data::Struct(s) => {
            return Err(syn::Error::new(
                s.struct_token.span,
                "#[derive(PyEnum)] can only be applied to enums, not structs",
            ));
        }
        Data::Union(u) => {
            return Err(syn::Error::new(
                u.union_token.span,
                "#[derive(PyEnum)] can only be applied to enums, not unions",
            ));
        }
    };

    if data_enum.variants.is_empty() {
        return Err(syn::Error::new(
            input.ident.span(),
            "#[derive(PyEnum)] requires at least one variant",
        ));
    }

    let (python_name, base) = parse_pyenum_attr(&input.attrs, &input.ident)?;

    let mut variants = Vec::with_capacity(data_enum.variants.len());
    for variant in data_enum.variants {
        variants.push(parse_variant(variant)?);
    }

    Ok(DeriveSpec {
        rust_ident: input.ident,
        python_name,
        base,
        variants,
    })
}

fn parse_variant(variant: syn::Variant) -> Result<VariantSpec> {
    let span = variant.span();

    match variant.fields {
        Fields::Unit => {}
        Fields::Unnamed(_) | Fields::Named(_) => {
            return Err(syn::Error::new(
                variant.ident.span(),
                format!(
                    "variant `{}` has fields; Python enum members must be \
                     unit variants",
                    variant.ident
                ),
            ));
        }
    }

    let value = match variant.discriminant {
        None => VariantValue::Auto,
        Some((_, expr)) => literal_from_expr(&expr, &variant.ident)?,
    };

    Ok(VariantSpec {
        rust_ident: variant.ident,
        value,
        span,
    })
}

fn literal_from_expr(expr: &Expr, variant_ident: &Ident) -> Result<VariantValue> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(int), ..
        }) => parse_int_literal(int),
        Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
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
            Err(syn::Error::new(
                expr.span(),
                format!(
                    "variant `{variant_ident}` has an unsupported \
                     discriminant expression; v1 only accepts integer \
                     literals"
                ),
            ))
        }
        _ => Err(syn::Error::new(
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
        .map_err(|e| syn::Error::new(int.span(), format!("invalid integer literal: {e}")))
}

/// Walk `#[pyenum(...)]` attributes and extract the base selector + python
/// name override. Unknown keys and duplicate keys are rejected here.
fn parse_pyenum_attr(
    attrs: &[syn::Attribute],
    enum_ident: &Ident,
) -> Result<(String, BaseSelector)> {
    let mut base: Option<BaseSelector> = None;
    let mut python_name: Option<String> = None;

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
                base = Some(BaseSelector::from_str(&value)?);
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
            let key = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "(unknown)".to_string());
            Err(meta.error(format!(
                "unknown key `{key}` in #[pyenum(...)]; expected one of: base, name"
            )))
        })?;
    }

    Ok((
        python_name.unwrap_or_else(|| enum_ident.to_string()),
        base.unwrap_or(BaseSelector::Enum),
    ))
}
