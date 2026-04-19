<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
specs/001-pyenum-derive/plan.md

For the project's non-negotiable principles and governance rules, read the
constitution:
.specify/memory/constitution.md
<!-- SPECKIT END -->

# pyenum

A Rust library that lets PyO3 authors expose Rust `enum` types to Python as
genuine `enum.Enum` subclasses — passing `isinstance(x, enum.Enum)`, iterating
in declaration order, supporting aliasing, and interoperating with downstream
tools (pydantic, FastAPI, SQLAlchemy, `match`/`case`, dataclasses) without
hand-written conversion shims.

**Active feature**: `specs/001-pyenum-derive/spec.md` (branch
`001-pyenum-derive`, status: Draft).

## Scope of v1

- **Target PyO3 version**: 0.28. Earlier / later PyO3 lines are out of scope
  (attempting to support multiple versions from one crate is blocked by the
  `pyo3-ffi` `links = "python"` native-library-uniqueness rule in cargo).
- **Target Python**: 3.11+ (so `enum.StrEnum` is available without polyfill).
- **Delivery surface**: a `#[derive(PyEnum)]`-style proc-macro attached to the
  user's Rust enum declaration.
- **Supported Python bases**: `Enum` (default), `IntEnum`, `StrEnum`, `Flag`,
  `IntFlag` — selectable via a derive attribute argument.
- **Accepted Rust input**: unit-variant enums only, optionally with explicit
  discriminants. Tuple, struct, generic, and lifetime-parameterized enums are
  rejected at compile time with a variant-level diagnostic.
- **Out of scope for v1**: projecting Rust `impl` methods onto the Python
  class; module-less standalone export; free-threaded (`--disable-gil`) Python
  guarantees.

## Core requirements (summary from spec)

- Expose Rust enum as a true subclass of the chosen Python base — full enum
  protocol (iteration, name/value lookup, aliasing, hashing, equality, base-
  specific ops such as bitwise for flag types).
- Generate PyO3 0.28's conversion traits automatically — `IntoPyObject<'py>`
  for both `T` and `&T`, plus `FromPyObject<'a, 'py>` — so the enum can
  appear directly in `#[pyfunction]`, `#[pymethods]`, and `#[pyclass]` field
  signatures without manual extraction or conversion code.
- Round-trip (Rust → Python → Rust) must preserve variant identity across every
  supported base.
- Construct the Python class **at most once per interpreter** via a per-type
  `pyo3::sync::PyOnceLock`, safe under GIL-held concurrent access.
- Reject non-conforming Rust enums at compile time with diagnostics that name
  the offending variant and the rule violated (trybuild-style negative tests).
- Raise standard Python exceptions (`TypeError`, `ValueError`) at the
  conversion boundary when Python callers pass invalid values.

## Performance budget (SC-004)

- First construction: < 2 ms for enums up to 32 variants, < 20 ms up to 1,024
  variants.
- Steady-state conversion (cache hit): < 1 µs per call.
- Scaling: linear in variant count, no worse.

## Repository layout

```
pyenum/
├── Cargo.toml                        # workspace manifest (edition 2024)
├── crates/
│   ├── pyenum/                       # runtime facade (published)
│   ├── pyenum-derive/                # proc-macro (published)
│   └── pyenum-test/                  # cdylib fixture (publish = false)
├── python/
│   └── pyproject.toml                # maturin + pytest config
├── tests/                            # Python pytest suite (no Cargo)
├── specs/001-pyenum-derive/
│   ├── spec.md                       # feature specification (source of truth)
│   ├── plan.md                       # implementation plan
│   ├── research.md                   # Phase 0 decisions
│   ├── tasks.md                      # Phase 2 task list
│   └── checklists/requirements.md    # requirements checklist
└── .specify/                         # Spec Kit workflow assets
```

## Toolchain

- **Rust**: stable channel, edition 2024 (per `Cargo.toml`).
- **Python build**: maturin for PyO3 extension builds.
- **Formatting / lint**: `cargo fmt`, `cargo clippy`, `cargo check`,
  `ruff format`, `ruff check`, `mypy` all run via `.pre-commit-config.yaml`.
- **PyO3 dep**: `pyo3 = { version = "0.28", features = ["abi3-py311"] }`
  single-version. The project deliberately does NOT expose a PyO3 version
  feature matrix because cargo's `pyo3-ffi` `links = "python"` rule disallows
  two `pyo3` versions coexisting as optional deps in the same graph.

## Spec Kit workflow

This repo uses Spec Kit. Stay within the documented flow:

1. `/speckit-specify` — create/update spec (done for 001).
2. `/speckit-clarify` — resolve underspecified areas before planning.
3. `/speckit-plan` — produce `plan.md`, architecture, data model, contracts.
4. `/speckit-tasks` — generate dependency-ordered `tasks.md`.
5. `/speckit-analyze` — cross-artifact consistency check.
6. `/speckit-implement` — execute tasks; keep TDD discipline (tests first,
   trybuild for compile-fail cases, `cargo test` for runtime behavior).

Do not short-circuit to implementation before plan + tasks exist for the
feature.

## Testing expectations

- **Unit tests** in the runtime crate for conversion helpers and cache logic.
- **Python integration tests** (`tests/`) that build the `pyenum-test` cdylib
  via `maturin develop` and assert the full enum protocol per base type
  (Enum, IntEnum, StrEnum, Flag, IntFlag), the conversion boundary, and
  cache identity stability.
- **Trybuild negative tests** for every compile-fail case (non-unit variants,
  generics, lifetimes, empty enum, base/value mismatch, reserved names).
- **End-to-end interop tests** against at least pydantic, FastAPI,
  SQLAlchemy, and Python `match`/`case`.
- Coverage target follows the repo standard (80%+).

## Notes when working in this repo

- The spec is the source of truth. If implementation pressure conflicts with
  the spec, update the spec first, do not drift silently.
- PyO3 0.28 is the pinned target. Note that 0.28 renamed `GILOnceCell` to
  `PyOnceLock`, and `FromPyObject` now takes two lifetimes — `FromPyObject<'a,
  'py>` with `fn extract(obj: Borrowed<'a, 'py, PyAny>)`. When consulting
  older pyo3 examples, always verify trait shapes against the 0.28 rustdoc.
- Python enum construction uses the functional `Enum("Name", [...])` API via
  PyO3; do not reach into CPython C-level enum internals.
- Preserve Python aliasing semantics (variants with equal values become
  aliases of the first-declared variant). Do not silently dedupe or reorder.
- Document any behavior that is "not supported in v1" (e.g., module reload,
  free-threaded build) rather than leaving it implicit.
