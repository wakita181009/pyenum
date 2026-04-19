<!--
Sync Impact Report
==================
Version change: (uninitialized template) → 1.0.0
Modified principles:
  - (none — initial ratification; all placeholder principles replaced with concrete ones)
Added sections:
  - Core Principles (5 principles: Spec-Driven Development, Test-First,
    Version Compatibility Isolation, Quality Gate, Performance Budget)
  - Scope Boundaries
  - Development Workflow
  - Governance
Removed sections:
  - (none — placeholder template sections fully replaced)
Templates reviewed:
  - ✅ .specify/templates/plan-template.md — Constitution Check section already
    references the constitution; principles below align with its TDD Plan,
    Version Matrix, and Performance gates.
  - ✅ .specify/templates/spec-template.md — No changes required; principles are
    execution-side, spec template stays behavior-focused.
  - ✅ .specify/templates/tasks-template.md — Task categories already cover
    testing discipline; Quality Gate principle adds post-implementation
    formatting/lint/type-check tasks which the tasks template can absorb
    without structural change.
  - ✅ .specify/templates/checklist-template.md — No structural change required.
Follow-up TODOs:
  - .specify/specs/001-pyenum-derive/plan.md: re-run its "Constitution Check" section
    against these principles and fill/adjust gate rows (pending owner action).
-->

# pyenum Constitution

## Core Principles

### I. Spec-Driven Development (NON-NEGOTIABLE)

The Spec Kit flow (`/speckit-specify` → `/speckit-clarify` → `/speckit-plan` →
`/speckit-tasks` → `/speckit-analyze` → `/speckit-implement`) is the only
supported path from intent to code. Implementation MUST NOT begin before a
feature's `spec.md`, `plan.md`, and `tasks.md` exist and have passed analysis.

The spec is the source of truth. If implementation pressure conflicts with the
spec, the spec MUST be updated first — silent drift is prohibited. Behavior
that is "not supported in v1" MUST be written down explicitly rather than left
implicit.

**Rationale**: pyenum's value depends on predictable semantics across Python,
Rust, and multiple PyO3 versions. Spec-first discipline is the only way to
keep cross-version guarantees coherent while the API surface solidifies.

### II. Test-First (NON-NEGOTIABLE)

Every user story and every non-trivial behavior described in the spec MUST be
driven by a failing test before implementation. Red → Green → Refactor is
mandatory.

Required test types for this project:

- **Unit tests** in the runtime crate for conversion helpers and cache logic.
- **Integration tests** that build an extension module, import it from Python,
  and assert the full enum protocol per base type (`Enum`, `IntEnum`,
  `StrEnum`, `Flag`, `IntFlag`). CI MUST run the full integration suite
  against each supported PyO3 version (0.25 / 0.26 / 0.27 / 0.28).
- **Trybuild negative tests** for every compile-fail case enumerated in the
  spec (non-unit variants, generics, lifetimes, empty enums, base/value
  mismatches, alias conflicts).
- **End-to-end interop tests** against at least pydantic, FastAPI,
  SQLAlchemy, and Python `match`/`case`.

Coverage target: 80%+ across the runtime and macro crates.

**Rationale**: proc-macros and FFI shims fail in ways that are invisible to
humans (wrong trait resolution, silent conversion bugs across PyO3 versions).
Tests first is the only way to catch divergence early.

### III. Version Compatibility Isolation

Per-PyO3-version divergence MUST be isolated behind a thin compatibility shim
module. Feature-gated `cfg` noise (`pyo3-0_25`..`pyo3-0_28`) MUST NOT leak into
the derive expansion, the enum-cache core, or public API signatures. The
compat layer is the only place allowed to branch on PyO3 version.

Exactly one of `pyo3-0_25`, `pyo3-0_26`, `pyo3-0_27`, `pyo3-0_28` MUST be
active at any time. `pyo3-0_28` is the default. Versions outside this range
are explicitly out of scope.

When adopting a new PyO3 conversion trait, GIL primitive, or module
registration macro, the exact target version's documentation MUST be verified
— examples from adjacent releases MUST NOT be relied on.

**Rationale**: PyO3's API surface changes between point releases (conversion
traits renamed/deprecated across 0.25→0.28, `Bound<'py, T>` ergonomics
evolving, GIL primitives relocating). Compat-shim discipline keeps the core
readable and makes future version additions a localized change.

### IV. Quality Gate (NON-NEGOTIABLE)

Implementation is considered complete only when every check below is green on
the changed surface. These checks are non-negotiable gates for merge and for
declaring a task "done".

**Rust checks** (run at the workspace root):

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

**Python checks** (run via `uv` / `uvx` so tool versions stay pinned and
reproducible — no reliance on globally installed `ruff` / `mypy`):

- `uvx ruff check`
- `uvx ruff format --check`
- `uvx mypy`
- `uv run pytest` (uses the project environment so the built extension is
  importable)

`trybuild` negative tests MUST be part of `cargo test` and therefore part of
this gate.

Lowering or skipping a check requires an explicit written justification in the
relevant spec or plan, and a time-boxed follow-up to restore it. Any `|| true`
or `--no-verify` equivalent used to bypass these checks is a constitution
violation.

**Rationale**: macro output is hard to eyeball; lints and type-checkers catch
things human review misses. Pinning the toolchain via `uv`/`uvx` removes
"works on my machine" variance across contributors and CI.

### V. Performance Budget

The performance targets from the spec (SC-004) are binding:

- **First construction**: < 2 ms for enums up to 32 variants; < 20 ms up to
  1,024 variants.
- **Steady-state conversion (cache hit)**: < 1 µs per call.
- **Scaling**: linear in variant count; no worse.

Any change that moves a benchmark past its budget MUST either be rolled back
or accompanied by a spec amendment that updates the budget with justification.
Benchmarks MUST exist for at least the 32-variant and 1,024-variant cases and
run in CI.

**Rationale**: pyenum is invoked on every conversion at the FFI boundary — a
regression in steady-state cost silently taxes every downstream user.
Budgets enforced in CI prevent silent drift.

## Scope Boundaries

The following are explicitly **in scope** for v1:

- `#[derive(PyEnum)]` on unit-variant Rust enums, with optional explicit
  discriminants.
- Python base selection among `Enum`, `IntEnum`, `StrEnum`, `Flag`, `IntFlag`
  via a derive attribute.
- Automatic PyO3 conversion trait generation per the active PyO3 version.
- Per-interpreter class caching via `GILOnceCell` or the equivalent primitive
  of the active PyO3 version, safe under GIL-held concurrent access.
- Python 3.11+ as the minimum supported runtime (so `enum.StrEnum` is
  available without polyfill).

The following are explicitly **out of scope** for v1 and MUST be rejected at
compile time or documented as unsupported:

- Tuple, struct, generic, or lifetime-parameterized Rust enums.
- Projecting Rust `impl` methods onto the generated Python class.
- Module-less standalone export (derive requires a `#[pymodule]` host).
- Free-threaded (`--disable-gil`) Python interpreter guarantees.
- PyO3 versions < 0.25 or > 0.28.

## Development Workflow

- **Toolchain**: Rust stable, edition 2024; Python 3.11+; `maturin` for PyO3
  extension builds; `uv` for Python tool and dependency management.
- **Version matrix**: CI MUST run the full test suite against each supported
  PyO3 version (0.25 / 0.26 / 0.27 / 0.28). A failure on any single version
  blocks merge.
- **Pre-commit feedback loop**: PostToolUse hooks (`.claude/settings.json`)
  run the relevant subset of Principle IV checks on the edited file after
  every `Write` / `Edit`. These are fast-feedback hooks, not a replacement
  for the full gate, which still runs in CI.
- **Python enum construction**: MUST use the functional
  `Enum("Name", [...])` API via PyO3. Reaching into CPython C-level enum
  internals is prohibited.
- **Aliasing semantics**: variants with equal values become aliases of the
  first-declared variant. Silent dedupe or reorder is prohibited.

## Governance

This constitution supersedes ad-hoc conventions. Where guidance here conflicts
with CLAUDE.md, rule files under `~/.claude/rules/`, or any individual
contributor preference, the constitution wins.

**Amendment procedure**:

1. Propose the change in a spec or dedicated amendment doc. State which
   principle is added, modified, or removed, and why.
2. Update `.specify/memory/constitution.md` via `/speckit-constitution`,
   bumping the version per the policy below.
3. Propagate the change across dependent artifacts (plan template, tasks
   template, command files, CLAUDE.md, CI config) in the same change set.

**Versioning policy** (semantic):

- **MAJOR**: a principle is removed, redefined, or made backward-incompatible
  (e.g., a gate is relaxed or a scope boundary is expanded across a line that
  existing code relies on).
- **MINOR**: a new principle or section is added, or an existing one is
  materially expanded.
- **PATCH**: clarifications, wording, typo fixes, non-semantic refinements.

**Compliance review**:

- Every PR description MUST assert compliance with the Core Principles, or
  document the explicit exception and its follow-up.
- Plans generated via `/speckit-plan` MUST fill in the "Constitution Check"
  section against the principles above before Phase 0 research proceeds.
- Performance budgets, version matrix results, and quality-gate status are
  reviewed at each release cut.

**Runtime guidance**: day-to-day implementation context (stack, commands,
version-specific quirks) lives in `CLAUDE.md` and the active feature's
`.specify/specs/<feature>/plan.md`. This constitution intentionally stays
behavior-focused and refers out for mechanics.

**Version**: 1.0.0 | **Ratified**: 2026-04-20 | **Last Amended**: 2026-04-20

