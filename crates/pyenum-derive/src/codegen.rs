//! Emit the `impl PyEnum` block for a parsed derive input.
//!
//! The generated code references items re-exported from
//! `pyenum::__private` — end-user crates see one stable surface regardless
//! of future internal refactors.

use crate::parse::{DeriveSpec, VariantValue};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Index, LitInt};

pub(crate) fn emit(spec: &DeriveSpec) -> TokenStream {
    let rust_ident = &spec.rust_ident;
    let python_name = &spec.python_name;
    let base_tokens = spec.base.tokens();

    let variant_count = spec.variants.len();
    let variant_count_lit = LitInt::new(&variant_count.to_string(), proc_macro2::Span::call_site());

    let variant_literals = spec.variants.iter().map(|v| {
        let name = v.rust_ident.to_string();
        let literal = match &v.value {
            VariantValue::Int(i) => quote!(::pyenum::__private::VariantLiteral::Int(#i)),
            VariantValue::Str(s) => {
                let s_lit = s.as_str();
                quote!(::pyenum::__private::VariantLiteral::Str(#s_lit))
            }
            VariantValue::Auto => quote!(::pyenum::__private::VariantLiteral::Auto),
        };
        quote! { (#name, #literal) }
    });

    // `[class.getattr("Red")?.unbind(), class.getattr("Green")?.unbind(), ...]`
    // — evaluated once per interpreter under `get_or_try_init`. Each entry is
    // an owned `Py<PyAny>` pinning the enum member so pointer comparison
    // against `obj.is(&members[i])` stays valid for the lifetime of the
    // cached class.
    let member_inits = spec.variants.iter().map(|v| {
        let member_name = v.rust_ident.to_string();
        quote! { __pyenum_class.getattr(#member_name)?.unbind() }
    });

    // `Self::Red => 0, Self::Green => 1, ...` — used by `to_py_member` to
    // index the cached member array without any Python-side lookup.
    let to_py_arms = spec.variants.iter().enumerate().map(|(i, v)| {
        let rust_variant = &v.rust_ident;
        let idx = Index::from(i);
        quote! { Self::#rust_variant => #idx, }
    });

    // One `if obj.is(&members[i]) { return Ok(Self::X); }` per variant.
    // Rust's compiler unrolls the linear scan; every comparison is a raw
    // pointer equality check against a cached Py<PyAny>, no allocations,
    // no attribute lookups.
    let from_py_checks = spec.variants.iter().enumerate().map(|(i, v)| {
        let rust_variant = &v.rust_ident;
        let idx = Index::from(i);
        quote! {
            if obj.is(&__pyenum_members[#idx]) {
                return ::core::result::Result::Ok(Self::#rust_variant);
            }
        }
    });

    let cache_ident = format_ident!("__PYENUM_CACHE_{}", rust_ident);
    let members_ident = format_ident!("__PYENUM_MEMBERS_{}", rust_ident);

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static #cache_ident: ::pyenum::__private::OnceLock<
            ::pyenum::__private::pyo3::Py<
                ::pyenum::__private::pyo3::types::PyType,
            >,
        > = ::pyenum::__private::OnceLock::new();

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static #members_ident: ::pyenum::__private::OnceLock<
            [
                ::pyenum::__private::pyo3::Py<
                    ::pyenum::__private::pyo3::PyAny,
                >;
                #variant_count_lit
            ],
        > = ::pyenum::__private::OnceLock::new();

        impl #rust_ident {
            /// Resolve the cached per-member `Py<PyAny>` array, constructing it
            /// on first call and reusing it on every subsequent conversion.
            #[doc(hidden)]
            fn __pyenum_cached_members<'py>(
                py: ::pyenum::__private::pyo3::Python<'py>,
            ) -> ::pyenum::__private::pyo3::PyResult<
                &'py [
                    ::pyenum::__private::pyo3::Py<
                        ::pyenum::__private::pyo3::PyAny,
                    >;
                    #variant_count_lit
                ],
            > {
                use ::pyenum::__private::pyo3::prelude::*;
                #members_ident.get_or_try_init(py, || {
                    let __pyenum_class = <Self as ::pyenum::__private::PyEnum>::py_enum_class(py)?;
                    ::core::result::Result::Ok::<_, ::pyenum::__private::pyo3::PyErr>([
                        #(#member_inits),*
                    ])
                })
            }
        }

        impl ::pyenum::__private::PyEnum for #rust_ident {
            const SPEC: ::pyenum::__private::PyEnumSpec = ::pyenum::__private::PyEnumSpec {
                name: #python_name,
                base: #base_tokens,
                variants: &[ #(#variant_literals),* ],
            };

            fn py_enum_class<'py>(
                py: ::pyenum::__private::pyo3::Python<'py>,
            ) -> ::pyenum::__private::pyo3::PyResult<
                ::pyenum::__private::pyo3::Bound<
                    'py,
                    ::pyenum::__private::pyo3::types::PyType,
                >,
            > {
                ::pyenum::__private::get_or_build(py, &#cache_ident, &Self::SPEC)
            }

            fn to_py_member<'py>(
                &self,
                py: ::pyenum::__private::pyo3::Python<'py>,
            ) -> ::pyenum::__private::pyo3::PyResult<
                ::pyenum::__private::pyo3::Bound<
                    'py,
                    ::pyenum::__private::pyo3::PyAny,
                >,
            > {
                use ::pyenum::__private::pyo3::prelude::*;
                let __pyenum_members = Self::__pyenum_cached_members(py)?;
                let __pyenum_idx: usize = match self {
                    #(#to_py_arms)*
                };
                ::core::result::Result::Ok(__pyenum_members[__pyenum_idx].bind(py).clone())
            }

            fn from_py_member<'py>(
                obj: &::pyenum::__private::pyo3::Bound<
                    'py,
                    ::pyenum::__private::pyo3::PyAny,
                >,
            ) -> ::pyenum::__private::pyo3::PyResult<Self> {
                use ::pyenum::__private::pyo3::prelude::*;
                let py = obj.py();
                let __pyenum_members = Self::__pyenum_cached_members(py)?;
                #(#from_py_checks)*
                let got = obj
                    .get_type()
                    .name()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|_| ::std::string::String::from("<unknown>"));
                ::core::result::Result::Err(
                    ::pyenum::__private::pyo3::exceptions::PyTypeError::new_err(
                        ::std::format!(
                            "expected `{}`, got `{}`",
                            <Self as ::pyenum::__private::PyEnum>::SPEC.name,
                            got,
                        ),
                    ),
                )
            }
        }

        impl<'py> ::pyenum::__private::pyo3::IntoPyObject<'py> for #rust_ident {
            type Target = ::pyenum::__private::pyo3::PyAny;
            type Output = ::pyenum::__private::pyo3::Bound<'py, Self::Target>;
            type Error = ::pyenum::__private::pyo3::PyErr;

            fn into_pyobject(
                self,
                py: ::pyenum::__private::pyo3::Python<'py>,
            ) -> ::core::result::Result<Self::Output, Self::Error> {
                <Self as ::pyenum::__private::PyEnum>::to_py_member(&self, py)
            }
        }

        impl<'py> ::pyenum::__private::pyo3::IntoPyObject<'py> for &#rust_ident {
            type Target = ::pyenum::__private::pyo3::PyAny;
            type Output = ::pyenum::__private::pyo3::Bound<'py, Self::Target>;
            type Error = ::pyenum::__private::pyo3::PyErr;

            fn into_pyobject(
                self,
                py: ::pyenum::__private::pyo3::Python<'py>,
            ) -> ::core::result::Result<Self::Output, Self::Error> {
                <#rust_ident as ::pyenum::__private::PyEnum>::to_py_member(self, py)
            }
        }

        impl<'a, 'py> ::pyenum::__private::pyo3::FromPyObject<'a, 'py> for #rust_ident {
            type Error = ::pyenum::__private::pyo3::PyErr;

            fn extract(
                obj: ::pyenum::__private::pyo3::Borrowed<'a, 'py, ::pyenum::__private::pyo3::PyAny>,
            ) -> ::core::result::Result<Self, Self::Error> {
                <Self as ::pyenum::__private::PyEnum>::from_py_member(&*obj)
            }
        }
    }
}
