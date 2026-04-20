//! Build a Python enum class via CPython's functional API.

use pyo3::IntoPyObject;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple, PyType};

use crate::trait_def::{PyEnumSpec, VariantLiteral};

/// Construct the Python enum class described by `spec`.
///
/// Imports the `enum` module, calls `enum.<Base>(name, members, …)`, and
/// hands back an owned `Bound<PyType>`. Variant values marked as
/// [`VariantLiteral::Auto`] are passed as `enum.auto()` so CPython applies
/// its per-base defaulting rules.
pub fn build_py_enum<'py>(py: Python<'py>, spec: &PyEnumSpec) -> PyResult<Bound<'py, PyType>> {
    let enum_mod = py.import("enum")?;
    let base_cls = enum_mod.getattr(spec.base.class_name())?;
    let auto_fn = enum_mod.getattr("auto")?;

    let members = PyList::empty(py);
    for (variant_name, literal) in spec.variants {
        let value: Bound<'py, PyAny> = match literal {
            VariantLiteral::Int(v) => v.into_pyobject(py)?.into_any(),
            VariantLiteral::Str(s) => (*s).into_pyobject(py)?.into_any(),
            VariantLiteral::Auto => auto_fn.call0()?,
        };
        let name_obj: Bound<'py, PyAny> = variant_name.into_pyobject(py)?.into_any();
        let entry = PyTuple::new(py, [name_obj, value])?;
        members.append(entry)?;
    }

    let name_arg: Bound<'py, PyAny> = spec.name.into_pyobject(py)?.into_any();
    let args = PyTuple::new(py, [name_arg, members.into_any()])?;
    let class = if spec.module.is_some() || spec.qualname.is_some() {
        let kwargs = PyDict::new(py);
        if let Some(m) = spec.module {
            kwargs.set_item("module", m)?;
        }
        if let Some(q) = spec.qualname {
            kwargs.set_item("qualname", q)?;
        }
        base_cls.call(args, Some(&kwargs))?
    } else {
        base_cls.call1(args)?
    };
    class.cast_into::<PyType>().map_err(Into::into)
}
