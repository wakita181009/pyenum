# Quickstart: pyenum (001)

**Goal**: Expose a Rust enum to Python as a real `enum.Enum` subclass in under 15 minutes (SC-007).

This quickstart assumes a working Rust toolchain (stable, edition 2024) and a Python 3.11+ environment with `maturin` installed.

## 1. Scaffold a PyO3 extension

```bash
cargo new --lib my_ext
cd my_ext
```

Edit `Cargo.toml`:

```toml
[package]
name = "my_ext"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.28", features = ["abi3-py311"] }
pyenum = "0.1"
```

`pyenum` v1 pins PyO3 0.28. Using a different PyO3 line with `pyenum` is unsupported and will fail to build — cargo's `pyo3-ffi` `links = "python"` rule prevents mixing PyO3 versions in a single graph.

## 2. Declare a Rust enum and derive `PyEnum`

`src/lib.rs`:

```rust
use pyenum::prelude::*;      // PyEnum derive + PyModuleExt
use pyo3::prelude::*;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntEnum")]
enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    Teapot = 418,
}

#[pyfunction]
fn describe(status: HttpStatus) -> &'static str {
    match status {
        HttpStatus::Ok => "all good",
        HttpStatus::NotFound => "missing",
        HttpStatus::Teapot => "short and stout",
    }
}

#[pymodule]
fn my_ext<'py>(m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_enum::<HttpStatus>()?;
    m.add_function(wrap_pyfunction!(describe, m)?)?;
    Ok(())
}
```

That is the complete Rust side. The `#[derive(PyEnum)]` emits the conversion plumbing; `m.add_enum::<HttpStatus>()?` registers the Python class; `describe` accepts the derived enum directly.

## 3. Build and import

```bash
maturin develop
```

`maturin` drops a Python extension module into the active interpreter's site-packages.

## 4. Use it from Python

```python
>>> import my_ext, enum
>>> issubclass(my_ext.HttpStatus, enum.IntEnum)
True
>>> list(my_ext.HttpStatus)
[<HttpStatus.Ok: 200>, <HttpStatus.NotFound: 404>, <HttpStatus.Teapot: 418>]
>>> my_ext.HttpStatus.Ok
<HttpStatus.Ok: 200>
>>> my_ext.HttpStatus["NotFound"]
<HttpStatus.NotFound: 404>
>>> my_ext.HttpStatus(418)
<HttpStatus.Teapot: 418>
>>> my_ext.describe(my_ext.HttpStatus.Ok)
'all good'
>>> my_ext.HttpStatus.Ok == 200
True
```

All standard enum protocol operations work. `pydantic`, `FastAPI` request parsing, and `match`/`case` accept the class transparently:

```python
>>> match my_ext.HttpStatus.Teapot:
...     case my_ext.HttpStatus.Ok: "ok"
...     case my_ext.HttpStatus.Teapot: "teapot"
'teapot'
```

## 5. Try the other bases

Swap `base = "IntEnum"` for any of `"Enum"`, `"StrEnum"`, `"Flag"`, `"IntFlag"`. For `IntFlag`:

```rust
#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntFlag")]
enum Permission {
    Read = 1,
    Write = 2,
    Admin = 4,
}
```

Python:

```python
>>> (my_ext.Permission.Read | my_ext.Permission.Write) & my_ext.Permission.Read
<Permission.Read: 1>
```

## 6. Variants without explicit values

Leave the discriminant off and `pyenum` delegates to Python's own `enum.auto()`:

```rust
#[derive(Clone, Copy, PyEnum)]
enum Color { Red, Green, Blue }
```

Python:

```python
>>> [m.value for m in my_ext.Color]
[1, 2, 3]
```

For `StrEnum` the auto-value is the variant name, exactly as CPython's `StrEnum.auto()` does.

## 7. What if a Rust enum cannot be represented as a Python enum?

The derive rejects ill-formed inputs at compile time with a diagnostic pointing at the offending variant:

```
error: variant `Pass` collides with a Python keyword
   --> src/lib.rs:12:5
    |
 12 |     Pass,
    |     ^^^^
```

Other rejection classes (tuple variants, struct variants, generics, lifetimes, empty enum, base/value mismatch, enum-reserved member names, dunder collisions) all produce analogous diagnostics. See `.specify/specs/001-pyenum-derive/contracts/derive-contract.md` for the full list.

## Next steps

- See `.specify/specs/001-pyenum-derive/spec.md` for the full acceptance surface.
- See `.specify/specs/001-pyenum-derive/contracts/` for per-surface guarantees.
- For interop with pydantic, FastAPI, SQLAlchemy, see the tests under `tests/python/test_interop_*.py` once `/speckit-tasks` and implementation land.
