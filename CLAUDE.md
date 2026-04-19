<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
specs/001-pyenum-derive/plan.md
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

- **Target PyO3 versions**: 0.25, 0.26, 0.27, 0.28 (selectable via cargo
  features; 0.28 is the default). Versions <0.25 and >0.28 are out of scope.
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
- Generate PyO3 conversion traits automatically (per the active PyO3 feature —
  `IntoPyObject`/`FromPyObject` on 0.26+, `IntoPy`/`ToPyObject` fallbacks on
  0.25 where applicable) so the enum can appear directly in `#[pyfunction]`,
  `#[pymethods]`, and `#[pyclass]` field signatures without manual extraction
  or conversion code.
- Round-trip (Rust → Python → Rust) must preserve variant identity across every
  supported base.
- Construct the Python class **at most once per interpreter** via a per-
  interpreter cache (PyO3's `GILOnceCell` or the version's equivalent primitive
  across 0.25–0.28), safe under GIL-held concurrent access.
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
├── Cargo.toml                        # crate manifest (edition 2024)
├── src/                              # library + proc-macro implementation (TBD in plan)
├── specs/001-pyenum-derive/
│   ├── spec.md                       # feature specification (source of truth)
│   └── checklists/requirements.md    # requirements checklist
└── .specify/                         # Spec Kit workflow assets
```

Implementation layout (proc-macro crate split, runtime support crate,
integration test crate, trybuild negative tests) will be fixed by
`/speckit-plan` and captured in `specs/001-pyenum-derive/plan.md`.

## Toolchain

- **Rust**: stable channel, edition 2024 (per `Cargo.toml`).
- **Python build**: maturin (or equivalent) for PyO3 extension builds.
- **Formatting / lint**: `cargo fmt`, `cargo clippy`, `cargo check` run via
  PostToolUse hooks — rely on repo-local `cargo` invocations, not remote tools.
- **PyO3 version matrix**: cargo features `pyo3-0_25` / `pyo3-0_26` /
  `pyo3-0_27` / `pyo3-0_28` (exactly one active; `pyo3-0_28` is the default).
  The proc-macro and runtime crates branch on these features to emit the
  correct conversion trait impls and cache primitives for each line.

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
- **Integration tests** that build an extension module, import it from Python,
  and assert the full enum protocol per base type (Enum, IntEnum, StrEnum,
  Flag, IntFlag). CI runs the full integration suite against each supported
  PyO3 version (0.25 / 0.26 / 0.27 / 0.28) in the matrix.
- **Trybuild negative tests** for every compile-fail case enumerated in spec
  FR-004/FR-005 and US5 (non-unit variants, generics, lifetimes, empty enum,
  base/value mismatch, alias conflicts).
- **End-to-end interop tests** against at least pydantic, FastAPI,
  SQLAlchemy, and Python `match`/`case` — per SC-006.
- Coverage target follows the repo standard (80%+).

## Notes when working in this repo

- The spec is the source of truth. If implementation pressure conflicts with
  the spec, update the spec first, do not drift silently.
- PyO3's API surface changes frequently between point releases (conversion
  traits renamed/deprecated across 0.25→0.28, `Bound<'py, T>` ergonomics
  evolving, GIL primitives relocating). Always verify conversion trait names,
  GIL primitives, and module registration macros against the **exact** target
  version's docs — do not rely on examples from an adjacent release.
- Keep per-version divergence isolated behind a thin compatibility shim module
  so feature-gated `cfg` noise does not leak into the derive expansion or the
  enum-cache core logic.
- Python enum construction uses the functional `Enum("Name", [...])` API via
  PyO3; do not reach into CPython C-level enum internals.
- Preserve Python aliasing semantics (variants with equal values become
  aliases of the first-declared variant). Do not silently dedupe or reorder.
- Document any behavior that is "not supported in v1" (e.g., module reload,
  free-threaded build) rather than leaving it implicit.
