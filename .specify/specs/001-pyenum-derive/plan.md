# Implementation Plan: pyenum — Rust-Defined Python Enums for PyO3

**Branch**: `001-pyenum-derive` | **Date**: 2026-04-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/.specify/specs/001-pyenum-derive/spec.md`

## Summary

Deliver a Rust procedural macro (`#[derive(PyEnum)]`) plus a small runtime crate that lets PyO3 0.28 authors expose a Rust `enum` as a *true subclass* of the user-selected `enum.Enum` / `IntEnum` / `StrEnum` / `Flag` / `IntFlag` Python base. The macro emits a `PyEnum` trait implementation (per-interpreter cached Python class built via the functional `enum.<Base>("Name", members)` API) plus the PyO3 0.28 conversion impls (`IntoPyObject<'py>` for `T` and `&T`, `FromPyObject<'a, 'py>`), so the enum drops into `#[pyfunction]`, `#[pymethods]`, and `#[pyclass]` field signatures with no manual extraction code. Users register a derived type from within their own `#[pymodule]` block via a single explicit call: `pyenum::add_enum::<MyEnum>(&m)?`.

## Technical Context

**Language/Version**: Rust stable, edition 2024. Workspace-level `rust-version = 1.94` (per `[workspace.package]`).
**Primary Dependencies**:
- `pyo3 = { version = "0.28", features = ["abi3-py311"] }` — single version. A multi-version cargo feature matrix was explored and withdrawn because cargo's `pyo3-ffi` `links = "python"` rule disallows two PyO3 lines coexisting in one graph even as mutually exclusive optional deps.
- `syn = "2"`, `quote = "1"`, `proc-macro2 = "1"` (proc-macro implementation)
- Dev-only: `trybuild = "1"` (compile-fail tests), `pyo3` with the `auto-initialize` feature for Rust-side integration tests, `maturin` for Python-side builds, `criterion` for benchmarks.
**Storage**: N/A — in-process cache only (`GILOnceCell<Py<PyType>>` per exposed enum type)
**Testing** (layered — Rust compile-time + Rust unit/integration + Python end-to-end against the real cdylib):
- **Rust compile-fail tests**: `trybuild` snapshot fixtures under `crates/pyenum-derive/tests/ui/{accept,fail}/`. Drives US5 (compile-time rejection of ill-formed enums) — these run with `cargo test` and require no Python interpreter.
- **Rust unit tests**: `#[cfg(test)] mod tests { … }` inline in `crates/pyenum/src/*.rs` for private-item coverage (spec parsing helpers, name-collision set lookup, etc.) via `cargo test -p pyenum`.
- **Rust integration tests** (`PyEnum` trait public-API level): `crates/pyenum/tests/{cache,convert,from_py}.rs` — flat layout, each file a separate Cargo integration test binary. Drives cache identity / single-construction and round-trip conversion at the Rust API surface using `pyo3` with the `auto-initialize` feature.
- **Rust benchmarks**: `crates/pyenum/benches/cache.rs` using `criterion` — covers SC-004 targets (first-build and cache-hit latency).
- **Python end-to-end tests** (pytest): `tests/` at the repo root — imports the `pyenum-test` cdylib that our own `#[derive(PyEnum)]` produced and asserts the resulting Python classes satisfy the full enum protocol per base, round-trip through `#[pyfunction]` signatures, and interoperate with pydantic / FastAPI / SQLAlchemy / `match`/`case`. There is no pure-Python reference implementation — the Rust extension is the sole system under test on the Python side. The dependency on CPython's `enum` module is exercised transitively through the extension.
- **cdylib fixture**: `crates/pyenum-test/` — workspace crate (`publish = false`) that registers one derived enum per supported base plus edge-case fixtures. `tests/conftest.py` builds it via `maturin develop --manifest-path crates/pyenum-test/Cargo.toml` in a session-scoped fixture, then imports it for the rest of the suite.
- **Coverage**: `cargo-llvm-cov` for Rust (threshold ≥ 80%); `pytest-cov` for the Python suite (threshold ≥ 80% over `pyenum_test_ext`, which transitively exercises every code path emitted by the derive).
**Target Platform**: Any OS where PyO3 0.28 builds (Linux / macOS / Windows). CPython 3.11+; free-threaded (`--disable-gil`) builds explicitly out of scope for v1.
**Project Type**: Rust library (two published crates: `pyenum` runtime facade + `pyenum-derive` proc-macro) distributed on crates.io. A third workspace crate `pyenum-test` is a cdylib fixture used only by the Python integration suite and is `publish = false`.
**Performance Goals**: First construction of a 32-variant enum < 2 ms; 1,024-variant enum < 20 ms; subsequent conversions < 1 µs steady-state (SC-004).
**Constraints**: Must not call into CPython's unexposed enum internals; all Python-class construction goes through the functional `Enum("Name", members, ...)` API imported at runtime. Must be thread-safe under the GIL via `pyo3::sync::PyOnceLock`. Must produce `rustc` diagnostics with `span` locality on the offending variant for every compile-time rejection in FR-004/FR-004a.
**Scale/Scope**: Library targets ~200–500 LOC of proc-macro + ~400–800 LOC of runtime. Scope bounded by spec FR-001..FR-014; method projection, runtime-constructed enums, and free-threaded Python are explicitly deferred.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution at `.specify/memory/constitution.md` is still the unfilled template. In the absence of ratified principles, the following industry-default gates apply; revisit once a real constitution is ratified.

| Gate | Status | Evidence |
|------|--------|----------|
| Library-first (self-contained, testable, documented) | PASS | Deliverable is two library crates; no binary, no service. Each crate builds and tests in isolation. |
| Test-first / TDD | PASS | See TDD Plan below — Red-Green-Refactor enforced per task. |
| Integration testing for every external contract | PASS | Contracts in `contracts/` each get a failing integration test before implementation. |
| Simplicity / YAGNI | PASS | v1 explicitly defers method projection, renaming, auto-registration, and free-threaded Python. Only the 2-crate split is adopted; a 3rd `pyenum-core` crate is *not* introduced until a shared-logic need manifests. |
| Observability | N/A | In-process library; no runtime logs or metrics surface. |

No violations to justify. Complexity Tracking table remains empty.

## TDD Plan (MANDATORY)

*GATE: Must pass before Phase 0 research. The plan is rejected if any row is unchecked or left as placeholder.*

Tests are MANDATORY for this project. Every user story and every non-trivial behavior described in the spec MUST be driven by a failing test before implementation. The implementation phase will follow the Red-Green-Refactor cycle:

- **RED** — Write a test that captures the desired behavior. Run it and confirm it fails for the expected reason.
- **GREEN** — Write the minimum production code to make the test pass. No untested behavior.
- **REFACTOR** — Clean up structure, naming, and duplication with all tests green; re-run the full suite.

| Check | Answer |
|-------|--------|
| [x] Test framework(s) chosen and installed | Rust: `cargo test` + `trybuild` (compile-fail snapshots). Python integration: `pytest` + `pytest-cov`. Extension built by `maturin develop` during test bootstrap. |
| [x] Unit / integration / contract test layout decided | Rust unit tests inline via `#[cfg(test)]` in `crates/pyenum/src/*.rs`; Rust integration tests at `crates/pyenum/tests/{cache,convert,from_py}.rs`; compile-fail fixtures at `crates/pyenum-derive/tests/ui/{accept,fail}/`; Python pytest suite at `tests/` (repo root) importing the `crates/pyenum-test` cdylib built on-demand by conftest via maturin. |
| [x] Coverage tool chosen | Rust: `cargo-llvm-cov`. Python: `pytest-cov` (emits both terminal summary and `coverage.xml` for CI artifact upload). |
| [x] Minimum coverage threshold declared (≥ 80%) | value: **80%** combined (Rust lines ≥ 80% via `cargo-llvm-cov --fail-under-lines 80`; Python integration ≥ 80% via `pytest --cov --cov-fail-under=80`). |
| [x] CI step that runs tests + coverage on every push | where: `.github/workflows/test.yml` (one job each for `cargo test`, `cargo-llvm-cov`, `cargo clippy -- -D warnings`, `trybuild`, and `pytest` with coverage; matrix over Python 3.11 / 3.12 / 3.13). |
| [x] Every user story in spec.md has at least one acceptance / integration test planned | **yes** — US1/US2/US3 map to `tests/test_protocol_<base>.py` + `tests/test_conversion.py`; US4 (cache) maps to `crates/pyenum/tests/cache.rs` + `tests/test_cache.py`; US5 (compile-time rejection) maps to `crates/pyenum-derive/tests/ui/fail/*.rs`. |
| [x] Every external contract in `/contracts` has a failing contract test planned | **yes** — `derive-contract.md` (trybuild accept/reject fixtures), `trait-contract.md` (Rust unit + doctest against `PyEnum` trait), `registration-contract.md` (pytest fixture importing `pyenum::add_enum`-registered module). |
| [x] Red-Green-Refactor cycle will be enforced in tasks.md via `[RED]` / `[GREEN]` / `[REFACTOR]` labels | **yes** |

## Project Structure

### Documentation (this feature)

```text
.specify/specs/001-pyenum-derive/
├── plan.md                              # this file
├── research.md                          # Phase 0 decisions (PyO3 0.25–0.28 API matrix, compatibility shim, cache primitive, trybuild layout)
├── data-model.md                        # Entities: Rust source enum, Python class, cache, conversion boundary
├── quickstart.md                        # End-to-end worked example (5-minute happy path)
├── contracts/
│   ├── derive-contract.md               # What `#[derive(PyEnum)]` accepts/rejects + emitted items
│   ├── trait-contract.md                # The `PyEnum` trait + `PyEnumSpec` metadata contract
│   └── registration-contract.md         # `add_enum::<T>(&module)` helper contract
├── checklists/
│   └── requirements.md                  # Already created by /speckit-specify
└── tasks.md                             # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code (repository root)

The repository root *is* the workspace root; its existing `Cargo.toml` is rewritten as a `[workspace]` manifest and all crates move under `crates/`. The pre-existing `src/main.rs` and top-level `src/` directory are deleted (legacy scaffolding).

```text
pyenum/                                  # repo root = workspace root (existing)
├── Cargo.toml                           # [workspace] manifest; members = ["crates/pyenum", "crates/pyenum-derive", "crates/pyenum-test"]
├── README.md                            # project overview, install, 1-minute example (NEW; required by crates.io)
├── LICENSE                              # MIT license text (NEW)
├── CLAUDE.md                            # agent context — plan reference kept in sync
├── pyproject.toml                       # dev/test deps only (pytest, pytest-cov, maturin, pydantic, fastapi, sqlalchemy). NOT a publishable Python package.
├── crates/
│   ├── pyenum/                          # runtime facade crate — published as `pyenum` on crates.io
│   │   ├── Cargo.toml                   # license = "MIT"; depends on pyo3 = "0.28" (abi3-py311) and pyenum-derive (path)
│   │   ├── src/
│   │   │   ├── lib.rs                   # crate docs; public re-exports; `__private` module for derive output
│   │   │   ├── trait_def.rs             # `trait PyEnum` + `PyEnumSpec`, `PyEnumBase`, `VariantLiteral`. Unit tests inline via `#[cfg(test)]`.
│   │   │   ├── cache.rs                 # `get_or_build(py, &PyOnceLock<Py<PyType>>, &PyEnumSpec)` accessor.
│   │   │   ├── construct.rs             # `fn build_py_enum(py, spec) -> PyResult<Bound<'_, PyType>>` — the functional `enum.<Base>("Name", members)` call
│   │   │   └── register.rs              # `add_enum::<T>(m: &Bound<'_, PyModule>) -> PyResult<()>` + `PyModuleExt::add_enum`
│   │   ├── tests/                       # Cargo integration tests — flat layout, one binary per file
│   │   │   ├── cache.rs                 # US4 — identity + single-construction assertion via `Python::with_gil`
│   │   │   ├── convert.rs               # US3 — round-trip per base
│   │   │   └── from_py.rs               # FR-011 — TypeError on foreign objects; domain errors passed through
│   │   └── benches/
│   │       └── cache.rs                 # `criterion` bench covering SC-004 (first build ≤ 2 ms / 20 ms, steady-state ≤ 1 µs)
│   ├── pyenum-derive/                   # proc-macro crate — published as `pyenum-derive`
│   │   ├── Cargo.toml                   # license = "MIT"; [lib] proc-macro = true; dev-deps: trybuild, pyenum (path = "../pyenum")
│   │   ├── src/
│   │   │   ├── lib.rs                   # `#[proc_macro_derive(PyEnum, attributes(pyenum))]` entry point
│   │   │   ├── parse.rs                 # syn-based parsing of enum + `#[pyenum(base = "…", name = "…")]`
│   │   │   ├── validate.rs              # compile-time rejections: reserved-name (future: non-unit / generic / lifetime / empty / base-value-mismatch)
│   │   │   ├── codegen.rs               # emit `impl PyEnum` + `IntoPyObject` / `FromPyObject` impls referring to `pyenum::__private` only
│   │   │   └── reserved.rs              # Python keywords ∪ enum-reserved member names ∪ enum-special dunders
│   │   └── tests/
│   │       └── ui/
│   │           ├── accept/              # compile-success fixtures (one per accepted input class)
│   │           └── fail/                # compile-fail fixtures (one per rejection class) + *.stderr snapshots
│   └── pyenum-test/                     # cdylib fixture for the Python pytest suite — `publish = false`, workspace-only
│       ├── Cargo.toml                   # [lib] crate-type = ["cdylib"]; depends on pyenum (path = "../pyenum"); pyo3 with `extension-module` feature
│       └── src/
│           └── lib.rs                   # `#[pymodule] fn pyenum_test(m)`: registers one `#[derive(PyEnum)]` enum per supported base + edge-case fixtures (aliases, zero-flag, mixed explicit/auto) + `#[pyfunction]`s exercising the conversion boundary + an instrumented single-construction counter exposed to pytest
├── tests/                               # Python pytest suite — verifies the Python enums produced by the Rust `pyenum-test` cdylib
│   ├── conftest.py                      # session-scoped fixture: `maturin develop --manifest-path crates/pyenum-test/Cargo.toml --quiet`; imports `pyenum_test`; exposes the construction-counter helper
│   ├── test_protocol_enum.py            # US1 + US2 — `Enum`-based class: `isinstance(.., enum.Enum)`, iteration order, name/value lookup
│   ├── test_protocol_intenum.py         # US2 — `IntEnum`: subclass of `enum.IntEnum`, integer arithmetic, comparison with plain `int`
│   ├── test_protocol_strenum.py         # US2 — `StrEnum`: subclass of `enum.StrEnum`, string operations, auto-value = variant name
│   ├── test_protocol_flag.py            # US2 — `Flag`: bitwise composition, explicit zero-member case per Q3
│   ├── test_protocol_intflag.py         # US2 — `IntFlag`: bitwise + integer arithmetic
│   ├── test_auto_values.py              # Q1 — `auto()` resolution per base; mixed explicit/defaulted sequences
│   ├── test_aliases.py                  # alias preservation — duplicate values: second variant is an alias of the first
│   ├── test_name_passthrough.py         # Q2 — Rust variant name surfaces unchanged as Python member name (`MyEnum.HttpOk`, `MyEnum["HttpOk"]`)
│   ├── test_conversion.py               # US3 — `#[pyfunction]`s exported by the extension: round-trip + `TypeError` on foreign object
│   ├── test_cache.py                    # US4 — class identity stable across repeated accesses; instrumented single-construction counter is 1 after N round-trips
│   ├── test_registration.py             # Q4 — `add_enum::<T>(&m)` and `m.add_enum::<T>()` both attach the class under `T::SPEC.name`
│   ├── test_interop_pydantic.py         # SC-006 slice — `BaseModel` field with the enum, JSON round-trip
│   ├── test_interop_fastapi.py          # SC-006 slice — request model accepts enum value
│   ├── test_interop_sqlalchemy.py       # SC-006 slice — `Column(Enum(MyEnum))` persists and reloads (sqlite in-memory)
│   └── test_interop_match.py            # Python `match`/`case` dispatches on members
├── .github/workflows/test.yml           # CI: cargo fmt check / clippy -D warnings / cargo test / cargo-llvm-cov / trybuild / pytest; matrix = {pyo3 feature: 0_25/0_26/0_27/0_28} × {Python: 3.11/3.12/3.13} × {OS: Ubuntu/macOS}
└── .specify/specs/001-pyenum-derive/    # this feature's spec-kit artifacts (documented above)
```

**Structure Decision**: Cargo workspace rooted at the repo root. Three crates under `crates/`:
- `pyenum` — runtime facade, published. Single `pyo3 = "0.28"` dep with the `abi3-py311` feature.
- `pyenum-derive` — proc-macro, published. Lives in its own crate because `proc-macro = true` crates cannot expose non-macro items. Output references `pyenum::__private::*` only.
- `pyenum-test` — cdylib fixture for the Python integration suite, `publish = false`, workspace-only.

No `core` crate, no `compat` shim, no `prelude`: the reserved-name list is proc-macro-local, pyo3 is referenced directly, and the runtime crate exposes only the handful of names end users need (`PyEnum` derive, `PyEnumTrait`, `PyEnumBase`, `PyEnumSpec`, `VariantLiteral`, `add_enum`, `PyModuleExt`).

Rust tests are split across two layers: `#[cfg(test)]` blocks in `src/*.rs` for private-item unit tests, and flat `crates/pyenum/tests/*.rs` (not yet populated — pending US3/US4 RED tasks) for public-API integration tests.

**PyO3 version policy** (see Clarifications Q6 in spec.md): v1 targets **PyO3 0.28 exclusively**. A multi-version feature matrix was drafted and withdrawn — cargo's `pyo3-ffi` `links = "python"` rule rejects graphs with more than one PyO3 line, even when the alternates are mutually exclusive optional deps. Supporting additional versions in the future would require a separate-crate strategy (one publishable crate per PyO3 line).

**Python tests live in a flat `tests/` directory at the repo root**, with no separate pure-Python reference implementation. The `pyenum-test` cdylib — produced by our own `#[derive(PyEnum)]` — is the sole system under test on the Python side. CPython's own `enum` module is the upstream reference; there is nothing to gain from reimplementing its behavior in Python.

- `tests/conftest.py` runs a session-scoped `maturin develop --manifest-path crates/pyenum-test/Cargo.toml` to (re)build and install the cdylib into the active interpreter, then imports it. Subsequent test modules `import pyenum_test` and assert on the classes, functions, and counters it exposes.
- `pyproject.toml` at the repo root declares only dev/test dependencies (pytest, pytest-cov, maturin, pydantic, fastapi, sqlalchemy). It is explicitly marked as not publishable — this project ships Rust crates, not a Python package.

TDD loop for the Python-observable surface:
1. **RED** — add a test in `tests/` asserting the expected protocol behavior against `pyenum_test`.
2. **GREEN** — iterate on the Rust side (`crates/pyenum`, `crates/pyenum-derive`, `crates/pyenum-test`) until `cargo build` and `maturin develop` produce a cdylib that satisfies the assertion.
3. **REFACTOR** — tighten Rust implementation with the test suite green.

For the Rust-only surface (proc-macro validation, cache logic) TDD lives entirely inside `cargo test`: `crates/pyenum-derive/tests/ui/fail/*.rs` for compile-fail cases, `crates/pyenum/tests/*.rs` for runtime behavior. These run without any Python interpreter.

Legacy cleanup: `src/main.rs` and the root-level `src/` directory are removed. The initial workspace commit rewrites the existing `Cargo.toml` as `[workspace]`. `README.md`, `LICENSE` (MIT), and `pyproject.toml` are added at the repo root.

Doctest note: Rust-side `///` examples that require a live Python interpreter use ` ```rust,ignore ` fences and are covered instead by the `crates/pyenum/tests/*.rs` integration binaries — `cargo test --doc` stays green without linking against Python.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified.**

No violations. Table intentionally empty.
