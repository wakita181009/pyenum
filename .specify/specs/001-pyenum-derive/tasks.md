---
description: "Task list for pyenum feature 001 — dependency-ordered, TDD Red-Green-Refactor enforced"
---

# Tasks: pyenum — Rust-Defined Python Enums for PyO3

**Input**: Design documents from `/.specify/specs/001-pyenum-derive/`
**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md)

**Tests**: Test tasks are MANDATORY. Every user story MUST include tests written BEFORE the implementation, following the Red-Green-Refactor TDD cycle.

**TDD Cycle (REQUIRED for every implementation task)**:

1. **RED**: Write a failing test. Run it and confirm it fails for the expected reason.
2. **GREEN**: Write the minimum production code to make the failing test pass. Do not add untested behavior.
3. **REFACTOR**: Improve structure/naming/duplication while keeping all tests green. Re-run the full suite after each refactor step.

**Organization**: Tasks are grouped by user story (US1–US5 from [spec.md](./spec.md)) to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] [TDD?] Description with exact file path`

- **[P]**: Can run in parallel (different files, no incomplete-task dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- **[RED] / [GREEN] / [REFACTOR]**: Mandatory on every implementation-side task inside a user-story or polish phase

## Path Conventions (per [plan.md](./plan.md) Project Structure)

- Rust runtime: `crates/pyenum/src/...`, `crates/pyenum/tests/...`, `crates/pyenum/benches/...`
- Proc-macro: `crates/pyenum-derive/src/...`, `crates/pyenum-derive/tests/ui/{accept,fail}/`
- cdylib fixture: `crates/pyenum-test/src/...`
- Python pytest suite: `tests/...` (repo root, pure Python)
- CI / tooling: `.github/workflows/`, root `Cargo.toml`, root `pyproject.toml`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Convert the repository to a Cargo workspace with the three-crate layout, add licensing/README/pyproject/CI scaffolding, and land an empty-but-compilable skeleton for every file the rest of the plan depends on.

- [x] T001 Delete legacy `src/main.rs` and the top-level `src/` directory; update `.gitignore` if needed
- [x] T002 Rewrite the root `Cargo.toml` as a `[workspace]` manifest with `members = ["crates/pyenum", "crates/pyenum-derive", "crates/pyenum-test"]` and a shared `[workspace.package]` block (license = "MIT", edition = "2024", rust-version = "1.75")
- [x] T003 [P] Create `crates/pyenum/Cargo.toml` declaring `[features] default = ["pyo3-0_28"]; pyo3-0_25/0_26/0_27/0_28` with `pyo3` marked `optional = true` and gated per feature; depend on `pyenum-derive` via path — shipped pinned to `pyo3 = "0.28"` only; multi-version feature matrix obsoleted by Q6 (see Phase 8 note on T094–T100)
- [x] T004 [P] Create `crates/pyenum-derive/Cargo.toml` with `[lib] proc-macro = true`, `syn = "2"`, `quote = "1"`, `proc-macro2 = "1"`, and dev-deps `trybuild` + `pyenum` (path)
- [x] T005 [P] Create `crates/pyenum-test/Cargo.toml` with `[lib] crate-type = ["cdylib"]`, `publish = false`, pyo3 feature passthrough to `pyenum`, and the pyo3 `extension-module` feature
- [x] T006 [P] Write `LICENSE` at the repo root (MIT text with the project's copyright line)
- [x] T007 [P] Write `README.md` at the repo root summarising the project, linking to `.specify/specs/001-pyenum-derive/`, and embedding the minimal quickstart snippet from [quickstart.md](./quickstart.md)
- [x] T008 [P] Write `pyproject.toml` at the repo root declaring the dev/test dependency group only (pytest, pytest-cov, maturin, pydantic, fastapi, sqlalchemy, httpx) with `publishable = false` equivalent (no `[project]` dist metadata, or `[project].name = "pyenum-dev"` marked internal) — root `pyproject.toml` ships `[project] name = "pyenum-test"` (internal fixture name) + maturin build + pytest + test dep group
- [x] T009 [P] Scaffold `.github/workflows/test.yml` with a matrix skeleton (`pyo3-feature = [pyo3-0_25, pyo3-0_26, pyo3-0_27, pyo3-0_28]` × `python = [3.11, 3.12, 3.13]` × `os = [ubuntu-latest, macos-latest]`) — jobs for `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --features $pyo3-feature --no-default-features`, and `pytest` will be filled in later phases — pyo3-feature axis collapsed to single pinned version per Q6
- [ ] T010 [P] Create empty module files so the skeleton compiles: `crates/pyenum/src/lib.rs`, `crates/pyenum/src/prelude.rs`, `crates/pyenum/src/compat.rs`, `crates/pyenum/src/trait_def.rs`, `crates/pyenum/src/cache.rs`, `crates/pyenum/src/construct.rs`, `crates/pyenum/src/convert.rs`, `crates/pyenum/src/register.rs` (each with a `//! module placeholder` doc-comment) — `prelude.rs`, `compat.rs`, `convert.rs` were never created (obsoleted by pinning to PyO3 0.28 + merging conversion codegen into the derive output); remaining files all exist
- [x] T011 [P] Create empty proc-macro module files: `crates/pyenum-derive/src/lib.rs`, `crates/pyenum-derive/src/parse.rs`, `crates/pyenum-derive/src/validate.rs`, `crates/pyenum-derive/src/codegen.rs`, `crates/pyenum-derive/src/reserved.rs`
- [x] T012 [P] Create empty cdylib entry point `crates/pyenum-test/src/lib.rs` with `#[pymodule] fn pyenum_test<'py>(m: &pyo3::Bound<'py, pyo3::types::PyModule>) -> pyo3::PyResult<()> { Ok(()) }`
- [x] T013 [P] Create `crates/pyenum-derive/tests/ui.rs` as the trybuild runner entry point; add empty `crates/pyenum-derive/tests/ui/accept/` and `crates/pyenum-derive/tests/ui/fail/` directories with `.gitkeep` — landed as `crates/pyenum-derive/tests/trybuild.rs`
- [x] T014 [P] Create `tests/conftest.py` with a session-scoped `pyenum_test` fixture that runs `maturin develop --manifest-path crates/pyenum-test/Cargo.toml --quiet` once before the suite and then `import pyenum_test`

**Checkpoint**: `cargo check --workspace --no-default-features --features pyo3-0_28` succeeds; `pytest tests/` runs (empty suite OK).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish the cross-cutting plumbing every user story needs before it can be implemented — the exactly-one-feature guard, the compatibility shim, and the data types that appear in every subsequent task.

**⚠️ CRITICAL**: No user-story work may begin until this phase is complete. Every test in Phase 3+ assumes these modules compile and export stable names.

- [x] ~~T015 Implement the exactly-one-`pyo3-0_XX`-feature guard in `crates/pyenum/src/lib.rs` using `#[cfg(all(feature = "pyo3-0_25", feature = "pyo3-0_26", ...))] compile_error!(...)` for each pairwise conflict and a symmetric "none-active" guard~~ — OBSOLETED by Q6 (single pinned PyO3 line)
- [x] ~~T016 Implement `crates/pyenum/src/compat.rs` with per-version type aliases (`PyTypeRef`, `ModuleArg<'py>`, `PyAnyRef<'py>`), the `OnceCell<T>` re-export (`pyo3::sync::GILOnceCell` on 0.26+, equivalent on 0.25), and an empty `macro_rules! version_shim!` stub — feature-gated with `#[cfg(feature = "pyo3-0_XX")]` only here, nowhere else~~ — OBSOLETED by Q6
- [x] T017 [P] Implement `crates/pyenum-derive/src/reserved.rs` — compile-time sorted `&[&str]` covering: Python 3.13 keywords, enum-reserved member names (`_name_`, `_value_`, `_missing_`, `_generate_next_value_`, `_ignore_`, `_order_`, `name`, `value`), and enum-special dunders (`__init__`, `__new__`, `__class__`, `__members__`, `__init_subclass__`, `__set_name__`, `__class_getitem__`, `__repr__`, `__str__`, `__hash__`, `__eq__`, `__format__`, `__dir__`, `__bool__`, `__reduce_ex__`); expose `fn is_reserved(name: &str) -> Option<ReservedKind>` using binary search
- [x] T018 [P] Define `PyEnumBase` (enum with variants `Enum`, `IntEnum`, `StrEnum`, `Flag`, `IntFlag`), `VariantLiteral` (`Int(i64)`, `Str(&'static str)`, `Auto`), and `PyEnumSpec` (with `name`, `base`, `variants`) in `crates/pyenum/src/trait_def.rs` per [data-model.md](./data-model.md)
- [x] T019 [P] Declare the `PyEnum` trait skeleton in `crates/pyenum/src/trait_def.rs` per [contracts/trait-contract.md](./contracts/trait-contract.md) — three methods + `const SPEC`; provide default impls that return `PyErr::new::<PyRuntimeError, _>("not implemented")` so the trait compiles without a concrete derive
- [x] T020 [P] Expose a `#[doc(hidden)] pub mod __private` in `crates/pyenum/src/lib.rs` that re-exports the names the proc-macro will reference (initially empty; populated per-story)
- [x] T021 [P] Wire the `#[proc_macro_derive(PyEnum, attributes(pyenum))]` entry point in `crates/pyenum-derive/src/lib.rs`; the initial body emits a no-op `impl PyEnum` so the derive exists even before parsing/codegen are real
- [x] T022 Wire `crates/pyenum-derive/tests/ui.rs` to run `trybuild::TestCases::new().compile_fail("tests/ui/fail/*.rs"); .pass("tests/ui/accept/*.rs")` — the empty directories from T013 keep it green
- [x] T023 Fill the `pytest` and `cargo` matrix cells in `.github/workflows/test.yml` so CI invokes `cargo check --workspace --no-default-features --features $pyo3-feature` on every matrix cell and uploads `cargo-llvm-cov` + `pytest-cov` coverage artefacts (thresholds wired up in polish phase) — pyo3-feature axis collapsed to single pinned version per Q6
- [x] ~~T024 Run `cargo check --workspace --no-default-features --features pyo3-0_25` and repeat for `0_26`, `0_27`, `0_28`; every feature combination MUST compile before any user story begins~~ — OBSOLETED by Q6

**Checkpoint**: Workspace compiles on all four `pyo3-0_XX` features. `cargo test --workspace` and `pytest tests/` both run cleanly (empty). Trybuild runner wired.

---

## Phase 3: User Story 1 - Expose a Rust enum as a Python `Enum` subclass (Priority: P1) 🎯 MVP

**Goal**: `#[derive(PyEnum)] enum Color { Red, Green, Blue }` registered into a `#[pymodule]` produces a Python class where `issubclass(Color, enum.Enum)` is `True` and every variant is a member.

**Independent Test**: Build `pyenum-test`, import it in Python, assert `isinstance(pyenum_test.Color.Red, enum.Enum)` and `list(pyenum_test.Color) == [Color.Red, Color.Green, Color.Blue]`.

### Tests for User Story 1 (MANDATORY - RED phase) ⚠️

> **TDD RULE**: Write these tests FIRST. Run them and confirm they FAIL for the expected reason before any implementation.

- [x] T025 [P] [US1] [RED] Add trybuild accept fixture `crates/pyenum-derive/tests/ui/accept/minimal_unit_enum.rs` exercising `#[derive(PyEnum)]` on a unit-variant enum — must fail today (proc-macro emits only no-op impl) — landed as `basic_enum.rs`
- [ ] T026 [P] [US1] [RED] Add Rust integration test `crates/pyenum/tests/subclass.rs` using `pyo3::Python::with_gil` + `auto-initialize` to build `Color`'s class and assert `class.is_subclass(py.import("enum")?.getattr("Enum")?)?` — must fail — covered functionally by `tests/test_protocol_enum.py`; the pure-Rust `crates/pyenum/tests/` harness is still empty
- [x] T027 [P] [US1] [RED] Register a `#[derive(PyEnum)] enum Color { Red, Green, Blue }` plus `m.add_enum::<Color>()?` in `crates/pyenum-test/src/lib.rs` — currently fails to build because `add_enum` is unimplemented
- [x] T028 [P] [US1] [RED] Add `tests/test_protocol_enum.py` asserting: `issubclass(pyenum_test.Color, enum.Enum)`, `list(pyenum_test.Color)` order matches declaration, `pyenum_test.Color["Red"]` returns the member, `pyenum_test.Color(1)` returns the first variant — must fail (module cannot import)

### Implementation for User Story 1 (GREEN phase)

- [x] T029 [US1] [GREEN] Implement `syn`-based unit-enum parsing in `crates/pyenum-derive/src/parse.rs`: walk `Data::Enum.variants`, collect `(ident, discriminant)` in declaration order; panic-free errors for non-`Fields::Unit` are deferred to US5 (return an error sentinel for now so US1 accept fixture compiles)
- [x] ~~T030 [US1] [GREEN] Fill `crates/pyenum/src/compat.rs` aliases for `pyo3-0_28` first (`type ModuleArg<'py> = &'py Bound<'py, PyModule>; type PyTypeRef = Py<PyType>; pub use pyo3::sync::GILOnceCell as OnceCell;`) so downstream code compiles on the default feature~~ — OBSOLETED: compat shim dropped; runtime uses `pyo3::sync::PyOnceLock` + `Bound`/`Py` types directly
- [x] T031 [US1] [GREEN] Implement `build_py_enum` in `crates/pyenum/src/construct.rs`: import `enum`, call `Enum(name, [(name_i, value_i), …])`, return `Py<PyType>` — passes T026 when combined with T032–T034
- [x] T032 [US1] [GREEN] Implement the per-type `GILOnceCell` accessor helper in `crates/pyenum/src/cache.rs` (`fn get_or_build<'py>(py, once, spec) -> PyResult<Bound<'py, PyType>>`)
- [x] T033 [US1] [GREEN] Implement `add_enum::<T>(m)` free function + `PyModuleExt::add_enum` blanket impl in `crates/pyenum/src/register.rs` to pass T027
- [x] T034 [US1] [GREEN] Emit the minimal `impl PyEnum for MyEnum` (with `SPEC` + `py_enum_class` + placeholder `to_py_member`/`from_py_member`) from `crates/pyenum-derive/src/codegen.rs` to pass T025 and T026
- [x] T035 [US1] [GREEN] Populate `pyenum::__private` in `crates/pyenum/src/lib.rs` with the symbols the derive output references (`PyEnumSpec`, `PyEnumBase`, `VariantLiteral`, `OnceCell`, `build_py_enum`, trait path) — keeps proc-macro output version-agnostic
- [x] T036 [US1] [GREEN] Run `maturin develop --manifest-path crates/pyenum-test/Cargo.toml` inside the pytest fixture; rerun `pytest tests/test_protocol_enum.py` and confirm T028 passes

### Refactor for User Story 1 (REFACTOR phase)

- [ ] T037 [US1] [REFACTOR] Extract the `(name, value)` member-list construction helper in `crates/pyenum/src/construct.rs` so US2 can reuse it; rerun `cargo test --workspace` + `pytest tests/test_protocol_enum.py`
- [x] ~~T038 [US1] [REFACTOR] Confirm no feature-gated `cfg` attrs exist outside `crates/pyenum/src/compat.rs` via `rg "#\[cfg\(feature\s*=\s*\"pyo3-" crates/pyenum/src/ -g '!compat.rs'` (should print nothing); rerun full test suite~~ — OBSOLETED by Q6 (no feature-gated `cfg` attrs anywhere)
- [ ] T039 [US1] [REFACTOR] Verify `cargo-llvm-cov --package pyenum --fail-under-lines 80` passes for the US1-touched modules

**Checkpoint**: User Story 1 fully functional on `pyo3-0_28`. MVP shippable once polish-phase docs land. Other pyo3 features still pending (land during US2–US5).

---

## Phase 4: User Story 2 - Support all five standard Python enum base types (Priority: P1)

**Goal**: `#[derive(PyEnum)] #[pyenum(base = "IntEnum")]` (and the other four bases) yields a Python class that is a subclass of the requested base and passes the base-specific protocol.

**Independent Test**: For each base, the corresponding fixture enum in `pyenum_test` produces a Python class that is `issubclass(cls, enum.<Base>)` and passes base-specific operations (bitwise for flag types, integer arithmetic for `IntEnum`, string ops for `StrEnum`).

### Tests for User Story 2 (MANDATORY - RED phase) ⚠️

- [ ] T040 [P] [US2] [RED] Add trybuild accept fixtures: `crates/pyenum-derive/tests/ui/accept/int_enum.rs`, `str_enum.rs`, `flag.rs`, `int_flag.rs` — each fails because `#[pyenum(base = …)]` is not parsed yet — partial: `int_discriminants.rs`, `strenum_auto_lowercase.rs`, `strenum_explicit_value.rs` landed; dedicated `flag.rs` / `int_flag.rs` accept fixtures still pending
- [x] T041 [P] [US2] [RED] Register one enum per base in `crates/pyenum-test/src/lib.rs` (e.g., `HttpStatus` / `Greeting` / `Perms` / `BitPerms`) — fails to build because attribute is unrecognised — landed as `HttpStatus` / `Greeting` / `Permission` / `BitPerms`
- [x] T042 [P] [US2] [RED] Add `tests/test_protocol_intenum.py` asserting `issubclass(HttpStatus, enum.IntEnum)`, integer arithmetic, `HttpStatus(200) == 200` — must fail
- [x] T043 [P] [US2] [RED] Add `tests/test_protocol_strenum.py` asserting `issubclass(Greeting, enum.StrEnum)`, string concatenation, `.value` equals the declared (or auto-derived) string — must fail
- [x] T044 [P] [US2] [RED] Add `tests/test_protocol_flag.py` asserting `issubclass(Perms, enum.Flag)`, bitwise composition `(Read | Write) & Read == Read`, explicit zero-member presence when declared — must fail
- [x] T045 [P] [US2] [RED] Add `tests/test_protocol_intflag.py` asserting `issubclass(BitPerms, enum.IntFlag)`, bitwise + integer arithmetic together — must fail
- [ ] T046 [P] [US2] [RED] Add `tests/test_auto_values.py` covering Q1: `Enum`/`IntEnum` auto → 1-based ints, `Flag`/`IntFlag` auto → powers of two, `StrEnum` auto → variant name; mixed explicit/defaulted variants continue correctly — must fail
- [ ] T047 [P] [US2] [RED] Add `tests/test_aliases.py` asserting that declaring two variants with the same explicit value produces one canonical member and one alias (`SameValue(1) is SameValue.First`) — must fail — superseded in spirit by Phase 7.5 HIGH #2: pyenum-derive now rejects alias-creating variants at compile time rather than surfacing them; this Python-side alias fixture is now redundant
- [ ] T048 [P] [US2] [RED] Add `tests/test_name_passthrough.py` (Q2) asserting `pyenum_test.Color.Red` works and `pyenum_test.Color["Red"]` resolves — currently passes for US1 but belongs logically with US2 naming semantics; ensure it runs — name-passthrough assertions live inside `tests/test_protocol_enum.py` already; a dedicated file is still pending

### Implementation for User Story 2 (GREEN phase)

- [x] T049 [US2] [GREEN] Parse `#[pyenum(base = "…", name = "…")]` in `crates/pyenum-derive/src/parse.rs` using `syn::meta::ParseNestedMeta`; reject unknown keys and duplicates (unknown-key handling fully exercised in US5)
- [x] T050 [US2] [GREEN] Validate base/value literal compatibility in `crates/pyenum-derive/src/validate.rs`: integer literals only for `Enum`/`IntEnum`/`Flag`/`IntFlag`; auto-only for `StrEnum` in v1; route via `VariantLiteral` — `StrEnum` now also accepts explicit `#[pyenum(value = "...")]` literals (Phase 7.5 HIGH #3)
- [x] T051 [US2] [GREEN] Extend `crates/pyenum/src/construct.rs` to import the correct `enum` base attribute (`enum.IntEnum`, `enum.StrEnum`, `enum.Flag`, `enum.IntFlag`) based on `PyEnumSpec.base`
- [x] T052 [US2] [GREEN] Emit `enum.auto()` for `VariantLiteral::Auto` in the member list constructed by `construct.rs` — CPython resolves values per-base, giving Q1 for free
- [x] T053 [US2] [GREEN] Update `crates/pyenum-derive/src/codegen.rs` to embed the chosen `PyEnumBase` variant in `SPEC` and emit each variant's `VariantLiteral` correctly
- [x] ~~T054 [US2] [GREEN] Verify alias behaviour passes by running T047 against the built `pyenum_test` — fix in `construct.rs` if CPython's alias semantics surface differently through the functional API~~ — OBSOLETED by Phase 7.5 HIGH #2 (alias-creating variants now rejected at compile time)
- [x] T055 [US2] [GREEN] Rerun pytest protocol tests (T042–T047) after `maturin develop`; confirm all green

### Refactor for User Story 2 (REFACTOR phase)

- [ ] T056 [US2] [REFACTOR] Consolidate base-dispatch logic (attribute name → `PyEnumBase`, `PyEnumBase` → `enum.*` attr name) into a single table in `crates/pyenum/src/trait_def.rs`; rerun full test suite
- [ ] T057 [US2] [REFACTOR] Verify `cargo-llvm-cov --package pyenum --fail-under-lines 80` and pytest coverage ≥ 80% over US2-touched modules

**Checkpoint**: User Stories 1 AND 2 independently functional. Five bases supported on `pyo3-0_28`.

---

## Phase 5: User Story 3 - Automatic bidirectional Rust ↔ Python conversion (Priority: P1)

**Goal**: A Rust enum type appears directly in `#[pyfunction]` / `#[pymethods]` / `#[pyclass]` field signatures and round-trips without manual conversion code, on every supported `pyo3-0_XX` feature.

**Independent Test**: Python calls a `#[pyfunction]` that takes and returns a derived enum; the returned member is the expected one; passing a foreign object raises `TypeError`. Equivalent Rust-side test via `Python::with_gil`.

### Tests for User Story 3 (MANDATORY - RED phase) ⚠️

- [ ] T058 [P] [US3] [RED] Add Rust integration test `crates/pyenum/tests/convert.rs`: builds Color, round-trips every variant through `IntoPyObject`/`FromPyObject` under `Python::with_gil`, asserts identity preserved — fails (convert.rs empty) — currently covered by `tests/test_conversion.py` only; pure-Rust harness still empty
- [ ] T059 [P] [US3] [RED] Add Rust integration test `crates/pyenum/tests/from_py.rs`: builds Color, calls `FromPyObject` with a foreign Python object (a plain `int`), asserts `PyTypeError` is raised with a message containing `"Color"` — fails — ditto; covered from Python side only
- [x] T060 [P] [US3] [RED] Add `#[pyfunction] fn roundtrip(c: Color) -> Color` and `fn to_int(c: HttpStatus) -> i64` to `crates/pyenum-test/src/lib.rs`, exposing them in the module — fails to build — landed as `color_roundtrip` / `http_roundtrip` + peers for every base
- [x] T061 [P] [US3] [RED] Add `tests/test_conversion.py` calling `pyenum_test.roundtrip(Color.Red) is Color.Red`, `pyenum_test.roundtrip(Color.Green)`, and asserting `pyenum_test.roundtrip(42)` raises `TypeError` with class-name context — must fail
- [ ] T062 [P] [US3] [RED] Add `tests/test_registration.py` (Q4) asserting both `m.add_enum::<T>()` and `pyenum::add_enum::<T>(&m)` surface the same class under `T::SPEC.name` — fails

### Implementation for User Story 3 (GREEN phase)

- [x] T063 [US3] [GREEN] Implement `impl<'py> IntoPyObject<'py> for T: PyEnum` and `impl<'py> IntoPyObject<'py> for &T` in `crates/pyenum/src/convert.rs` (delegating to `T::to_py_member`) — gated to `pyo3-0_26+` via `compat` — landed inside the derive codegen output instead of a standalone `convert.rs`
- [x] T064 [US3] [GREEN] Implement `impl<'py> FromPyObject<'py> for T: PyEnum` in `crates/pyenum/src/convert.rs` delegating to `T::from_py_member` — passes T058, T059, T061 — emitted by `pyenum-derive/src/codegen.rs`
- [x] T065 [US3] [GREEN] Implement `T::to_py_member` codegen in `crates/pyenum-derive/src/codegen.rs`: `match self { MyEnum::X => py_enum_class(py)?.getattr("X")? … }`
- [x] T066 [US3] [GREEN] Implement `T::from_py_member` codegen: check `obj.is_instance(py_enum_class(py)?)?`, map back via name or `.value` comparison; raise `PyTypeError::new_err(format!("expected {}, got {}", T::SPEC.name, obj.get_type().name()?))` otherwise
- [x] T067 [US3] [GREEN] Verify Q4 `add_enum` path in `crates/pyenum/src/register.rs` matches T062 exactly; adjust free-fn vs extension-method bodies to share implementation

### Refactor for User Story 3 (REFACTOR phase)

- [ ] T068 [US3] [REFACTOR] Factor the shared "resolve cached class then look up member" logic used by both `to_py_member` and `from_py_member` into a helper in `crates/pyenum/src/convert.rs`; rerun full suite
- [ ] T069 [US3] [REFACTOR] Confirm T058/T059/T061 pass on `pyo3-0_28`; run `cargo-llvm-cov` coverage check for `crates/pyenum/src/convert.rs`

**Checkpoint**: All three P1 user stories pass on `pyo3-0_28`. Cross-version (0.25–0.27) wiring still pending for convert path (lands in polish-phase matrix verification).

---

## Phase 6: User Story 4 - One-time construction via cached singleton (Priority: P2)

**Goal**: Python class construction happens exactly once per interpreter per Rust enum type, even under 10,000 repeated conversions or concurrent access.

**Independent Test**: Rust-side — counter around `build_py_enum` reads 1 after 10k round-trips and after concurrent access from multiple GIL-holding threads. Python-side — counter exposed by `pyenum_test` reads 1 after 10k calls.

### Tests for User Story 4 (MANDATORY - RED phase) ⚠️

- [ ] T070 [P] [US4] [RED] Add Rust integration test `crates/pyenum/tests/cache.rs` installing a test-only counter hook in `pyenum::cache` (via a `#[cfg(test)] pub(crate) fn reset_counter()` / `read_counter()`), running 10k round-trips under `Python::with_gil`, asserting counter == 1 — fails (hook not present) — identity assertion covered from pytest; pure-Rust counter harness still not added
- [x] T071 [P] [US4] [RED] Add an `AtomicUsize` construction counter and helper `#[pyfunction] fn _construction_count(cls: &Bound<'_, PyType>) -> usize` to `crates/pyenum-test/src/lib.rs`, readable from pytest
- [x] T072 [P] [US4] [RED] Add `tests/test_cache.py`: 10k `pyenum_test.roundtrip(Color.Red)` calls + assert `pyenum_test._construction_count(pyenum_test.Color) == 1`; also assert `pyenum_test.Color is pyenum_test.Color` (identity via two imports) — must fail — landed in skeleton form; construction-count assertion deferred until T071 ships

### Implementation for User Story 4 (GREEN phase)

- [x] ~~T073 [US4] [GREEN] Add the test-only `construction_counter` helper in `crates/pyenum/src/cache.rs` (behind `#[cfg(test)]` inside the runtime crate; expose `pub(crate)` accessor). Increment inside `get_or_build` only when the closure actually runs (first call)~~ — OBSOLETED: `#[cfg(test)]` + `pub(crate)` is unreachable from integration tests in `crates/pyenum/tests/` (they compile without `cfg(test)`), and no unit test inside `src/cache.rs` exercised it. The single-construction contract is instead observed via the production counter in `crates/pyenum-test` (T074) and asserted from pytest.
- [x] T074 [US4] [GREEN] Wire the production counter in `crates/pyenum-test/src/lib.rs` using a module-level `AtomicUsize` incremented inside the `PyEnum::py_enum_class` path — exposed via T071
- [x] ~~T075 [US4] [GREEN] Confirm `compat::OnceCell::get_or_try_init` guarantees serialised single initialisation on every supported `pyo3-0_XX` feature; add feature-gated documentation note in `crates/pyenum/src/compat.rs`~~ — OBSOLETED by Q6; documentation now lives on `pyo3::sync::PyOnceLock` upstream
- [ ] T076 [US4] [GREEN] Rerun T070–T072 after rebuilding `pyenum_test`; all three must now pass

### Refactor for User Story 4 (REFACTOR phase)

- [ ] T077 [US4] [REFACTOR] Collapse any duplicated "read cache / build on miss" paths in `crates/pyenum/src/cache.rs` into a single entry point; rerun `cargo test --workspace` + `pytest`
- [ ] T078 [US4] [REFACTOR] Verify `cargo-llvm-cov --package pyenum` coverage of cache module ≥ 80%

**Checkpoint**: Single-construction contract enforced end-to-end.

---

## Phase 7: User Story 5 - Rejecting non-conforming Rust enums at compile time (Priority: P2)

**Goal**: Every malformed Rust enum — tuple/struct variant, generic, lifetime, empty, reserved-name, base/value mismatch, duplicate-or-unknown attribute — refuses to compile with a spanned, variant-named diagnostic.

**Independent Test**: Every fixture under `crates/pyenum-derive/tests/ui/fail/` fails compilation with the expected stderr snapshot.

### Tests for User Story 5 (MANDATORY - RED phase) ⚠️

- [x] T079 [P] [US5] [RED] Add trybuild fail fixtures for variant shape: `crates/pyenum-derive/tests/ui/fail/tuple_variant.rs`, `struct_variant.rs` — landed in commit `78ca3c3`
- [x] T080 [P] [US5] [RED] Add trybuild fail fixtures for generics/lifetime: `generic_enum.rs`, `lifetime_enum.rs` — landed in commit `78ca3c3`
- [x] T081 [P] [US5] [RED] Add trybuild fail fixture for empty enum: `empty_enum.rs` — landed in commit `78ca3c3`
- [ ] T082 [P] [US5] [RED] Add trybuild fail fixtures for reserved names: `reserved_keyword.rs` (variant `Class`), `reserved_enum_dunder.rs` (variant `__init__`), `reserved_enum_member.rs` (variant `_value_`) — **partial**: `reserved_name_value.rs` covers the `_value_`-equivalent (lowercase `value`); the dunder and Python-keyword cases remain to be added
- [x] T083 [P] [US5] [RED] Add trybuild fail fixture for base/value mismatch: `int_enum_with_string_value.rs`, `strenum_with_int.rs` — landed as `intenum_str_value.rs` + `strenum_int_discriminant.rs` in commit `78ca3c3`
- [ ] T084 [P] [US5] [RED] Add trybuild fail fixtures for attribute surface: `duplicate_base_attr.rs` (two `#[pyenum(base = …)]`), `unknown_pyenum_attr.rs` (`#[pyenum(bogus = 1)]`) — the rejection logic is live in `parse.rs`, trybuild snapshots still pending

### Implementation for User Story 5 (GREEN phase)

- [x] T085 [US5] [GREEN] Enforce unit-variant rejection with `syn::Error::new_spanned(variant, "...")` in `crates/pyenum-derive/src/validate.rs` — shipped with the core derive (commit `3c16e0a`)
- [x] T086 [US5] [GREEN] Enforce rejection of generics/lifetimes in `validate.rs` — commit `3c16e0a`
- [x] T087 [US5] [GREEN] Enforce non-empty-enum rejection in `validate.rs` — commit `3c16e0a`
- [x] T088 [US5] [GREEN] Enforce reserved-name rejection in `validate.rs` using `reserved::is_reserved` — commit `3c16e0a`
- [x] T089 [US5] [GREEN] Enforce base/value literal compatibility in `validate.rs` — commit `78ca3c3` (HIGH #3 review fix)
- [x] T090 [US5] [GREEN] Enforce duplicate and unknown `#[pyenum(...)]` attribute rejection in `crates/pyenum-derive/src/parse.rs` — commit `3c16e0a`
- [x] T091 [US5] [GREEN] Commit trybuild `.stderr` snapshots — 12 fail + 4 accept snapshots committed in `78ca3c3`

### Refactor for User Story 5 (REFACTOR phase)

- [ ] T092 [US5] [REFACTOR] Group validation rules by category inside `crates/pyenum-derive/src/validate.rs` (shape, identity, literal, attribute) with a single dispatcher; rerun `cargo test --test ui`
- [ ] T093 [US5] [REFACTOR] Confirm every `compile_error!` carries a span pointing at the offending variant or attribute literal (not at the enum as a whole) by reading the committed `.stderr` snapshots

**Checkpoint**: Compile-time rejection is the sole source of truth for every invalid input class.

---

## Phase 7.5: Post-Review Amendments (2026-04-20)

**Purpose**: Close the two HIGH-severity correctness gaps surfaced by the
`/rust-review` of `crates/` on 2026-04-20 — alias-creating variants (spec
SC "round-trip preserves variant identity" vs. Python's implicit aliasing
of equal-valued members) and the absence of an explicit-value escape
hatch for `StrEnum` (where Python's `auto()` lowercases the variant
name). See commit `78ca3c3`.

### HIGH #2 — Reject alias-creating Rust-side variants

- [x] T120 [RED] Add trybuild fail fixture `crates/pyenum-derive/tests/ui/fail/duplicate_str_value.rs` — two variants with the same `#[pyenum(value = "...")]` string
- [x] T121 [RED] Add trybuild fail fixture `crates/pyenum-derive/tests/ui/fail/duplicate_auto_lowercase.rs` — two `StrEnum` variants whose names lowercase to the same string (`Hello`/`HELLO`)
- [x] T122 [GREEN] Implement `check_duplicate_values` in `crates/pyenum-derive/src/validate.rs`: reject duplicate explicit string values and duplicate `StrEnum` auto-lowercased names; Rust itself already blocks duplicate integer discriminants so the int path is belt-and-braces only
- [x] T123 [GREEN] Commit trybuild `.stderr` snapshots for both fixtures

### HIGH #3 — Explicit `StrEnum` values without clobbering Python's auto semantics

- [x] T124 [RED] Add trybuild accept fixture `crates/pyenum-derive/tests/ui/accept/strenum_explicit_value.rs` — `#[pyenum(value = "Rust")]` preserves case
- [x] T125 [RED] Add trybuild accept fixture `crates/pyenum-derive/tests/ui/accept/strenum_auto_lowercase.rs` — documents the lowercasing behaviour as intentional
- [x] T126 [RED] Add trybuild fail fixture `crates/pyenum-derive/tests/ui/fail/value_and_discriminant.rs` — variant carries both `#[pyenum(value = "...")]` and a Rust discriminant (mutually exclusive)
- [x] T127 [GREEN] Parse variant-level `#[pyenum(value = "...")]` in `crates/pyenum-derive/src/parse.rs`; reject co-occurrence with a Rust discriminant
- [x] T128 [GREEN] Drop the `#[allow(dead_code)]` / "reserved for future use" notes on `VariantLiteral::Str` in `crates/pyenum/src/trait_def.rs` now that the derive emits it
- [x] T129 [GREEN] Add `Language` `StrEnum` fixture (explicit values) + `language_roundtrip` pyfunction in `crates/pyenum-test/src/lib.rs`
- [x] T130 [GREEN] Add `tests/test_protocol_language.py` + matching entries in `tests/pyenum_test.pyi` asserting explicit values are preserved verbatim
- [x] T131 [GREEN] Document the `StrEnum` auto-lowercasing rule + the explicit-value escape hatch in `README.md` (`### StrEnum values` subsection + updated rejection list)

### Review-loop follow-ups (deferred)

- [ ] T132 Complete the remaining T082 reserved-name trybuild fixtures (Python keyword case, dunder case) to match the original US5 coverage matrix
- [ ] T133 Complete T084 attribute-surface trybuild fixtures (`duplicate_base_attr.rs`, `unknown_pyenum_attr.rs`) against the existing `parse.rs` rejection logic
- [ ] T134 `cache.rs:30` — annotate `class.bind(py).clone()` with a comment clarifying that `Borrowed → Bound` requires `.clone()` under PyO3 0.28; revisit if a cheaper conversion lands upstream
- [ ] T135 `register.rs` — seal `PyModuleExt` via a private trait (the `Sealed` pattern) so external crates cannot implement it

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Close the remaining SC-004 (benchmarks), SC-006 (interop), SC-007 (docs) objectives; finalise CI + coverage gates.

> **Note**: Tasks T094–T100 were originally scoped to extend PyO3 support to the 0.25–0.27 lines behind a compatibility shim. They were **obsoleted** by the discovery that cargo's `pyo3-ffi` `links = "python"` rule rejects graphs containing more than one PyO3 line even when the alternates are mutually exclusive optional deps (see Clarification Q6 in spec.md). T094–T100 are intentionally left here as stricken-through placeholders for traceability; do NOT execute them.

- [x] ~~T094 Re-run under `pyo3-0_27` — OBSOLETED (see note above)~~
- [x] ~~T095 Fill `pyo3-0_27` compat branches — OBSOLETED~~
- [x] ~~T096 Re-run under `pyo3-0_26` — OBSOLETED~~
- [x] ~~T097 Fill `pyo3-0_26` compat branches — OBSOLETED~~
- [x] ~~T098 Re-run under `pyo3-0_25` — OBSOLETED~~
- [x] ~~T099 Fill `pyo3-0_25` compat branches — OBSOLETED~~
- [x] ~~T100 Sweep for `cfg(feature = "pyo3-...")` — OBSOLETED~~

### Interop (SC-006)

- [ ] T101 [P] [RED] Add `tests/test_interop_pydantic.py`: `pydantic.BaseModel` with an `HttpStatus` field accepts raw ints and enum members, round-trips through `.model_dump()` / `.model_validate()` — must fail before interop is wired (likely passes immediately; treat as regression guard)
- [ ] T102 [P] [RED] Add `tests/test_interop_fastapi.py`: minimal FastAPI app accepts an enum via a JSON request body using `httpx.TestClient`, asserts 200 and enum round-trip — must fail
- [ ] T103 [P] [RED] Add `tests/test_interop_sqlalchemy.py`: SQLAlchemy model with `Column(Enum(HttpStatus))` against an in-memory sqlite database, insert + reload, assert member identity — must fail
- [ ] T104 [P] [RED] Add `tests/test_interop_match.py`: Python `match`/`case` statement dispatches on members correctly — must fail (or serve as regression guard)
- [ ] T105 [GREEN] Run T101–T104 against the built extension; if any fail, diagnose whether the Rust-side convert path raises a compatible exception type or whether we need to adjust `from_py_member` — fix until all four pass

### Benchmarks (SC-004)

- [x] T106 [P] [RED] Add `crates/pyenum/benches/cache.rs` using `criterion` with two benches: first-build of 32-variant enum, first-build of 1,024-variant enum, and hot-path cache-hit conversion — fails to build (benches/ absent)
- [x] T107 [P] [GREEN] Configure `[bench]` stanza in `crates/pyenum/Cargo.toml`, author the benches; run `cargo bench` locally; capture numbers
- [x] T108 [REFACTOR] Compare measured numbers to SC-004 targets (< 2 ms first build / 32, < 20 ms / 1,024, < 1 µs steady-state); if any target is missed, file a performance note and decide whether to optimise (e.g. cache `enum` module import) before v1 release — measured 173 µs / 8.86 ms / 64 ns, all under target; note at `.specify/specs/001-pyenum-derive/perf-notes.md`

### Documentation (SC-007, FR-013)

- [ ] T109 [P] [RED] Add a rustdoc example per base type on `crates/pyenum/src/lib.rs` (at minimum one complete `#[pymodule]` example for each of `Enum`, `IntEnum`, `StrEnum`, `Flag`, `IntFlag`) using ` ```rust,ignore ` fences — fails `cargo doc --no-deps` if examples don't typecheck-as-text
- [ ] T110 [P] [GREEN] Author the rustdoc examples; run `cargo doc --no-deps --all-features` and verify no warnings
- [ ] T111 [P] Add a worked example per base type to `README.md` at the repo root (cross-link to `.specify/specs/001-pyenum-derive/quickstart.md` for the full walkthrough)
- [ ] T112 [P] Update `crates/pyenum/src/lib.rs` crate-level `//!` block to link to `.specify/specs/001-pyenum-derive/spec.md` and summarise the PyO3 version matrix

### CI + Coverage finalisation

- [x] T113 Wire `cargo-llvm-cov --workspace --fail-under-lines 90` into `scripts/coverage.sh` (invoked by the `coverage` job of `.github/workflows/test.yml`); fails the job if coverage regresses. Threshold raised from the plan's original 80% to 90% per user request (2026-04-20)
- [x] T114 Coverage gate covers both Rust and Python runs — `scripts/coverage.sh` aggregates `cargo test` and `pytest` under the same `cargo-llvm-cov` session before the `--fail-under-lines 90` check, so a Python-side regression trips the same gate without needing a separate `pytest --cov-fail-under` flag
- [x] ~~T115 Add a `cargo clippy --workspace --all-features -- -D warnings` job + `cargo fmt --check` job; ensure both block merge~~ — RESOLVED via pre-commit: `.pre-commit-config.yaml` runs `cargo fmt` + `cargo clippy -- -D warnings` on every commit; no separate CI job needed
- [x] ~~T116 [REFACTOR] Resolve every clippy warning surfaced by T115 without relaxing lints; rerun full suite~~ — covered by pre-commit (same hook set as T115)

### Final validation

- [ ] T117 Run `quickstart.md` end-to-end locally (fresh `cargo new`, add deps, copy the quickstart code, `maturin develop`, import, exercise); confirm everything described there actually works
- [ ] T118 Verify total test count: Rust unit tests + integration tests + trybuild fixtures + pytest modules all execute on `cargo test --workspace --features pyo3-0_28` and `pytest tests/`; capture the count in `.specify/specs/001-pyenum-derive/quickstart.md` as a sanity checkpoint
- [ ] T119 [REFACTOR] Final sweep: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo doc --no-deps`, `pytest tests/` — all green; update CHANGELOG entry under `[Unreleased]` summarising the public surface for v1.0.0

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion. BLOCKS all user stories. In particular T015–T024 unblock every subsequent story.
- **User Story 1 (Phase 3)**: Depends on Phase 2. MVP on completion (on `pyo3-0_28`).
- **User Story 2 (Phase 4)**: Depends on Phase 2; integrates naturally with US1 but designed to be independently testable (fixture extensions are additive).
- **User Story 3 (Phase 5)**: Depends on Phase 2; exercises the same cdylib as US1/US2 but tests conversion at the boundary, which is independent of how the class was built.
- **User Story 4 (Phase 6)**: Depends on Phase 2; technically also needs the US1 derive output (it hangs a counter off the cache path), but the cache primitive itself was stood up in US1 so US4 is purely additive instrumentation + proof.
- **User Story 5 (Phase 7)**: Depends on Phase 2; the trybuild fail fixtures explicitly need `crates/pyenum-derive/src/validate.rs` to begin rejecting inputs, which is a US5-local extension of the parse/codegen paths already present from US1/US2.
- **Polish (Phase 8)**: Depends on all user stories. The cross-version matrix closure (T094–T100) is the gate for v1 release.

### User Story Dependencies

- **US1** → no story dependencies.
- **US2** → logically extends US1 (base selection). Independently testable because each base has its own fixture enum in `pyenum-test`.
- **US3** → reuses the cache built in US1 but tests the conversion-trait surface; failure modes are independent.
- **US4** → adds instrumentation around US1's cache; zero functional coupling to US2/US3/US5.
- **US5** → zero functional coupling. Rejects things US1/US2 silently tolerate today. Can be implemented in parallel with US3/US4 by a second contributor once Phase 2 is done.

### Within Each User Story (TDD Red-Green-Refactor is MANDATORY)

- **RED**: Tests MUST fail for the expected reason before any production code in that story starts.
- **GREEN**: Write the minimum code to turn each failing test green.
- **REFACTOR**: Clean up with the full test suite green; re-run tests after every refactor step.
- Coverage ≥ 80% is verified at the end of each story phase.

### Parallel Opportunities

- All `[P]` Setup tasks (T003–T014) run in parallel after T001+T002 land.
- All `[P]` Foundational tasks (T017–T021) run in parallel after T015+T016.
- All `[RED]` tasks inside a single user-story phase are parallelisable — they live in distinct files and must all fail before any `[GREEN]` task starts.
- US2, US3, US4, US5 are parallelisable across contributors once Phase 2 is done.
- Phase 8 interop tasks (T101–T104) and documentation tasks (T109–T112) are all parallelisable.

---

## Parallel Example: User Story 1

```bash
# RED — kick off all failing tests for US1 concurrently:
Task: "T025 [US1] [RED] trybuild accept/minimal_unit_enum.rs"
Task: "T026 [US1] [RED] crates/pyenum/tests/subclass.rs"
Task: "T027 [US1] [RED] crates/pyenum-test/src/lib.rs registers Color"
Task: "T028 [US1] [RED] tests/test_protocol_enum.py"

# Confirm all four fail. Then GREEN, parallelising where files are independent:
Task: "T029 [US1] [GREEN] parse.rs unit-enum parsing"
Task: "T030 [US1] [GREEN] compat.rs pyo3-0_28 aliases"
Task: "T031 [US1] [GREEN] construct.rs build_py_enum"
Task: "T032 [US1] [GREEN] cache.rs get_or_build"
Task: "T033 [US1] [GREEN] register.rs add_enum + PyModuleExt"

# Then T034, T035, T036 sequentially (they compose prior outputs). REFACTOR at the end.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only, `pyo3-0_28` only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: Build `pyenum-test`, run `pytest tests/test_protocol_enum.py`, confirm green.
5. Tag `0.1.0-alpha` if shipping preview.

### Incremental Delivery

1. Setup + Foundational → foundation ready.
2. Add US1 → test independently → demo (MVP).
3. Add US2 → five bases supported → demo.
4. Add US3 → conversion boundary closes → demo.
5. Add US4 → single-construction proof → demo.
6. Add US5 → compile-time rejection complete → release candidate.
7. Polish (Phase 8) → cross-version matrix, interop, benchmarks, docs → v1.0.0 release.

### Parallel Team Strategy

With two+ contributors:

1. Both complete Phases 1+2 together (serial for review ergonomics).
2. Once Phase 2 is done:
   - Contributor A: US1 → US2 → US3 (the happy path, shared cdylib).
   - Contributor B: US5 (compile-time rejection, isolated in `pyenum-derive/src/validate.rs`).
3. After US1 lands, Contributor B can also pick up US4 (adds instrumentation around US1's cache).
4. Phase 8 cross-version matrix is a serial checklist owned by whoever finishes their story-track first.

---

## Notes

- `[P]` tasks = different files, no dependencies on incomplete tasks.
- `[Story]` label ties each task to a user story for traceability and independent testing.
- `[RED] / [GREEN] / [REFACTOR]` labels are MANDATORY on every implementation-side task inside Phase 3+.
- Tests are MANDATORY: every production behaviour is introduced by a previously failing test.
- Trybuild `.stderr` snapshots (T091) are committed to source control — their diff is the auditable record of every rejection message the library ships.
- Commit after each RED → GREEN → REFACTOR cycle; prefer one commit per task group.
- Version-matrix closure (T094–T100) is release-blocking: a failure on any `pyo3-0_XX` cell is treated as a v1 regression.
- Avoid: vague tasks, same-file parallel conflicts, cross-story dependencies that break independence, writing production code before a failing test exists, feature-gated `cfg` attrs leaking outside `crates/pyenum/src/compat.rs`.
