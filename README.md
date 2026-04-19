# pyenum

**Expose Rust enums to Python as real `enum.Enum` subclasses — via PyO3.**

`pyenum` provides a `#[derive(PyEnum)]` macro that turns a Rust `enum` into a
genuine Python enum class. The resulting type passes `isinstance(x, enum.Enum)`,
iterates in declaration order, supports aliasing, and interoperates with the
tools that actually check enum membership — pydantic, FastAPI, SQLAlchemy,
`match`/`case`, dataclasses — with zero hand-written conversion code.

> Status: **draft / pre-release.** Spec `001-pyenum-derive` is frozen; the
> implementation crate is under active development. API surface may still
> shift before the first tagged release.

---

## Why

PyO3's `#[pyclass]` gives you a Python class, but not a Python `enum.Enum`.
Downstream libraries that branch on `isinstance(x, enum.Enum)` — pydantic
field validation, FastAPI request parsing, SQLAlchemy `Enum` columns — reject
the result. The common workaround is hand-written `FromPyObject` /
`IntoPyObject` shims plus a mirror class on the Python side.

`pyenum` eliminates that boilerplate:

- The derive generates the PyO3 conversion traits automatically.
- The Python class is constructed once per interpreter via a cached
  `GILOnceCell`, so the boundary cost is negligible after the first call.
- Ill-formed Rust input (field-carrying variants, generics, base/value
  mismatches) is rejected at compile time with a variant-level diagnostic.

---

## Features

- Full `enum.Enum` protocol: iteration order, name/value lookup, aliasing,
  hashing, equality.
- Supports all five standard Python enum bases — `Enum` (default), `IntEnum`,
  `StrEnum`, `Flag`, `IntFlag` — selectable via a derive attribute argument.
- Automatic bidirectional conversion for `#[pyfunction]`, `#[pymethods]`, and
  `#[pyclass]` field signatures.
- Per-interpreter class cache: constructed once, shared by identity across all
  call sites.
- Compile-time validation via `trybuild`-style negative tests.

---

## Quick start

Add the crate to your PyO3 extension:

```toml
# Cargo.toml
[dependencies]
pyo3 = { version = "0.28", features = ["extension-module"] }
pyenum = { version = "0.1", features = ["pyo3-0_28"] }
```

Declare a Rust enum and derive `PyEnum`:

```rust
use pyenum::PyEnum;
use pyo3::prelude::*;

#[derive(Clone, Copy, PyEnum)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[pyfunction]
fn invert(c: Color) -> Color {
    match c {
        Color::Red => Color::Green,
        Color::Green => Color::Blue,
        Color::Blue => Color::Red,
    }
}

#[pymodule]
fn my_ext(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Color>()?;
    m.add_function(wrap_pyfunction!(invert, m)?)?;
    Ok(())
}
```

On the Python side the exposed class behaves exactly like a native `enum.Enum`:

```python
from enum import Enum
from my_ext import Color, invert

assert issubclass(Color, Enum)
assert list(Color) == [Color.RED, Color.GREEN, Color.BLUE]
assert Color["RED"] is Color.RED
assert invert(Color.RED) is Color.GREEN
```

### Targeting a different base

Every standard Python enum base is one attribute argument away:

```rust
#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntEnum")]
pub enum Status {
    Ok = 200,
    NotFound = 404,
    ServerError = 500,
}

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Role {
    #[pyenum(value = "admin")]
    Admin,
    #[pyenum(value = "user")]
    User,
}

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntFlag")]
pub enum Permission {
    Read  = 0b001,
    Write = 0b010,
    Exec  = 0b100,
}
```

```python
assert Status.OK == 200 and isinstance(Status.OK, int)
assert Role.ADMIN + "/panel" == "admin/panel"
assert Permission.READ | Permission.WRITE in Permission
```

### `StrEnum` values

`StrEnum` variants without an explicit `#[pyenum(value = "...")]` defer to
Python's `enum.auto()`, which — per the `StrEnum` contract introduced in
Python 3.11 — **lowercases the variant name**:

```rust
#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Greeting {
    Hello,                     // Greeting.HELLO.value == "hello"
    #[pyenum(value = "Bye")]
    Bye,                       // Greeting.BYE.value   == "Bye"
}
```

Attach an explicit `value` whenever you need to preserve case or pick a
label that differs from the variant identifier.

---

## Compatibility

| Surface       | Supported                                    |
| ------------- | -------------------------------------------- |
| **PyO3**      | 0.25, 0.26, 0.27, 0.28 (cargo feature gated) |
| **Python**    | 3.10, 3.11, 3.12, 3.13, 3.14                 |
| **Rust**      | stable (edition 2024)                        |
| **Platforms** | Linux (x86_64 / aarch64), macOS (x86_64 / arm64), Windows (x64) |

Select one PyO3 feature per build:

```toml
pyenum = { version = "0.1", default-features = false, features = ["pyo3-0_28"] }
```

> Note: `enum.StrEnum` was added in Python 3.11. On 3.10 it is emulated by
> mixing `str` into `enum.Enum` — semantics are preserved but the runtime
> base is slightly different. See the feature spec for details.

---

## Compile-time rejections

The derive will not let an invalid Rust enum reach the Python boundary. Each of
these fails the build with a variant-level diagnostic:

- Tuple-struct or struct variants (`Variant(u8)`, `Variant { x: u8 }`)
- Generics or lifetime parameters
- Zero-variant enums
- Base/value mismatches: integer discriminants on `StrEnum`, string
  `#[pyenum(value = "...")]` on `IntEnum` / `Flag` / `IntFlag`
- Both a Rust discriminant **and** `#[pyenum(value = "...")]` on the
  same variant
- Name collisions with Python dunder names or `enum`-reserved members
- Duplicate Python values across variants — including `StrEnum` auto
  collisions where two Rust variant names lowercase to the same string.
  The library refuses to create Python-side aliases because they would
  break Rust-side round-trip identity

Every case is covered by a `trybuild` snapshot test.

---

## Repository layout

```
pyenum/
├── crates/
│   ├── pyenum/           # runtime crate (cache, conversion helpers, re-exports)
│   └── pyenum-derive/    # proc-macro crate (#[derive(PyEnum)])
├── python/               # maturin-built test extension + pytest suite
├── specs/001-pyenum-derive/
│   ├── spec.md           # feature specification (source of truth)
│   ├── plan.md           # implementation plan
│   └── checklists/       # requirements checklist
├── .github/workflows/    # lint, test, publish
└── CLAUDE.md             # contributor / agent operating notes
```

---

## Development

Prerequisites: Rust stable (edition 2024), Python 3.10+, [`uv`](https://github.com/astral-sh/uv),
[`maturin`](https://github.com/PyO3/maturin).

```bash
# Rust checks
cargo fmt --all
cargo clippy --workspace --all-targets --features pyo3-0_28 -- -D warnings
cargo test  --workspace --features pyo3-0_28
cargo test  -p pyenum-derive --test trybuild

# Python integration (builds the extension into a venv)
cd python
uv venv --python 3.13
uv pip install maturin pytest pydantic fastapi sqlalchemy
uv run maturin develop --release --features pyo3-0_28
uv run pytest -q
```

CI runs the full matrix on every PR: `cargo test` across every supported
OS × Python × PyO3 combination, the `trybuild` suite, the Python integration
tests, and a coverage report via `cargo-llvm-cov`.

---

## Spec Kit workflow

This repo uses [Spec Kit](https://github.com/anymindgroup/spec-kit). The active
feature lives under `specs/001-pyenum-derive/`; treat `spec.md` as the source
of truth. When requirements change, update the spec **before** touching
implementation.

Standard flow:

```text
/speckit-specify  →  /speckit-clarify  →  /speckit-plan
                →  /speckit-tasks     →  /speckit-analyze
                →  /speckit-implement
```

---

## Performance budget

Targets from `spec.md` §SC-004, validated by the benchmark suite:

- First construction of a Python enum class:
  - &lt; 2 ms for enums up to 32 variants
  - &lt; 20 ms for enums up to 1,024 variants
- Steady-state conversion (cache hit): &lt; 1 µs per call
- Scaling: linear in variant count, no worse.

---

## License

MIT
