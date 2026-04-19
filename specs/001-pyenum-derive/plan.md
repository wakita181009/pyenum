# Implementation Plan: pyenum — Rust-Defined Python Enums for PyO3

**Branch**: `001-pyenum-derive` | **Date**: 2026-04-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-pyenum-derive/spec.md`

## Summary

Deliver a Rust procedural macro (`#[derive(PyEnum)]`) plus a small runtime crate that lets PyO3 0.28 authors expose a Rust `enum` as a *true subclass* of the user-selected `enum.Enum` / `IntEnum` / `StrEnum` / `Flag` / `IntFlag` Python base. The macro emits a `PyEnum` trait implementation (per-interpreter cached `Py<PyType>` built via Python's functional `Enum("Name", members)` API), plus PyO3 0.28 conversion impls (`IntoPyObject` + `FromPyObject`), so the enum drops into `#[pyfunction]`, `#[pymethods]`, and `#[pyclass]` field signatures with no manual extraction code. Users register a derived type from within their own `#[pymodule]` block via a single explicit call: `pyenum::add_enum::<MyEnum>(&m)?`.

## Technical Context

**Language/Version**: Rust stable (edition 2024, MSRV 1.82 — aligned with PyO3 0.28's own MSRV)
**Primary Dependencies**:
- `pyo3 = { version = "0.28", features = ["abi3-py311"] }` (baseline; Python 3.11+)
- `syn = "2"`, `quote = "1"`, `proc-macro2 = "1"` (proc-macro implementation)
- Dev-only: `trybuild = "1"` (compile-fail tests), `pyo3 = { ..., features = ["auto-initialize"] }` for Rust-side integration tests that spin up an interpreter, `maturin` for Python-side integration test builds
**Storage**: N/A — in-process cache only (`GILOnceCell<Py<PyType>>` per exposed enum type)
**Testing**:
- Rust unit tests: `cargo test` (built-in)
- Rust compile-fail tests: `trybuild` snapshot fixtures under `pyenum-derive/tests/ui/`
- Python integration tests: `pytest` driving a maturin-built extension module in `tests/python/`
- Coverage: `cargo-llvm-cov` for Rust, `coverage.py`/`pytest-cov` for Python integration suite
**Target Platform**: Any OS where PyO3 0.28 builds (Linux / macOS / Windows). CPython 3.11+; free-threaded (`--disable-gil`) builds explicitly out of scope for v1 per spec Assumptions.
**Project Type**: Rust library (two crates: runtime facade + proc-macro) distributed on crates.io; the extension built under `tests/python/` is only a fixture for integration tests, not a published artifact.
**Performance Goals**: First construction of a 32-variant enum < 2 ms; 1,024-variant enum < 20 ms; subsequent conversions < 1 µs steady-state (SC-004).
**Constraints**: Must not call into CPython's unexposed enum internals; all Python-class construction goes through the functional `Enum("Name", members, ...)` API imported at runtime. Must be thread-safe under the GIL via `GILOnceCell`. Must produce `rustc` diagnostics with `span` locality on the offending variant for every compile-time rejection in FR-004/FR-004a.
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
| [x] Unit / integration / contract test layout decided | `pyenum/tests/unit/` (runtime unit tests via `cargo test`), `pyenum-derive/tests/ui/` (trybuild compile-fail contract tests), `tests/python/` (pytest integration suite — protocol conformance per base + pydantic/FastAPI/SQLAlchemy/match interop) |
| [x] Coverage tool chosen | Rust: `cargo-llvm-cov`. Python: `pytest-cov` (emits both terminal summary and `coverage.xml` for CI artifact upload). |
| [x] Minimum coverage threshold declared (≥ 80%) | value: **80%** combined (Rust lines ≥ 80% via `cargo-llvm-cov --fail-under-lines 80`; Python integration ≥ 80% via `pytest --cov --cov-fail-under=80`). |
| [x] CI step that runs tests + coverage on every push | where: `.github/workflows/test.yml` (one job each for `cargo test`, `cargo-llvm-cov`, `cargo clippy -- -D warnings`, `trybuild`, and `pytest` with coverage; matrix over Python 3.11 / 3.12 / 3.13). |
| [x] Every user story in spec.md has at least one acceptance / integration test planned | **yes** — US1/US2/US3 map to `tests/python/test_protocol_<base>.py`; US4 (cache) maps to `pyenum/tests/unit/cache.rs` + a pytest instrumented-counter test; US5 (compile-time rejection) maps to `pyenum-derive/tests/ui/*.rs`. |
| [x] Every external contract in `/contracts` has a failing contract test planned | **yes** — `derive-contract.md` (trybuild accept/reject fixtures), `trait-contract.md` (Rust unit + doctest against `PyEnum` trait), `registration-contract.md` (pytest fixture importing `pyenum::add_enum`-registered module). |
| [x] Red-Green-Refactor cycle will be enforced in tasks.md via `[RED]` / `[GREEN]` / `[REFACTOR]` labels | **yes** |

## Project Structure

### Documentation (this feature)

```text
specs/001-pyenum-derive/
├── plan.md                              # this file
├── research.md                          # Phase 0 decisions (PyO3 0.28 APIs, trybuild layout, cache primitive)
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

```text
pyenum/                                  # workspace root
├── Cargo.toml                           # workspace manifest (members = ["pyenum", "pyenum-derive"])
├── pyenum/                              # runtime facade crate — published as `pyenum` on crates.io
│   ├── Cargo.toml                       # depends on pyo3 = "0.28", pyenum-derive (path = "../pyenum-derive")
│   ├── src/
│   │   ├── lib.rs                       # re-exports `PyEnum`, `add_enum`, derive; crate-level docs
│   │   ├── trait_def.rs                 # `trait PyEnum` + `PyEnumSpec`, `PyEnumBase` enum (Enum/IntEnum/…)
│   │   ├── cache.rs                     # `GILOnceCell<Py<PyType>>` wrapper + build path
│   │   ├── build.rs_notmod              # (example) `fn build_py_enum(py, spec) -> PyResult<Py<PyType>>` — functional `Enum("Name", members)` invocation
│   │   ├── convert.rs                   # blanket impls of `IntoPyObject` / `FromPyObject` via `PyEnum`
│   │   └── register.rs                  # `add_enum::<T>(&Bound<'_, PyModule>) -> PyResult<()>` + extension trait `PyModuleExt::add_enum`
│   └── tests/
│       └── unit/                        # Rust unit tests: cache invariants, base-value defaulting, name-collision set
├── pyenum-derive/                       # proc-macro crate — published as `pyenum-derive`
│   ├── Cargo.toml                       # [lib] proc-macro = true; dev-deps: trybuild, pyenum (path = "../pyenum")
│   ├── src/
│   │   ├── lib.rs                       # `#[proc_macro_derive(PyEnum, attributes(pyenum))]`
│   │   ├── parse.rs                     # syn-based parsing of enum + `#[pyenum(base = "…")]` attribute
│   │   ├── validate.rs                  # compile-time rejections: non-unit, generic/lifetime, reserved-name, empty, base/value mismatch
│   │   ├── codegen.rs                   # emit `impl PyEnum for …` + conversion trait impls
│   │   └── reserved.rs                  # shared constant: Python keywords + enum-reserved names + dunders
│   └── tests/
│       └── ui/                          # trybuild fixtures: accept/*.rs (compile + pass) and fail/*.rs (compile-fail + expected stderr)
├── tests/
│   └── python/                          # PyO3 extension fixture + pytest integration suite
│       ├── Cargo.toml                   # cdylib crate `pyenum_test_ext` depending on `pyenum`
│       ├── src/lib.rs                   # `#[pymodule]` registering one enum per supported base + edge cases
│       ├── conftest.py                  # pytest fixture: `maturin develop` + import the extension
│       ├── test_protocol_enum.py        # US1 + US2 for Enum
│       ├── test_protocol_intenum.py     # US2 for IntEnum
│       ├── test_protocol_strenum.py     # US2 for StrEnum
│       ├── test_protocol_flag.py        # US2 for Flag (incl. explicit zero-member case)
│       ├── test_protocol_intflag.py     # US2 for IntFlag
│       ├── test_conversion.py           # US3 round-trip + TypeError on wrong type
│       ├── test_cache.py                # US4 identity + single-construction assertion
│       ├── test_interop_pydantic.py     # SC-006 slice
│       ├── test_interop_fastapi.py      # SC-006 slice
│       ├── test_interop_sqlalchemy.py   # SC-006 slice
│       └── test_interop_match.py        # Python match/case compatibility
├── .github/workflows/test.yml           # CI: cargo test / clippy / fmt check / trybuild / cargo-llvm-cov / pytest matrix
├── CLAUDE.md                            # agent context — plan reference updated by this command
└── specs/001-pyenum-derive/             # this feature's spec-kit artifacts (above)
```

**Structure Decision**: Cargo workspace with two crates (`pyenum`, `pyenum-derive`). The proc-macro must live in its own crate because `proc-macro = true` crates cannot expose non-macro items. A third "core" crate is *not* introduced in v1 — the only truly shared surface (the reserved-name set) is small enough to duplicate or share via a tiny inline `include!` macro if needed without justifying a separate crate. The Python extension used by the integration test suite lives in `tests/python/` and is built on-demand by `maturin develop` inside a pytest fixture; it is not a published crate.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified.**

No violations. Table intentionally empty.
