# Contract: `#[derive(PyEnum)]`

**Owner crate**: `pyenum-derive`
**Surface**: public proc-macro derive
**Binding since**: v1.0.0

## Attribute surface

```rust
#[derive(PyEnum)]
#[pyenum(base = "IntFlag")]  // optional; default "Enum"; one of Enum | IntEnum | StrEnum | Flag | IntFlag
#[pyenum(name = "Permissions")] // optional; default = Rust enum identifier
enum Permission {
    Read = 1,
    Write = 2,
    Admin = 4,
}
```

Only the documented keys are recognised. Unknown keys inside `#[pyenum(...)]` are a compile error.

## Accepts (compile-success contract)

The macro accepts, and successfully emits code for, any Rust enum satisfying ALL of:

1. At least one variant.
2. Every variant is `Fields::Unit`.
3. No generics and no lifetime parameters on the enum header.
4. Every variant identifier is outside the reserved-name set (Python keywords ∪ enum-reserved member names ∪ enum-special dunders — full list in `pyenum-derive/src/reserved.rs`).
5. If a variant carries a discriminant literal:
   - For `Enum`/`IntEnum`/`Flag`/`IntFlag`: the literal is an integer literal (`i64` range).
   - For `StrEnum`: no explicit discriminant in v1 (value defaults to variant name via `enum.auto()`).
6. Variants without a discriminant take `auto()` — resolved by CPython's own `enum.auto()` per base.
7. Duplicate discriminant values are permitted and become Python aliases (FR-003, spec Assumptions).

## Rejects (compile-error contract)

The macro MUST emit `compile_error!` with the span on the offending item and a human-readable message when ANY of the following is true. Every case listed here has a corresponding `pyenum-derive/tests/ui/fail/*.rs` + `*.stderr` snapshot:

| Case | Example | Expected message fragment |
|------|---------|---------------------------|
| Variant has fields (tuple) | `Foo(u32)` | `variant \`Foo\` has fields; Python enum members must be unit variants` |
| Variant has fields (struct) | `Foo { x: u32 }` | `variant \`Foo\` has fields; Python enum members must be unit variants` |
| Enum is generic | `enum E<T> { … }` | `#[derive(PyEnum)] cannot be applied to a generic or lifetime-parameterised enum` |
| Enum has lifetime | `enum E<'a> { … }` | same as above |
| Enum is empty | `enum E {}` | `#[derive(PyEnum)] requires at least one variant` |
| Variant collides with Python keyword | `Class` with `base = "Enum"`: OK; but `Pass`, `None`, `True`, etc. collide | `variant \`Pass\` collides with a Python keyword` |
| Variant collides with enum-reserved member name | `_value_`, `_missing_`, `_name_`, … | `variant \`_value_\` collides with an enum-reserved member name` |
| Variant collides with a special dunder | `__init__`, `__new__`, … | `variant \`__init__\` collides with an enum special method name` |
| Base/value mismatch | `IntEnum` base with `Foo = "bar"` | `variant \`Foo\` declares a string discriminant but the chosen base \`IntEnum\` requires integer values` |
| Duplicate `#[pyenum(base)]` | two `base =` attrs | `duplicate \`base\` in #[pyenum(...)]` |
| Unknown attribute key | `#[pyenum(bogus = 1)]` | `unknown key \`bogus\` in #[pyenum(...)]\n  expected one of: base, name` |

## Emitted items

Given `#[derive(PyEnum)]` on `enum MyEnum { … }`, the macro emits, **into the same module as the enum**, these items (exact names are implementation-private but stable in behaviour):

1. `impl pyenum::PyEnum for MyEnum` with:
   - `const SPEC: pyenum::PyEnumSpec = …;`
   - `fn py_enum_class<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyType>>` (uses `GILOnceCell`).
   - `fn to_py_member<'py>(&self, py) -> PyResult<Bound<'py, PyAny>>`.
   - `fn from_py_member<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Self>`.
2. `impl<'py> pyo3::IntoPyObject<'py> for MyEnum` delegating to `PyEnum::to_py_member`.
3. `impl<'py> pyo3::IntoPyObject<'py> for &MyEnum` delegating to `PyEnum::to_py_member`.
4. `impl<'py> pyo3::FromPyObject<'py> for MyEnum` delegating to `PyEnum::from_py_member`.

No other items are emitted. No public items are emitted outside `impl` blocks.

## Idempotence

Applying the derive twice on the same type is a compile error (Rust's orphan/overlap rules enforce this — no special handling needed).

## Test obligations (this contract)

A compile-fail fixture + `.stderr` snapshot MUST exist for every row of the Rejects table, plus a compile-success fixture for every row-class of the Accepts contract. Added via `crates/pyenum-derive/tests/ui/{accept,fail}/*.rs` and gated by `cargo test --test ui`. The Python-side suite in `tests/` does not re-test these compile-time rejections (there is no pure-Python reference); the trybuild fixtures are the sole source of truth for rejection behavior.
