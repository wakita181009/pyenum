# Phase 1 Data Model: pyenum (001)

**Feature**: [spec.md](./spec.md) · **Plan**: [plan.md](./plan.md) · **Research**: [research.md](./research.md)

This document names the internal entities the library manipulates and their relationships. The library has no persistent storage; every entity here lives in the Rust process address space or is a reference into the Python interpreter heap.

---

## Entities

### `PyEnumBase` (enum)

Selector for the Python base class a derived type will extend.

| Field | Type | Notes |
|-------|------|-------|
| variant | one of `Enum`, `IntEnum`, `StrEnum`, `Flag`, `IntFlag` | Case-sensitive, matches the Python class name exactly (R6). |

Derived from the `base = "…"` string in `#[pyenum(base = "…")]`, defaulting to `Enum`.

Invariants:
- Exactly one `PyEnumBase` per derived type.
- `Enum`/`IntEnum` → member values must be integer (or `auto()`); `StrEnum` → string (or `auto()`); `Flag`/`IntFlag` → integer with recommended power-of-two (or `auto()`).

### `VariantSpec` (record)

Per-variant description collected by the proc-macro and baked into generated code.

| Field | Type | Notes |
|-------|------|-------|
| `rust_name` | `&'static str` | The Rust identifier, used as-is for the Python member name (Q2 / FR-014). |
| `value` | `VariantValue` (see below) | Determines how codegen emits the `(name, value)` tuple passed to the functional API. |
| `span` | `Span` | Used at compile time only — retained for the purpose of routing `compile_error!` diagnostics back to the source variant. Not present in generated runtime data. |

Invariants:
- `rust_name` is a valid Rust identifier (enforced by `syn`).
- `rust_name` is NOT in the reserved-name set (R8) — otherwise the proc-macro aborts with `compile_error!`.
- If `value.kind == Explicit`, the literal kind matches the declared `PyEnumBase`.

### `VariantValue` (enum)

How the generated code materialises each member's value.

| Variant | Payload | Emitted expression |
|---------|---------|--------------------|
| `ExplicitInt(i64)` | 64-bit integer literal from the Rust discriminant | `<int-literal>` |
| `ExplicitStr(String)` | string literal from `#[pyenum(value = "…")]` (reserved for future v1+ use; not parsed in v1) | `<string-literal>` |
| `Auto` | none | `pyenum::__private::auto()` — at runtime resolves to `enum.auto()` imported once per module |

Invariants (per `PyEnumBase`):
- `Enum` / `IntEnum` / `Flag` / `IntFlag`: `ExplicitInt` or `Auto` only.
- `StrEnum`: `Auto` only in v1 (explicit string values are deferred to v1.1 to avoid expanding attribute surface now); we revisit when we add `#[pyenum(value = "…")]`.

### `PyEnumSpec` (struct, generated per derived type)

The *metadata* the derive emits for each enum. Consumed at runtime by the cache/build path.

| Field | Type | Notes |
|-------|------|-------|
| `name` | `&'static str` | Python class name. Defaults to the Rust enum's identifier; overridable via `#[pyenum(name = "…")]` (v1 will accept the attribute but the proc-macro will treat omission as the common case). |
| `base` | `PyEnumBase` | From the derive attribute. |
| `variants` | `&'static [(&'static str, VariantLiteral)]` | Ordered per declaration; duplicates preserved (Python treats them as aliases of the first occurrence). |

Invariants:
- `variants` is non-empty (empty enums are rejected at compile time).
- Variant ordering matches declaration order (FR-003's iteration-order guarantee).
- `VariantLiteral` is a `#[doc(hidden)]` enum that mirrors `VariantValue` at runtime — see R7.

### `PyEnum` (trait, public)

The bridge between user types and the Python class/cache.

```text
pub trait PyEnum: Sized + Copy + 'static {
    const SPEC: PyEnumSpec;
    fn py_enum_class<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyType>>;
    fn to_py_member<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;
    fn from_py_member<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Self>;
}
```

All three methods are emitted by the derive. `py_enum_class` is the cache accessor; `to_py_member` / `from_py_member` are the workhorses behind the blanket `IntoPyObject` / `FromPyObject` impls. User code never implements this trait by hand.

### `EnumClassCache` (implicit entity)

Logically a `HashMap<TypeId, Py<PyType>>` scoped per interpreter, but physically realised as a per-type `GILOnceCell<Py<PyType>>` living inside each `impl PyEnum`. No single shared cache exists — distribution makes lock contention impossible.

| Field | Type | Notes |
|-------|------|-------|
| `cell` | `GILOnceCell<Py<PyType>>` | Initialised on first `py_enum_class` call on the current interpreter. |

Invariants:
- At most one successful initialisation per (`TypeId`, interpreter-id) pair.
- Concurrent callers under the GIL see the same `Py<PyType>` instance.
- Cache contents are never invalidated during the lifetime of the interpreter.

### `Conversion Boundary` (conceptual)

The compiled site where a Rust enum value becomes (or is recovered from) a Python member. Implemented by blanket impls of `IntoPyObject` and `FromPyObject` over `T: PyEnum`. No runtime representation.

---

## Relationships

```
User's enum  ──[#[derive(PyEnum)]]──▶  Derived impl PyEnum
        │                                      │
        │                                      ├─ SPEC: PyEnumSpec  (static metadata)
        │                                      ├─ py_enum_class()   ──▶ GILOnceCell ──▶ Py<PyType>
        │                                      ├─ to_py_member()    ──▶ Bound<PyAny>
        │                                      └─ from_py_member()  ──▶ Self
        │
        └─ PyO3 call sites (via IntoPyObject / FromPyObject blanket impls)

add_enum::<T>(m)  ──▶  T::py_enum_class(py)  ──▶  m.add(T::SPEC.name, class)
```

- One `PyEnum` impl per user-defined Rust enum.
- One `PyEnumSpec` per impl (static).
- One `GILOnceCell<Py<PyType>>` per impl per interpreter.
- Many call sites per impl (conversion boundary).

## State Transitions

The only stateful entity is `EnumClassCache`. Two states:

```
    Uninitialised ──(first py_enum_class() under GIL)──▶ Initialised
        │                                                   │
        └── concurrent caller blocks until init done ───────┘
```

No reverse transition (no invalidation) during interpreter lifetime. Sub-interpreter finalisation is out of scope for v1 (documented assumption).

## Validation Rules (compile-time, proc-macro enforced)

| Rule | Source | Diagnostic |
|------|--------|-----------|
| Only unit variants | FR-004 | `variant `Foo` has fields; Python enum members must be unit variants` |
| No generics / lifetimes on the enum | FR-004 | `#[derive(PyEnum)] cannot be applied to a generic or lifetime-parameterised enum` |
| Non-empty | Edge Cases | `#[derive(PyEnum)] requires at least one variant` |
| Variant name not reserved | FR-004a / R8 | `variant `class` collides with a Python keyword` (category-specific wording) |
| Explicit discriminant kind matches base | FR-005 | `variant `Foo` declares a string discriminant but the chosen base `IntEnum` requires integer values` |
| At most one `#[pyenum(base = …)]` on the type | FR-002 | `duplicate #[pyenum(base = …)] attribute` |

## Validation Rules (runtime, raised at the conversion boundary)

| Rule | Source | Python exception raised |
|------|--------|-------------------------|
| Argument is an instance of the exposed class | FR-011 | `TypeError: expected <ClassName>, got <type>` |
| Integer coercion (`IntEnum`) respects enum domain | FR-003 | `ValueError: <value> is not a valid <ClassName>` (raised by Python's own enum machinery; we propagate) |
