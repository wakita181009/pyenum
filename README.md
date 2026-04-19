# pyenum

[![Crates.io](https://img.shields.io/crates/v/pyenum.svg)](https://crates.io/crates/pyenum)
[![CI](https://github.com/wakita181009/pyenum/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/wakita181009/pyenum/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/wakita181009/pyenum/branch/main/graph/badge.svg)](https://codecov.io/gh/wakita181009/pyenum)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**Expose Rust enums to Python as real `enum.Enum` subclasses — via PyO3.**

`pyenum` provides a `#[derive(PyEnum)]` macro that turns a Rust `enum` into a
genuine Python enum class. The resulting type passes `isinstance(x, enum.Enum)`,
iterates in declaration order, and interoperates with the tools that actually
check enum membership — pydantic, FastAPI, SQLAlchemy, `match`/`case`,
dataclasses — with zero hand-written conversion code.

---

## Why

PyO3's `#[pyclass]` gives you a Python class, but not a Python `enum.Enum`.
Downstream libraries that branch on `isinstance(x, enum.Enum)` — pydantic
field validation, FastAPI request parsing, SQLAlchemy `Enum` columns — reject
the result. The common workaround is hand-written `FromPyObject` /
`IntoPyObject` shims plus a mirror class on the Python side.

`pyenum` eliminates that boilerplate:

- The derive generates the PyO3 conversion traits automatically
  (`IntoPyObject<'py>` for `T` and `&T`, plus `FromPyObject<'a, 'py>`).
- The Python class is constructed once per interpreter via a cached
  `pyo3::sync::PyOnceLock`, so the boundary cost is negligible after the
  first call.
- Ill-formed Rust input (field-carrying variants, generics, base/value
  mismatches) is rejected at compile time with a variant-level diagnostic.

---

## Features

- Full `enum.Enum` protocol: iteration order, name/value lookup, hashing,
  equality, and base-specific operations (bitwise ops on `Flag` / `IntFlag`,
  `int` / `str` mixins for `IntEnum` / `StrEnum`).
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
pyo3   = { version = "0.28", features = ["extension-module", "abi3-py311"] }
pyenum = "0.0.1"
```

`pyenum` pins PyO3 to **0.28** — see [Compatibility](#compatibility) for the
rationale.

Declare a Rust enum and derive `PyEnum`:

```rust
use pyenum::{PyEnum, PyModuleExt};
use pyo3::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
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
    // `add_enum::<T>()` comes from `PyModuleExt`. It registers `T` as a real
    // Python `enum.Enum` subclass under `T`'s Rust identifier.
    m.add_enum::<Color>()?;
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

| Surface       | Supported                                                       |
| ------------- | --------------------------------------------------------------- |
| **PyO3**      | 0.28 only                                                       |
| **Python**    | 3.11, 3.12, 3.13 (CPython; `abi3-py311` limited API)            |
| **Rust**      | stable, edition 2024, MSRV 1.94                                 |
| **Platforms** | Linux (x86_64 / aarch64), macOS (x86_64 / arm64), Windows (x64) |

### Why PyO3 0.28 only

Cargo's `pyo3-ffi` `links = "python"` rule forbids two `pyo3` versions
coexisting in the same dependency graph, so a `pyo3-0_2X` feature matrix
cannot actually be built. `pyenum` therefore tracks a single PyO3 minor
line and will bump in lockstep with upstream.

### Why Python 3.11+

`enum.StrEnum` landed in Python 3.11. Polyfilling it on 3.10 means mixing
`str` into `enum.Enum`, which changes the runtime base class and breaks
the "pass `isinstance(x, StrEnum)`" guarantee. We chose the strict floor.

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

## Development

Prerequisites: Rust stable (edition 2024, MSRV 1.94), Python 3.11+,
[`uv`](https://github.com/astral-sh/uv), [`maturin`](https://github.com/PyO3/maturin).

```bash
# Rust checks
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace
cargo test  -p pyenum-derive --test trybuild

# Python integration — conftest.py rebuilds the pyenum-test cdylib
# on every pytest run via `maturin develop`, so no manual build step.
cd python
uv venv --python 3.11
uv pip install -e ".[test]" maturin
uv run pytest -q
```

CI runs, on every PR: `cargo fmt`/`clippy`/`test`, the `trybuild` suite,
and the Python integration tests against the supported Python versions
on Linux / macOS / Windows.

---

## License

MIT
