# Contract: `pyenum::add_enum` (registration helper)

**Owner crate**: `pyenum`
**Surface**: public free function + extension trait on `pyo3::Bound<'_, PyModule>`
**Binding since**: v1.0.0

## Signatures

```rust
// Free function (canonical form, used by generated code and docs).
pub fn add_enum<T: PyEnum>(m: &Bound<'_, PyModule>) -> PyResult<()>;

// Extension trait (ergonomic form, imported via `use pyenum::prelude::*;`).
pub trait PyModuleExt {
    fn add_enum<T: PyEnum>(&self) -> PyResult<()>;
}

impl PyModuleExt for Bound<'_, PyModule> {
    fn add_enum<T: PyEnum>(&self) -> PyResult<()> {
        add_enum::<T>(self)
    }
}
```

## Behavioural contract

1. The call MUST invoke `T::py_enum_class(m.py())?` to resolve the (possibly cached) Python class.
2. The call MUST register the class on the module under `T::SPEC.name` via `m.add(T::SPEC.name, class)?`.
3. Subsequent calls to `add_enum::<T>` on the **same module handle** MUST overwrite the previous entry (PyO3's `add` does overwrite) — but in practice users call once per module per type; no idempotence beyond PyO3's own is promised.
4. No hidden global registration occurs. The call is pure with respect to the module handle; nothing happens if the user never calls it.
5. On the first call for a given `T` in a given interpreter, `py_enum_class` constructs the class; subsequent calls on any module handle reuse the cache.

## Usage contract

A typical PyO3 user writes:

```rust
use pyenum::prelude::*;   // brings PyModuleExt into scope
use pyo3::prelude::*;

#[pymodule]
fn my_ext<'py>(m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_enum::<crate::Permission>()?;
    m.add_enum::<crate::Color>()?;
    Ok(())
}
```

If the user prefers the free-function form (e.g. when the import of `prelude::*` is undesirable), both spellings are valid and semantically identical:

```rust
pyenum::add_enum::<crate::Permission>(&m)?;
```

## Error surface

- Any `PyErr` raised by `T::py_enum_class(py)` during first-call construction is propagated unchanged (typically an `ImportError` if the `enum` module cannot be imported — which would indicate a broken interpreter, not a library bug).
- Any `PyErr` from `m.add(...)` is propagated unchanged.

## Test obligations (this contract)

- `tests/python/conftest.py` + `tests/python/src/lib.rs`: the test extension registers one derived enum per supported `PyEnumBase` using both the free function and the extension method. Pytest fixtures import the module and assert each class is present and is a subclass of the expected base.
- `tests/python/test_cache.py`: import the module, perform 10,000 conversions, and assert — via an instrumented module-level Python counter — that the Python class object identity is stable across calls. (The counter is set by a helper exposed from the test extension, not by `pyenum` itself.)
- `tests/python/test_conversion.py`: exercises both directions of the conversion boundary from Python code calling `#[pyfunction]`s in the test extension.
