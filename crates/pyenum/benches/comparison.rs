//! pyenum vs pyo3 vanilla comparison benchmarks.
//!
//! Compares:
//! - pyenum: #[derive(PyEnum)] with full enum.Enum protocol
//! - pyo3 vanilla: #[pyclass] + #[pymethods] manual implementation
//! - Alternative caching strategies
//!
//! IMPORTANT: All benchmarks except `first_construct_*` measure **steady-state
//! runtime performance** — i.e., after the Python enum class has been built and
//! cached. This represents the critical path for high-frequency conversions in
//! production applications.

use criterion::{Criterion, criterion_group, criterion_main};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::hint::black_box;

// ============================================================================
// pyenum types (using derive with full caching)
// ============================================================================
use pyenum::{PyEnum, PyEnumTrait};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PyEnum)]
#[pyenum(base = "IntEnum")]
enum PyenumColor {
    Red = 1,
    Green = 2,
    Blue = 3,
}

// ============================================================================
// pyo3 vanilla types (#[pyclass] — not a real Python enum.Enum)
// ============================================================================

#[pyclass(skip_from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VanillaColor {
    Red = 1,
    Green = 2,
    Blue = 3,
}

#[pymethods]
impl VanillaColor {
    #[new]
    fn new(value: i32) -> PyResult<Self> {
        match value {
            1 => Ok(VanillaColor::Red),
            2 => Ok(VanillaColor::Green),
            3 => Ok(VanillaColor::Blue),
            _ => Err(PyValueError::new_err("invalid value")),
        }
    }

    #[getter]
    fn value(&self) -> i32 {
        *self as i32
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            VanillaColor::Red => "Red",
            VanillaColor::Green => "Green",
            VanillaColor::Blue => "Blue",
        }
    }

    fn __repr__(&self) -> String {
        format!("VanillaColor.{}", self.name())
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

// ============================================================================
// Benchmark: Initial class construction (ONE-TIME cost)
// ============================================================================

fn bench_first_construct_pyenum(c: &mut Criterion) {
    Python::initialize();
    c.bench_function("first_construct_pyenum", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let cls = PyenumColor::py_enum_class(py).unwrap();
                black_box(cls);
            });
        });
    });
}

fn bench_first_construct_vanilla(c: &mut Criterion) {
    Python::initialize();
    c.bench_function("first_construct_vanilla", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let cls = py.get_type::<VanillaColor>();
                black_box(cls);
            });
        });
    });
}

// ============================================================================
// Benchmark: Steady-state Rust -> Python conversion (RUNTIME critical path)
// ============================================================================

fn bench_to_python_pyenum(c: &mut Criterion) {
    Python::initialize();
    // Prime cache: class + members cached
    Python::attach(|py| {
        let _ = PyenumColor::py_enum_class(py).unwrap();
        let _ = PyenumColor::Red.to_py_member(py).unwrap();
    });

    c.bench_function("to_python_pyenum", |b| {
        b.iter(|| {
            Python::attach(|py| {
                // Runtime: O(1) cache lookup (no Python getattr)
                let member = PyenumColor::Green.to_py_member(py).unwrap();
                black_box(member);
            });
        });
    });
}

fn bench_to_python_vanilla(c: &mut Criterion) {
    Python::initialize();

    c.bench_function("to_python_vanilla", |b| {
        b.iter(|| {
            Python::attach(|py| {
                // Runtime: creates new pyclass instance
                let obj: Bound<'_, VanillaColor> = VanillaColor::Green.into_pyobject(py).unwrap();
                black_box(obj);
            });
        });
    });
}

// ============================================================================
// Benchmark: Steady-state Python -> Rust conversion (RUNTIME critical path)
// ============================================================================

fn bench_from_python_pyenum(c: &mut Criterion) {
    Python::initialize();
    // Create persistent Python object
    let owned = Python::attach(|py| PyenumColor::Blue.to_py_member(py).unwrap().unbind());

    c.bench_function("from_python_pyenum", |b| {
        b.iter(|| {
            Python::attach(|py| {
                // Runtime: O(1) pointer equality check
                let bound = owned.bind(py);
                let variant = PyenumColor::from_py_member(bound).unwrap();
                black_box(variant);
            });
        });
    });
}

fn bench_from_python_vanilla(c: &mut Criterion) {
    Python::initialize();
    let owned = Python::attach(|py| VanillaColor::Blue.into_pyobject(py).unwrap().unbind());

    c.bench_function("from_python_vanilla", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let bound = owned.bind(py);
                let borrowed: PyRef<'_, VanillaColor> = bound.borrow();
                let variant = *borrowed;
                black_box(variant);
            });
        });
    });
}

// ============================================================================
// Benchmark: Steady-state round-trip (RUNTIME critical path)
// ============================================================================

fn bench_roundtrip_pyenum(c: &mut Criterion) {
    Python::initialize();
    Python::attach(|py| {
        let _ = PyenumColor::py_enum_class(py).unwrap();
        let _ = PyenumColor::Red.to_py_member(py).unwrap();
    });

    c.bench_function("roundtrip_pyenum", |b| {
        b.iter(|| {
            Python::attach(|py| {
                // Full cache hit both directions
                let py_obj = PyenumColor::Green.to_py_member(py).unwrap();
                let back: PyenumColor = PyenumColor::from_py_member(&py_obj).unwrap();
                black_box(back);
            });
        });
    });
}

fn bench_roundtrip_vanilla(c: &mut Criterion) {
    Python::initialize();

    c.bench_function("roundtrip_vanilla", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let py_obj: Bound<'_, VanillaColor> =
                    VanillaColor::Green.into_pyobject(py).unwrap();
                let borrowed: PyRef<'_, VanillaColor> = py_obj.borrow();
                let back: VanillaColor = *borrowed;
                black_box(back);
            });
        });
    });
}

// ============================================================================
// Benchmark: Member access patterns
// ============================================================================

fn bench_getattr_pyenum(c: &mut Criterion) {
    Python::initialize();
    Python::attach(|py| {
        let _ = PyenumColor::py_enum_class(py).unwrap();
    });

    c.bench_function("getattr_pyenum", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let cls = PyenumColor::py_enum_class(py).unwrap();
                let member = cls.getattr("Green").unwrap();
                black_box(member);
            });
        });
    });
}

fn bench_getattr_vanilla(c: &mut Criterion) {
    Python::initialize();

    c.bench_function("getattr_vanilla", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let cls = py.get_type::<VanillaColor>();
                let instance = cls.call1((2_i32,)).unwrap();
                black_box(instance);
            });
        });
    });
}

// ============================================================================
// Benchmark: Equality comparison
// ============================================================================

fn bench_equality_pyenum(c: &mut Criterion) {
    Python::initialize();
    let green = Python::attach(|py| {
        let _ = PyenumColor::py_enum_class(py).unwrap();
        PyenumColor::Green.to_py_member(py).unwrap().unbind()
    });
    let blue = Python::attach(|py| PyenumColor::Blue.to_py_member(py).unwrap().unbind());

    c.bench_function("equality_pyenum", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let green_bound = green.bind(py);
                let blue_bound = blue.bind(py);
                let eq = green_bound
                    .as_any()
                    .call_method1("__eq__", (blue_bound.as_any(),))
                    .unwrap();
                black_box(eq);
            });
        });
    });
}

fn bench_equality_vanilla(c: &mut Criterion) {
    Python::initialize();
    let green = Python::attach(|py| VanillaColor::Green.into_pyobject(py).unwrap().unbind());
    let blue = Python::attach(|py| VanillaColor::Blue.into_pyobject(py).unwrap().unbind());

    c.bench_function("equality_vanilla", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let green_bound = green.bind(py);
                let blue_bound = blue.bind(py);
                let eq = green_bound
                    .as_any()
                    .call_method1("__eq__", (blue_bound.as_any(),))
                    .unwrap();
                black_box(eq);
            });
        });
    });
}

// ============================================================================
// ALTERNATIVE STRATEGIES (for comparison)
// ============================================================================

/// Strategy: Class-only cache
/// Python enum class cached, members fetched via getattr each call
struct ClassOnlyCached {
    py_enum_class: Py<PyType>,
}

impl ClassOnlyCached {
    fn new(py: Python<'_>) -> PyResult<Self> {
        let enum_module = py.import("enum")?;
        let int_enum = enum_module.getattr("IntEnum")?;

        let list = pyo3::types::PyList::new(
            py,
            [
                ("Red", 1i32).into_pyobject(py)?,
                ("Green", 2i32).into_pyobject(py)?,
                ("Blue", 3i32).into_pyobject(py)?,
            ],
        )?;

        let enum_class = int_enum.call1(("ClassCachedColor", list))?;
        let class: Py<PyType> = enum_class.cast::<PyType>()?.clone().unbind();

        Ok(Self {
            py_enum_class: class,
        })
    }

    fn get_member<'py>(&self, py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyAny>> {
        self.py_enum_class.bind(py).getattr(name)
    }
}

fn bench_to_python_class_only_cached(c: &mut Criterion) {
    Python::initialize();
    let cached = Python::attach(|py| ClassOnlyCached::new(py).unwrap());

    c.bench_function("to_python_class_only_cached", |b| {
        b.iter(|| {
            Python::attach(|py| {
                // Runtime: getattr call required (NOT cached)
                let member = cached.get_member(py, "Green").unwrap();
                black_box(member);
            });
        });
    });
}

fn bench_roundtrip_class_only_cached(c: &mut Criterion) {
    Python::initialize();
    let cached = Python::attach(|py| ClassOnlyCached::new(py).unwrap());

    c.bench_function("roundtrip_class_only_cached", |b| {
        b.iter(|| {
            Python::attach(|py| {
                // Runtime: getattr + value extraction needed
                let member = cached.get_member(py, "Green").unwrap();
                let value: i32 = member.getattr("value").unwrap().extract().unwrap();
                let variant = match value {
                    1 => VanillaColor::Red,
                    2 => VanillaColor::Green,
                    3 => VanillaColor::Blue,
                    _ => unreachable!(),
                };
                black_box(variant);
            });
        });
    });
}

/// Strategy: No cache — build Python enum every time
fn build_enum_no_cache(py: Python<'_>, variant_index: usize) -> PyResult<Bound<'_, PyAny>> {
    let enum_module = py.import("enum")?;
    let int_enum = enum_module.getattr("IntEnum")?;
    let variant_name = ["Red", "Green", "Blue"][variant_index];

    let list = pyo3::types::PyList::new(
        py,
        [
            ("Red", 1i32).into_pyobject(py)?,
            ("Green", 2i32).into_pyobject(py)?,
            ("Blue", 3i32).into_pyobject(py)?,
        ],
    )?;

    let enum_class = int_enum.call1(("NoCacheColor", list))?;
    enum_class.getattr(variant_name)
}

fn bench_to_python_no_cache(c: &mut Criterion) {
    Python::initialize();

    c.bench_function("to_python_no_cache", |b| {
        b.iter(|| {
            Python::attach(|py| {
                // Runtime: FULL rebuild every call (constructor + functional API)
                let member = build_enum_no_cache(py, 1).unwrap();
                black_box(member);
            });
        });
    });
}

fn bench_roundtrip_no_cache(c: &mut Criterion) {
    Python::initialize();

    c.bench_function("roundtrip_no_cache", |b| {
        b.iter(|| {
            let _ = Python::attach(|py| {
                // Runtime: FULL rebuild + extraction
                let member = build_enum_no_cache(py, 1).unwrap();
                let value: i32 = member.getattr("value")?.extract()?;
                let variant = match value {
                    1 => VanillaColor::Red,
                    2 => VanillaColor::Green,
                    3 => VanillaColor::Blue,
                    _ => unreachable!(),
                };
                black_box(variant);
                Ok::<_, PyErr>(())
            });
        });
    });
}

criterion_group!(
    benches,
    bench_first_construct_pyenum,
    bench_first_construct_vanilla,
    bench_to_python_pyenum,
    bench_to_python_vanilla,
    bench_to_python_class_only_cached,
    bench_to_python_no_cache,
    bench_from_python_pyenum,
    bench_from_python_vanilla,
    bench_roundtrip_pyenum,
    bench_roundtrip_vanilla,
    bench_roundtrip_class_only_cached,
    bench_roundtrip_no_cache,
    bench_getattr_pyenum,
    bench_getattr_vanilla,
    bench_equality_pyenum,
    bench_equality_vanilla,
);
criterion_main!(benches);
