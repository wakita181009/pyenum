# Feature Specification: pyenum — Rust-Defined Python Enums for PyO3

**Feature Branch**: `001-pyenum-derive`
**Created**: 2026-04-20
**Status**: Draft
**Input**: User description: "Rust library enabling PyO3 users to expose Rust enums as true `enum.Enum` subclasses in Python, supporting all Python enum variants (Enum, IntEnum, StrEnum, Flag, IntFlag) with caching and bidirectional conversion, targeting PyO3 0.28."

## Clarifications

### Session 2026-04-20

- Q: For variants declared without an explicit discriminant, which default value strategy does the derive apply? → A: `auto()`-equivalent semantics — `Enum`/`IntEnum` use 1-based incrementing integers, `Flag`/`IntFlag` use successive powers of two, `StrEnum` uses the variant's declared name as its string value. Behavior matches what Python's own `enum.auto()` would produce for the chosen base.
- Q: How are Rust variant identifiers mapped to Python enum member names? → A: Pass-through. The Python member name is exactly the Rust variant identifier (e.g., `HttpOk` in Rust → `MyEnum.HttpOk` and `MyEnum["HttpOk"]` in Python). No automatic casing transformation. A future `#[pyenum(rename = "...")]`-style per-variant override MAY be added, but is out of scope for v1.
- Q: How does the derive handle the zero-valued member for `Flag` / `IntFlag`? → A: Explicit declaration only. The derive never auto-synthesizes a zero-valued member; if the user wants one, they declare a variant with value `0` themselves. The `auto()` default sequence (Q1) starts at 1 (2^0) regardless of whether a zero member is declared, matching CPython's own `enum.Flag` behavior.
- Q: How does a user register a `#[derive(PyEnum)]` type into a `#[pymodule]` block? → A: Via an explicit generic helper exposed by the library, e.g. `pyenum::add_enum::<MyEnum>(&m)?` (also surfaceable as an extension method, `m.add_enum::<MyEnum>()?`). The derive emits whatever trait impl the helper needs. No auto-registration, no replacement module macro, no overload of PyO3's `add_class`.
- Q: What happens when a Rust variant name collides with a Python keyword or an enum-reserved dunder (e.g. `class`, `_value_`, `_missing_`, `__init__`)? → A: Compile-time rejection. The derive carries a known-forbidden set — Python keywords plus `enum`-module reserved names (`_name_`, `_value_`, `_missing_`, `_generate_next_value_`, `_ignore_`, `_order_`, `name`, `value`) plus dunders that `enum.EnumType` interprets specially (`__init__`, `__new__`, `__class__`, `__members__`, `__init_subclass__`, `__set_name__`, `__class_getitem__`, `__repr__`, `__str__`, `__hash__`, `__eq__`, `__format__`, `__dir__`, `__bool__`, `__reduce_ex__`) — and emits a compile error naming the offending variant. No auto-rename, no runtime deferral, no opt-out in v1.
- Q: Which PyO3 version does v1 target? → A: PyO3 **0.28 exclusively**. An earlier draft proposed a 0.25–0.28 feature matrix; this was withdrawn after discovering that `pyo3-ffi`'s `links = "python"` native-library-uniqueness rule prevents multiple `pyo3` versions from coexisting as optional dependencies in the same cargo graph, even when the features are mutually exclusive. Supporting additional versions, if ever needed, would require a separate-crate strategy (e.g. `pyenum-0_28`, `pyenum-0_29`) or a `build.rs`-based detection scheme — both out of scope for v1.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Expose a Rust enum as a Python `Enum` subclass (Priority: P1)

A Rust developer building a Python extension with PyO3 defines a Rust enum (e.g., `Color { Red, Green, Blue }`) and annotates it with a derive attribute. When the Python side imports the extension module, `Color` appears as a genuine subclass of `enum.Enum`, passes `isinstance(Color.RED, enum.Enum)`, and integrates with any downstream library that inspects enum membership (pydantic, FastAPI, SQLAlchemy, dataclasses).

**Why this priority**: This is the core value proposition. Without true `enum.Enum` subclass behavior, users continue hitting the exact interoperability problem that motivates the library. The other user stories extend this capability but none of them are useful without it.

**Independent Test**: Define a unit-variant Rust enum with the derive attribute, build the extension, import it in Python, and assert that the exposed symbol is a subclass of `enum.Enum`, that each variant is a member, and that standard enum protocol operations (`list(MyEnum)`, `MyEnum["RED"]`, `MyEnum.RED.name`, `MyEnum.RED.value`) all behave per the Python enum specification.

**Acceptance Scenarios**:

1. **Given** a Rust enum with unit variants only and the derive attribute applied, **When** the extension module is imported in Python, **Then** the enum is exposed as a class where `issubclass(MyEnum, enum.Enum)` is `True` and every Rust variant is accessible as a Python enum member.
2. **Given** the exposed Python enum, **When** iterated (`list(MyEnum)`), **Then** the iteration order matches the declaration order of the Rust variants and yields only the defined members.
3. **Given** the exposed Python enum, **When** accessed by name (`MyEnum["RED"]`) or by value (`MyEnum(1)`), **Then** the lookup returns the expected member or raises `KeyError`/`ValueError` consistently with Python's own `Enum`.
4. **Given** a Rust enum declaration that violates Python enum rules (e.g., variant carrying fields, duplicate explicit values in a non-alias context), **When** the crate is compiled, **Then** compilation fails with a clear, actionable error identifying the offending variant.

---

### User Story 2 - Support all five standard Python enum base types (Priority: P1)

The same derive mechanism supports selecting which Python enum base class backs the exposed type: `Enum` (default), `IntEnum`, `StrEnum`, `Flag`, and `IntFlag`. The developer chooses the base type via a derive attribute option. The resulting Python class satisfies the full semantic contract of the chosen base (e.g., `IntFlag` members support bitwise operators; `StrEnum` members are `str` subclasses).

**Why this priority**: Modern Python codebases rely on these specialized bases heavily (status codes as `IntEnum`, permission bits as `IntFlag`, textual keys as `StrEnum`). Shipping only `Enum` would force users to drop back to `#[pyclass]` workarounds for the most common real-world cases and defeat the library's purpose.

**Independent Test**: For each of the five base types, define a Rust enum with the appropriate value type, apply the derive with the corresponding base option, and verify in Python that (a) the class is a subclass of the requested base, (b) value types match the base's contract, and (c) base-specific operations succeed (bitwise OR/AND for flag types, integer arithmetic for `IntEnum`, string operations for `StrEnum`).

**Acceptance Scenarios**:

1. **Given** a Rust enum targeting `IntEnum` with explicit integer variant values, **When** exposed to Python, **Then** the class is a subclass of `enum.IntEnum`, members compare equal to their integer values, and standard `IntEnum` arithmetic works.
2. **Given** a Rust enum targeting `StrEnum` with explicit string values, **When** exposed to Python, **Then** the class is a subclass of `enum.StrEnum`, members are `str` instances, and string operations (concatenation, formatting) behave as expected.
3. **Given** a Rust enum targeting `IntFlag` with power-of-two values, **When** exposed to Python, **Then** bitwise composition (`A | B`) produces a valid combined flag member and membership tests (`A in combined`) return `True`.
4. **Given** a Rust enum whose declared values are incompatible with the requested base type (e.g., non-integer values for `IntEnum`), **When** compiled, **Then** compilation fails with a clear error explaining the mismatch.

---

### User Story 3 - Automatic bidirectional Rust ↔ Python conversion (Priority: P1)

A PyO3 function signature using the Rust enum type accepts and returns values transparently. Callers pass the Python enum member and Rust code receives the Rust variant; Rust returns a variant and Python callers receive the matching Python enum member. Developers do not write manual conversion shims.

**Why this priority**: Without this, every PyO3 function boundary involving the enum requires hand-rolled conversion code, which is exactly the boilerplate the library is meant to eliminate. This is inseparable from the core value of the derive.

**Independent Test**: Define a PyO3 function `fn invert(c: Color) -> Color` annotated with `#[pyfunction]`, expose it to Python, call it with `Color.RED`, and assert the returned object is `Color.GREEN` (or whatever the inversion rule dictates) — no manual conversion is written by the user anywhere in the call chain.

**Acceptance Scenarios**:

1. **Given** a `#[pyfunction]` accepting the Rust enum type as a parameter, **When** Python calls it passing a Python enum member, **Then** the Rust function receives the correct Rust variant without explicit user conversion code.
2. **Given** a `#[pyfunction]` returning the Rust enum type, **When** Rust returns a variant, **Then** the Python caller receives the corresponding Python enum member (same identity as `MyEnum.VARIANT_NAME`).
3. **Given** Python code passing an object that is not a member of the expected enum, **When** the function is called, **Then** PyO3 raises a `TypeError` with a clear message identifying the expected type.

---

### User Story 4 - One-time construction via cached singleton (Priority: P2)

The Python enum class is constructed exactly once per Rust enum type per Python interpreter, the first time it is needed. Subsequent accesses — from conversion, from user code reading the class, or from re-imports within the same interpreter — reuse the cached class. The cache is safe under the GIL and its behavior across sub-interpreters is consistent with PyO3 0.28's `PyOnceLock` — the library does not attempt guarantees beyond what that primitive provides.

**Why this priority**: Functional construction of Python enums via `Enum("Name", [...])` is not free. Reconstructing on every boundary crossing would make the library unusable at scale. Users will not explicitly observe the cache, but they will observe its absence through latency and identity bugs (two "same" enum classes being `!=`).

**Independent Test**: Instrument a counter or log around the construction call, exercise the conversion boundary 10,000 times, and verify the counter equals 1. Additionally verify that the class object retrieved through two independent code paths is the same Python object (identity, not just equality).

**Acceptance Scenarios**:

1. **Given** the extension module has been imported, **When** the Rust enum is converted to and from Python repeatedly, **Then** the underlying Python enum class is constructed once and reused on every subsequent access.
2. **Given** two separate Rust code paths that both trigger Python exposure of the same Rust enum type, **When** both resolve the Python class, **Then** they receive the same Python class object (`is` identity holds).
3. **Given** concurrent access to the enum from multiple Python threads (under the GIL), **When** the first conversion races with a second, **Then** construction happens exactly once and neither caller observes a partially-initialized class.

---

### User Story 5 - Rejecting non-conforming Rust enums at compile time (Priority: P2)

When a developer applies the derive to a Rust enum that cannot be faithfully represented as a Python enum, the crate refuses to compile and emits a diagnostic that names the exact violation and the variant at fault. Examples: variants with fields, generics, lifetimes, or value types incompatible with the chosen base.

**Why this priority**: Silent acceptance of ill-formed input would produce runtime errors in downstream Python code that are very hard to trace back to the Rust declaration. Failing loudly at compile time is a correctness property — it prevents a class of bugs entirely — but it ships alongside US2, not before it.

**Independent Test**: Maintain a collection of intentionally invalid Rust enum declarations (one per violation class), attempt to build each, and assert that (a) compilation fails, (b) the error message references the offending variant by name, and (c) the message explains which Python enum rule was violated.

**Acceptance Scenarios**:

1. **Given** a Rust enum with a tuple-struct or struct variant, **When** the derive is applied and the crate is compiled, **Then** compilation fails with a message identifying the variant and explaining that Python enum members must be unit variants.
2. **Given** a Rust enum declared generic or with lifetime parameters, **When** the derive is applied, **Then** compilation fails with a message explaining that Python enums cannot be parameterized.
3. **Given** explicit variant values incompatible with the requested Python base type, **When** compiled, **Then** compilation fails with a message naming the variant, the declared value, and the required value kind for the base.

---

### Edge Cases

- **Variant aliases**: If two variants declare the same explicit value, Python enum semantics treat the second as an alias of the first. The library must either faithfully preserve this aliasing behavior or reject it with a clear diagnostic; silent reordering or deduplication is not acceptable.
- **Empty enum**: A Rust enum with zero variants cannot form a valid Python enum. The derive must reject it at compile time.
- **Variant name collisions with Python reserved words or dunder names**: The derive rejects these at compile time. The forbidden set includes Python keywords (`class`, `def`, `None`, `True`, `False`, `return`, `import`, etc.), `enum`-module reserved member names (`_name_`, `_value_`, `_missing_`, `_generate_next_value_`, `_ignore_`, `_order_`), and dunders the `enum.EnumType` metaclass interprets specially (`__init__`, `__new__`, `__class__`, `__members__`, `__init_subclass__`, `__set_name__`). No auto-escape, no opt-out attribute in v1.
- **Sub-interpreter lifecycle**: When a sub-interpreter is finalized, the cached Python class reference for that interpreter must not dangle. Cache keying must account for per-interpreter state.
- **Module reloading**: If the Python module is reloaded, consumers that held a reference to the previous enum class must not silently receive a stale class. Behavior on reload must be documented, even if the chosen behavior is "reload is not supported."
- **Flag-type zero value**: `Flag`/`IntFlag` semantics permit a zero-valued "no flags" member. The derive supports this only when the user explicitly declares a variant with value `0`; it never auto-synthesizes one. The `auto()` default sequence begins at `1` (2^0) and is unaffected by the presence or absence of a declared zero member.
- **Very large enums**: An enum with thousands of variants should still construct and cache within a reasonable startup budget (see SC-004); performance degradation must be linear in variant count, not worse.
- **Derive combined with other PyO3 attributes on the same type**: If the user also applies `#[pyclass]` or unrelated PyO3 attributes to the type, behavior must be deterministic — either cleanly integrated or rejected with a diagnostic, never silently broken.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The library MUST provide a derive macro that, when applied to a Rust enum, causes that enum to be exposed to Python as a true subclass of the selected Python enum base class.
- **FR-002**: The derive MUST support targeting any of the five standard Python enum bases — `enum.Enum`, `enum.IntEnum`, `enum.StrEnum`, `enum.Flag`, `enum.IntFlag` — selectable via an attribute option, with `enum.Enum` as the default.
- **FR-003**: The exposed Python class MUST satisfy the full semantic contract of the chosen base class per the Python enum specification, including `isinstance`/`issubclass` relationships, member iteration order, name/value lookup, aliasing rules, and base-specific operations (bitwise composition for flag types, value-type behavior for `IntEnum`/`StrEnum`).
- **FR-004**: The derive MUST accept only Rust enums consisting exclusively of unit variants, and MUST reject at compile time any enum containing tuple-struct variants, struct variants, generics, or lifetime parameters, with diagnostics naming the offending variant.
- **FR-004a**: The derive MUST reject at compile time any variant whose Rust identifier collides with a Python keyword, with a reserved `enum`-module member name (`_name_`, `_value_`, `_missing_`, `_generate_next_value_`, `_ignore_`, `_order_`), or with a dunder that `enum.EnumType` interprets specially (`__init__`, `__new__`, `__class__`, `__members__`, `__init_subclass__`, `__set_name__`). The diagnostic MUST name the offending variant and the specific collision category. No auto-rename and no opt-out attribute are offered in v1.
- **FR-005**: The derive MUST validate that declared variant values are compatible with the chosen Python base type at compile time (or, where value determination is deferred to runtime construction, at first use with a clear error identifying the declaration site).
- **FR-006**: The library MUST generate PyO3 0.28 conversion implementations — `IntoPyObject<'py>` for both `T` and `&T`, plus `FromPyObject<'a, 'py>` — so that the Rust enum can appear directly in `#[pyfunction]`, `#[pymethods]`, and `#[pyclass]` field signatures without manual extraction or conversion code.
- **FR-007**: Round-trip conversion (Rust → Python → Rust) MUST preserve variant identity for every variant of every supported base type.
- **FR-008**: The library MUST construct each exposed Python enum class at most once per Python interpreter instance, caching the resulting class object for reuse across all subsequent conversions and accesses within that interpreter.
- **FR-009**: Cached class construction MUST be safe under concurrent access by Python threads holding the GIL, such that no caller observes a partially-constructed class and no class is constructed more than once per interpreter.
- **FR-010**: The library MUST integrate with PyO3 0.28's module registration workflow such that exposing a Rust enum to Python requires no more than declaring the derive and invoking a single library-provided registration call from inside a user-written `#[pymodule]` block.
- **FR-010a**: The library MUST provide a generic registration helper — spelled as an extension method (`m.add_enum::<MyEnum>()?`) and/or a free function (`pyenum::add_enum::<MyEnum>(&m)?`) — that takes the module handle and the derived enum type and adds the Python class to the module. The derive MUST emit whatever trait implementation this helper requires, so the user writes exactly one line per enum in `#[pymodule]`. The derive MUST NOT rely on hidden global registries, `inventory`/`ctor`-style static registration, or a replacement module attribute macro.
- **FR-011**: Error surfaces MUST be layered: declaration-time errors surface as Rust compile errors with variant-level locality; runtime errors crossing the Python boundary MUST raise standard Python exceptions (`TypeError`, `ValueError`) with messages that identify the enum type and the offending input.
- **FR-012**: The derive MUST be compatible with Rust enums that carry explicit discriminant values (e.g., `Red = 1`) and MUST use those values as the Python enum member values where the chosen base requires explicit values.
- **FR-012a**: For variants declared without an explicit discriminant, the derive MUST assign default values equivalent to Python's `enum.auto()` for the selected base: 1-based incrementing integers for `Enum` and `IntEnum`; successive powers of two (1, 2, 4, 8, …) for `Flag` and `IntFlag`; and the variant's declared name (as a string) for `StrEnum`. Mixing explicit and defaulted variants within the same enum MUST be supported, with defaults continuing the auto-sequence from the last assigned value, consistent with Python's own `enum.auto()` behavior.
- **FR-013**: The library MUST document (via rustdoc and at least one worked example per base type) exactly how to declare, expose, and consume Rust-defined Python enums, including the expected Python-side behavior.
- **FR-014**: The Python enum member name MUST be identical to the Rust variant identifier, byte-for-byte (e.g., `HttpOk` in Rust → `MyEnum.HttpOk` and `MyEnum["HttpOk"]` in Python). The derive MUST NOT apply automatic casing transformation (no PascalCase → `UPPER_SNAKE_CASE` rewrite). Variant-level renaming via a derive attribute MAY be introduced in a later version but is out of scope for v1.

### Key Entities *(include if feature involves data)*

- **Rust source enum**: The user's `enum` type declaration in Rust. Bears the derive attribute, consists of unit variants, and optionally carries explicit discriminant values and a base-type selector attribute argument.
- **Python enum class**: The runtime Python class object that is a subclass of the selected Python enum base. Constructed on demand, cached per interpreter, and exposed through the PyO3 module.
- **Enum class cache**: The per-interpreter association between a Rust enum type and its corresponding Python class object. Ensures construction happens exactly once and that all references share identity.
- **Conversion boundary**: The point at which a Rust enum variant becomes a Python enum member or vice versa — used implicitly whenever the Rust enum appears in a PyO3-exposed function signature.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A Rust developer can expose a new Rust enum to Python by adding a single derive attribute, with no manual conversion or registration code beyond what PyO3 already requires for a plain function.
- **SC-002**: Exposed classes pass 100% of an interoperability test suite that exercises the Python enum protocol (`isinstance`, iteration, lookup by name, lookup by value, aliasing, equality, hashing) plus base-specific behaviors (bitwise ops for flag types, integer arithmetic for `IntEnum`, string behavior for `StrEnum`) for every supported base type.
- **SC-003**: Round-trip conversion across the PyO3 boundary preserves variant identity with zero discrepancies across an exhaustive test covering every variant of every base type.
- **SC-004**: The first conversion of a given Rust enum constructs the Python class in under 2 ms for enums up to 32 variants and under 20 ms for enums up to 1,024 variants on commodity developer hardware; subsequent conversions complete in under 1 µs per call.
- **SC-005**: Every class of ill-formed Rust enum declaration (non-unit variant, generic/lifetime, base/value mismatch, empty enum) produces a compile-time error whose message names the specific variant (where applicable) and the rule violated, verified by a negative-test suite ("trybuild"-style).
- **SC-006**: Third-party Python libraries that dispatch on `isinstance(x, enum.Enum)` or on a specific base class (pydantic field validation, FastAPI request parsing, SQLAlchemy `Enum` column, Python `match`/`case` patterns) accept enums produced by the library without special adaptation in at least one end-to-end integration test per library.
- **SC-007**: Documentation includes at least one copy-pasteable worked example per supported Python base type, and a reader following only the documentation can ship a working Rust-defined Python enum in under 15 minutes.

## Assumptions

- The library targets PyO3 0.28 as a single version. Multi-version support is not offered in v1 — cargo's `pyo3-ffi` `links = "python"` rule prevents multiple PyO3 lines from coexisting as optional dependencies in the same graph.
- The library targets Python 3.11+ so that `enum.StrEnum` (introduced in 3.11) is available without a polyfill. Earlier Python versions are out of scope for v1.
- Python enum construction is performed via Python's own functional `Enum(...)` API invoked through PyO3; no attempt is made to interact with unexposed CPython C-level enum internals.
- Rust-side methods defined via `impl` on the source enum are NOT automatically projected onto the Python class in v1; users who need method exposure continue to use existing PyO3 mechanisms on a separate helper type. This is explicitly out of scope to keep v1 focused.
- Aliasing behavior follows Python enum semantics: variants with equal values become aliases of the first-declared variant. The library preserves this behavior rather than redefining it.
- Per-interpreter caching relies on `pyo3::sync::PyOnceLock` (the 0.28 successor to the now-deprecated `GILOnceCell`). Behavior under the free-threaded (`--disable-gil`) Python build is not guaranteed in v1 and must be documented.
- The exposed Python class is owned by the Python module that registers it; no standalone "export without a module" path is promised in v1.
- The development environment has a working Rust toolchain (stable channel) and a Python interpreter suitable for building PyO3 extensions (maturin or equivalent). Setup of that toolchain is outside the library's scope.
- Consumers write their own `#[pymodule]` registration; the library provides the derive and conversion plumbing but does not take over module registration.