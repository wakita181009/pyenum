//! Emit the `impl PyEnum` block for a parsed derive input.
//!
//! The generated code references items re-exported from
//! `pyenum::__private` — end-user crates see one stable surface regardless
//! of future internal refactors.

use crate::parse::{DeriveSpec, VariantValue};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn emit(spec: &DeriveSpec) -> TokenStream {
    let rust_ident = &spec.rust_ident;
    let python_name = &spec.python_name;
    let base_tokens = spec.base.tokens();

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

    let to_py_arms = spec.variants.iter().map(|v| {
        let rust_variant = &v.rust_ident;
        let member_name = v.rust_ident.to_string();
        quote! {
            Self::#rust_variant => {
                let class = <Self as ::pyenum::__private::PyEnum>::py_enum_class(py)?;
                class.getattr(#member_name).map(|b| b.into_any())
            }
        }
    });

    let from_py_arms = spec.variants.iter().map(|v| {
        let rust_variant = &v.rust_ident;
        let member_name = v.rust_ident.to_string();
        quote! {
            #member_name => ::core::result::Result::Ok(Self::#rust_variant),
        }
    });

    let cache_ident = format_ident!("__PYENUM_CACHE_{}", rust_ident);

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static #cache_ident: ::pyenum::__private::OnceLock<
            ::pyenum::__private::pyo3::Py<
                ::pyenum::__private::pyo3::types::PyType,
            >,
        > = ::pyenum::__private::OnceLock::new();

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
                match self {
                    #(#to_py_arms)*
                }
            }

            fn from_py_member<'py>(
                obj: &::pyenum::__private::pyo3::Bound<
                    'py,
                    ::pyenum::__private::pyo3::PyAny,
                >,
            ) -> ::pyenum::__private::pyo3::PyResult<Self> {
                use ::pyenum::__private::pyo3::prelude::*;
                let py = obj.py();
                let class = <Self as ::pyenum::__private::PyEnum>::py_enum_class(py)?;
                if !obj.is_instance(class.as_any())? {
                    let got = obj
                        .get_type()
                        .name()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|_| ::std::string::String::from("<unknown>"));
                    return ::core::result::Result::Err(
                        ::pyenum::__private::pyo3::exceptions::PyTypeError::new_err(
                            ::std::format!(
                                "expected `{}`, got `{}`",
                                <Self as ::pyenum::__private::PyEnum>::SPEC.name,
                                got,
                            ),
                        ),
                    );
                }
                let name_attr: ::std::string::String = obj.getattr("name")?.extract()?;
                match name_attr.as_str() {
                    #(#from_py_arms)*
                    other => ::core::result::Result::Err(
                        ::pyenum::__private::pyo3::exceptions::PyValueError::new_err(
                            ::std::format!(
                                "`{}` is not a recognised member of `{}`",
                                other,
                                <Self as ::pyenum::__private::PyEnum>::SPEC.name,
                            ),
                        ),
                    ),
                }
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
