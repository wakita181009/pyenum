# Phase 0 Research: pyenum (001)

**Feature**: [spec.md](./spec.md) · **Plan**: [plan.md](./plan.md) · **Date**: 2026-04-20

Every decision below closes a `NEEDS CLARIFICATION` slot in Technical Context, a `research task` implied by the plan's "Primary Dependencies", or a best-practices question raised during planning. For each, we state the chosen direction, the reasoning, and the alternatives considered and rejected. Decisions are binding for Phase 1 and for `/speckit-tasks`.

---

## R1 — Python class construction path

- **Decision**: Construct each Python enum class by **importing `enum` and calling the functional API**: `enum.Enum("Name", members, type=<base>)` (equivalently `enum.IntFlag("Name", members)` etc. for the specialized bases). Members are passed as a `list[tuple[name, value]]` built from Rust variant metadata. No C-level enum internals are touched.
- **Rationale**: Aligns with spec Assumptions ("Python enum construction is performed via Python's own functional `Enum(...)` API"). Functional API is public, stable since 3.4, covers all five base types, preserves aliasing semantics automatically (duplicate values become aliases), and is the only portable path since CPython does not expose enum construction at the C API level. Using the `type=` keyword ensures the resulting class is a subclass of the exact base we want, without us subclassing in Rust.
- **Alternatives rejected**:
  - *Subclass `enum.Enum` via `type.__call__` or C-level `tp_new`* — unsupported by CPython's public C API; would couple us to internal layout.
  - *Re-implement enum semantics in Rust and present a `#[pyclass]`* — returns us to the exact failure mode the library exists to fix (`isinstance(x, enum.Enum)` is `False`).
  - *Generate a Python source fragment and `exec()` it* — equivalent expressiveness, more moving parts, harder to reason about string escaping of variant names.

## R2 — Per-interpreter cache primitive

- **Decision**: Use `pyo3::sync::GILOnceCell<Py<PyType>>` as the sole cache primitive. Each `#[derive(PyEnum)]`-emitted `impl PyEnum for MyEnum` owns a `static CACHE: GILOnceCell<Py<PyType>> = GILOnceCell::new();` (inside a `fn py_enum_class<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyType>>` associated method). First call initialises via R1's construction path under the GIL; all subsequent calls return the cached reference.
- **Rationale**: `GILOnceCell` exists explicitly for this pattern in PyO3 ≥ 0.19 and remains present in 0.28. It handles the "first thread wins, everyone else blocks and then reads" contract under the GIL without the caller writing `Mutex`/`OnceLock` glue. The cell is keyed *by location* (per `static`), which gives us free per-interpreter freshness in the single-interpreter case and correct semantics in the sub-interpreter case because PyO3 0.28's `GILOnceCell` is interpreter-aware.
- **Alternatives rejected**:
  - *`std::sync::OnceLock<Py<PyType>>`* — not interpreter-aware; holding a `Py<…>` across sub-interpreter finalisation is a use-after-free risk.
  - *Global `HashMap<TypeId, Py<PyType>>` behind a `Mutex`* — an extra hash lookup on the hot path and a shared lock contention point across unrelated enums.
  - *`lazy_static!` / `once_cell::sync::Lazy`* — cannot initialise under the GIL safely; initialisation can happen on any thread.

## R3 — PyO3 0.28 conversion trait surface

- **Decision**: Implement **`IntoPyObject`** (for `T` and `&T`) and **`FromPyObject`** for every type that derives `PyEnum`. Emission happens from the proc-macro so users never see the boilerplate. `IntoPyObject::Target = PyAny`; `IntoPyObject::Error = PyErr`.
- **Rationale**: PyO3 0.28 replaced the deprecated `ToPyObject` / `IntoPy` pair with `IntoPyObject`/`IntoPyObjectRef` as the canonical one-way conversion trait; `FromPyObject` remains unchanged. Implementing both directions covers every signature position where the enum can appear (arguments, return values, `#[pyclass]` fields, iteration, setitem). Because we always convert to/from a `Py<PyType>` retrieved from R2's cache, the conversion is O(1) after first use.
- **Alternatives rejected**:
  - *Only derive `IntoPyObject`, let users call `MyEnum::extract(obj)` manually* — defeats FR-006's "no manual conversion code" requirement.
  - *Implement deprecated `ToPyObject`/`IntoPy`* — warns on 0.28, scheduled for removal; ties us to the wrong trait surface for the stated baseline.

## R4 — Registration helper shape

- **Decision**: Provide both (a) a free function `pub fn add_enum<T: PyEnum>(m: &Bound<'_, PyModule>) -> PyResult<()>` and (b) a blanket-impl extension trait `PyModuleExt` in `pyenum::prelude` giving `m.add_enum::<T>()?`. Both delegate to the same logic: resolve the cached `Py<PyType>` via `T::py_enum_class(py)`, then `m.add(T::NAME, class)?`.
- **Rationale**: Matches Q4 clarification verbatim. The free function is the unambiguous canonical form for generated code and docs; the extension trait is the ergonomic form users reach for inside `#[pymodule]`. Keeping logic in one place avoids drift.
- **Alternatives rejected**:
  - *Auto-registration via `inventory`/`linkme`/`ctor`* — explicitly rejected in Q4 for hidden-global / ordering reasons.
  - *Replacement `#[pyenum_module]` attribute macro* — explicitly rejected in Q4; unnecessary surface area for v1.

## R5 — Proc-macro parsing and validation

- **Decision**: Parse with `syn::DeriveInput` + `syn::Data::Enum`. For each variant, require `Fields::Unit`. Extract optional discriminant via `Variant::discriminant`; only accept `ExprLit` (integer for int-valued bases, string for `StrEnum`); reject arbitrary `const`/`call` expressions (keep v1 scope bounded). Attribute `#[pyenum(base = "IntEnum", name = "PublicName")]` parsed with `syn::meta::ParseNestedMeta`. Reserved-name set (Q5) stored in `pyenum-derive/src/reserved.rs` as a sorted `&[&str]` scanned via binary search per variant.
- **Rationale**: `syn 2` is the standard and matches PyO3 0.28's own proc-macro expectations. Rejecting expression discriminants for v1 keeps validation deterministic; spec FR-012 only requires *explicit values* be honoured, not arbitrary expressions. Binary-searching the reserved set is O(log n) per variant, trivial even at 1,024 variants.
- **Alternatives rejected**:
  - *Accept any `Expr` discriminant and evaluate at runtime* — expands both the failure surface and the acceptance test matrix; not required by spec.
  - *`darling` for attribute parsing* — extra dependency for a derive that takes one-to-two attribute keys; not worth the dep.
  - *`HashSet` for reserved names* — bigger code size in a proc-macro, no measurable speedup for this volume.

## R6 — Base-type selection syntax

- **Decision**: `#[pyenum(base = "Enum" | "IntEnum" | "StrEnum" | "Flag" | "IntFlag")]`. String-valued attribute arg, parsed case-sensitively to match Python's own class names. Default is `"Enum"` if omitted. Exactly one base allowed per derive.
- **Rationale**: Strings match the Python side 1:1 so there is no second vocabulary to document. Keeps the attribute surface minimal (one key, one value) for v1. A dedicated path-valued variant (`base = pyenum::IntEnum`) is tempting but would require users to import a Rust-side marker type and buys nothing in v1.
- **Alternatives rejected**:
  - *Dedicated keyword (`#[pyenum(int_enum)]`)* — no extensibility path without breaking change; also five keywords to learn instead of one.
  - *Path-valued* — premature abstraction.

## R7 — Default value generation for unset discriminants (Q1)

- **Decision**: For variants without an explicit discriminant, codegen injects `enum.auto()` as the variant's value in the `members` list passed to the functional API. CPython's own `auto()` then resolves it per-base (`Enum`/`IntEnum` → incrementing ints, `Flag`/`IntFlag` → powers of two, `StrEnum` → name string). We do **not** pre-compute values in Rust; deferring to `enum.auto()` guarantees behaviour-matches Python exactly and keeps the derive's codegen trivial.
- **Rationale**: Matches Q1 clarification. Eliminates an entire class of bugs where Rust-computed defaults would drift from CPython's (notably `StrEnum.value` defaulting to the lowercase-member-name in CPython 3.12+; we inherit whatever behaviour the running Python has).
- **Alternatives rejected**:
  - *Precompute ints in Rust* — must then match CPython's exact rules per base *and* handle mixed explicit/default sequences; strictly more surface and more risk.
  - *Require explicit discriminants* — rejected by Q1 Option B.

## R8 — Name collision and variant-name pass-through (Q2, Q5)

- **Decision**: The proc-macro takes the Rust variant identifier *as-is* for the Python member name (Q2). Before emission it checks each identifier against a compiled-in sorted list comprising:
  - All Python 3.13 keywords (`class`, `def`, `None`, `True`, `False`, `return`, `import`, `lambda`, `match`, `case`, `pass`, `yield`, `async`, `await`, `global`, `nonlocal`, `and`, `or`, `not`, `if`, `elif`, `else`, `for`, `while`, `with`, `as`, `from`, `try`, `except`, `finally`, `raise`, `is`, `in`, `break`, `continue`, `del`, `assert`, `return`)
  - Enum-module reserved member names (`_name_`, `_value_`, `_missing_`, `_generate_next_value_`, `_ignore_`, `_order_`, `name`, `value`)
  - Dunders that `enum.EnumType` interprets specially (`__init__`, `__new__`, `__class__`, `__members__`, `__init_subclass__`, `__set_name__`, `__class_getitem__`, `__repr__`, `__str__`, `__hash__`, `__eq__`, `__format__`, `__dir__`, `__bool__`, `__reduce_ex__`)
  - A collision triggers `compile_error!` with span on the offending variant and a message like: `variant `class` collides with a Python keyword; rename the Rust variant or expose it under a different name`.
- **Rationale**: Matches Q5 clarification. Pass-through (Q2) is preserved; the forbidden set exists precisely to surface what pass-through would otherwise defer to a Python import-time crash.
- **Alternatives rejected**:
  - *Auto-prefix with `_` on collision* — rejected in Q5.
  - *Allow opt-out via `#[pyenum(allow_reserved)]`* — rejected for v1 scope.

## R9 — Trybuild snapshot strategy

- **Decision**: `pyenum-derive/tests/ui/` split into `accept/` (files that must compile) and `fail/` (files that must fail to compile with the expected stderr captured in `*.stderr` snapshots). CI runs `cargo test --test ui` which invokes `trybuild`. Snapshots are regenerated explicitly via `TRYBUILD=overwrite cargo test --test ui` — never silently.
- **Rationale**: Standard Rust proc-macro testing pattern. Keeps compile-fail fixtures close to the macro that produces the error and makes every reserved-name, base/value-mismatch, and non-unit-variant case an auditable file in source control.
- **Alternatives rejected**:
  - *Inline `#[test]` with `compile_error!` expectations* — impossible; compile errors halt compilation of the test binary.
  - *`compiletest-rs`* — abandoned upstream for stable Rust.

## R10 — Python integration-test harness

- **Decision**: `tests/python/` is a cdylib PyO3 extension crate that depends on `pyenum` via path. `conftest.py` runs `maturin develop --manifest-path tests/python/Cargo.toml --quiet` during the pytest session-scope fixture, then imports `pyenum_test_ext`. The extension intentionally registers one enum per supported base plus edge cases (aliases, explicit zero-flag, mixed explicit/auto defaults).
- **Rationale**: Produces real `isinstance(x, enum.Enum)` checks against a real import, which is the only way to verify FR-003 end-to-end. Session-scoped build amortises the cost across the whole suite. Having a single extension register every fixture enum keeps the test matrix sparse and easy to review.
- **Alternatives rejected**:
  - *Pure Rust PyO3 `Python::with_gil` tests that construct modules programmatically* — works, but doesn't exercise the `#[pymodule]` registration path (FR-010/FR-010a) which is a key contract.
  - *Separate extension per test file* — build-time blowup; no isolation benefit (process starts fresh anyway).

## R11 — Coverage tooling

- **Decision**: Rust side: `cargo-llvm-cov` with `--fail-under-lines 80` wired into the CI coverage job. Python side: `pytest-cov` with `--cov=pyenum_test_ext --cov-fail-under=80` (noting that "Python coverage" here measures our integration-test glue, not user code). Reports uploaded as artefacts, no third-party coverage service assumed.
- **Rationale**: `cargo-llvm-cov` produces accurate line coverage for proc-macro output plus runtime, works out of the box on stable, and `--fail-under-lines` turns the 80% bar into a hard gate. `pytest-cov`'s `--cov-fail-under` does the same on the Python side.
- **Alternatives rejected**:
  - *`tarpaulin`* — slower and less reliable than `llvm-cov` on modern toolchains.
  - *`grcov`* — extra setup; `cargo-llvm-cov` wraps the same underlying machinery more ergonomically.

## R12 — CI matrix & Rust MSRV

- **Decision**: GitHub Actions job matrix: `{ os: [ubuntu-latest, macos-latest], python: [3.11, 3.12, 3.13], rust: [stable, 1.82 (MSRV)] }`. Windows not in v1 CI to keep the matrix tractable; users may build locally. MSRV pinned at `1.82` because edition 2024 requires `1.82+` and so does PyO3 0.28.
- **Rationale**: Covers the three Python versions we support (3.11 floor per spec Assumptions), two platforms where PyO3 0.28 wheels are first-class, and both stable and MSRV Rust to catch accidental reliance on post-MSRV features.
- **Alternatives rejected**:
  - *Include `windows-latest` at v1* — low signal for a proc-macro crate; re-evaluate after v1 ships if users report issues.
  - *Track `nightly`* — not needed; we use no unstable features.

## Open Questions

None. Every `NEEDS CLARIFICATION` from Technical Context is resolved above. Sub-interpreter finalisation behaviour and free-threaded (`--disable-gil`) Python behaviour remain explicitly out of scope for v1 per spec Assumptions — documented but not gated.
