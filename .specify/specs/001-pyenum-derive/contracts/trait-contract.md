# Contract: `pyenum::PyEnum` trait + `PyEnumSpec`

**Owner crate**: `pyenum`
**Surface**: public trait implemented automatically by `#[derive(PyEnum)]`. Users SHOULD NOT implement it by hand; doing so is technically possible but unsupported.
**Binding since**: v1.0.0

## Trait definition

```rust
pub trait PyEnum: Sized + Copy + 'static {
    const SPEC: PyEnumSpec;

    /// Returns the cached Python class object for this enum, constructing it
    /// (exactly once per interpreter) on first call.
    fn py_enum_class<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyType>>;

    /// Returns the Python enum member corresponding to `self`. The returned
    /// `Bound<PyAny>` is `is`-equal to `py_enum_class(...).getattr(name)`.
    fn to_py_member<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;

    /// Extracts `Self` from a Python object. Raises `TypeError` if `obj` is not
    /// an instance of the cached class.
    fn from_py_member<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct PyEnumSpec {
    pub name: &'static str,
    pub base: PyEnumBase,
    pub variants: &'static [(&'static str, VariantLiteral)],
}

#[derive(Debug, Clone, Copy)]
pub enum PyEnumBase { Enum, IntEnum, StrEnum, Flag, IntFlag }

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum VariantLiteral {
    Int(i64),
    Str(&'static str),
    Auto,
}
```

`VariantLiteral` is `#[doc(hidden)]` — not part of the public contract beyond "whatever the derive emits". Its enum layout MAY be extended in minor versions.

## Behavioural contract

### `py_enum_class`

1. Called with the GIL held (enforced by the `Python<'py>` parameter).
2. MUST return the same `Bound<'py, PyType>` object — identity-equal — for every call within the same interpreter.
3. First call constructs the class via Python's functional API: `enum.<base>(name, members)`, where `members` is `[(name, value), …]` in declaration order. `Auto` values are emitted as calls to `enum.auto()`.
4. Concurrent first-call attempts from two threads (both under the GIL — so effectively sequenced) MUST result in exactly one construction.
5. MUST propagate any `PyErr` from the functional API unchanged.

### `to_py_member`

1. Called with the GIL held.
2. MUST return the exact Python member object — `x.to_py_member(py)? is Py_enum_class(py)?.getattr(name)?` is `True`.
3. MUST NOT allocate a new Python object per call; the returned `Bound` references a cached attribute.

### `from_py_member`

1. Called with the GIL held (implied by `Bound`).
2. If `obj` is an instance of `py_enum_class(py)?`, MUST return the matching `Self` variant.
3. If `obj` is NOT an instance, MUST raise `PyTypeError` with a message containing `T::SPEC.name`.
4. MUST NOT attempt coercion (e.g., `IntEnum` from raw `int`). Python-side coercion is handled by Python's own enum `__call__` machinery when constructed via `MyEnum(value)`; Rust-side extraction is strict.

### `SPEC`

1. `variants` MUST be non-empty.
2. Order of `variants` MUST match declaration order in the Rust source.
3. `variants` MAY contain duplicate values; those become Python aliases of the first occurrence.

## Stability

- Adding a new `PyEnumBase` variant in a future minor version (e.g., for `ReprEnum`) is non-breaking *only* for code that matches on the enum exhaustively via the public API. Internal exhaustive matches in the derive are fine because we own both sides.
- Adding a new `VariantLiteral` case is non-breaking because the type is `#[doc(hidden)]`.

## Test obligations (this contract)

- `crates/pyenum/tests/cache.rs`: instrument a counter around the first-call construction path; hammer concurrently via spawned threads holding the GIL; assert the counter is 1 at the end.
- `crates/pyenum/tests/convert.rs`: assert that `x.to_py_member(py)?.is(py_enum_class(py)?.getattr(name)?)` is `True` for every variant of a test enum (each supported base).
- `crates/pyenum/tests/from_py.rs`: assert `from_py_member` raises `TypeError` on foreign objects and returns the right variant on class members.
- `#[cfg(test)]` blocks inside `crates/pyenum/src/trait_def.rs`: static checks that `SPEC.variants` is non-empty for every fixture enum used elsewhere in the suite (the empty-enum case is itself covered at compile time by `crates/pyenum-derive/tests/ui/fail/`).
- Python-side equivalents (end-to-end against the `pyenum-test` cdylib) are covered by `tests/test_cache.py` and `tests/test_conversion.py`.
