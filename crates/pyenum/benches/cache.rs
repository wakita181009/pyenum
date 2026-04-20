//! Cache + first-build performance benchmarks.
//!
//! * First construction: `< 2 ms` for enums up to 32 variants, `< 20 ms` up to
//!   1,024 variants.
//! * Steady-state conversion (cache hit): `< 1 µs` per call.
//! * Scaling: linear in variant count, no worse.
//!
//! Run locally with `cargo bench -p pyenum`. Criterion writes HTML reports to
//! `target/criterion/`.

use criterion::{Criterion, criterion_group, criterion_main};
use pyenum::__private::{build_py_enum, get_or_build};
use pyenum::{PyEnum, PyEnumBase, PyEnumSpec, PyEnumTrait, VariantLiteral};
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyType;
use std::hint::black_box;
use std::sync::OnceLock;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntEnum")]
#[allow(dead_code)]
enum BenchColor {
    Red = 1,
    Green = 2,
    Blue = 3,
}

fn make_variants(n: usize) -> &'static [(&'static str, VariantLiteral)] {
    let mut v: Vec<(&'static str, VariantLiteral)> = Vec::with_capacity(n);
    for i in 0..n {
        let name: &'static str = Box::leak(format!("V{i}").into_boxed_str());
        v.push((name, VariantLiteral::Int(i as i64)));
    }
    Box::leak(v.into_boxed_slice())
}

fn spec_32() -> &'static PyEnumSpec {
    static CELL: OnceLock<PyEnumSpec> = OnceLock::new();
    CELL.get_or_init(|| PyEnumSpec {
        name: "BenchEnum32",
        base: PyEnumBase::IntEnum,
        variants: make_variants(32),
        module: None,
        qualname: None,
    })
}

fn spec_1024() -> &'static PyEnumSpec {
    static CELL: OnceLock<PyEnumSpec> = OnceLock::new();
    CELL.get_or_init(|| PyEnumSpec {
        name: "BenchEnum1024",
        base: PyEnumBase::IntEnum,
        variants: make_variants(1024),
        module: None,
        qualname: None,
    })
}

fn spec_cache_hit() -> &'static PyEnumSpec {
    static CELL: OnceLock<PyEnumSpec> = OnceLock::new();
    CELL.get_or_init(|| PyEnumSpec {
        name: "CacheHitEnum",
        base: PyEnumBase::IntEnum,
        variants: &[
            ("A", VariantLiteral::Int(0)),
            ("B", VariantLiteral::Int(1)),
            ("C", VariantLiteral::Int(2)),
        ],
        module: None,
        qualname: None,
    })
}

fn bench_first_build_32(c: &mut Criterion) {
    Python::initialize();
    let spec = spec_32();
    c.bench_function("first_build_32_variants", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let cls = build_py_enum(py, black_box(spec)).unwrap();
                black_box(cls);
            });
        });
    });
}

fn bench_first_build_1024(c: &mut Criterion) {
    Python::initialize();
    let spec = spec_1024();
    let mut group = c.benchmark_group("first_build_1024_variants");
    // Each sample is expensive (~10+ ms); keep the sample count modest so
    // the bench finishes in a reasonable time.
    group.sample_size(20);
    group.bench_function("build", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let cls = build_py_enum(py, black_box(spec)).unwrap();
                black_box(cls);
            });
        });
    });
    group.finish();
}

fn bench_cache_hit(c: &mut Criterion) {
    Python::initialize();
    static CACHE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    let spec = spec_cache_hit();
    // Prime the cache so the benchmarked call always hits the fast path.
    Python::attach(|py| {
        let _ = get_or_build(py, &CACHE, spec).unwrap();
    });
    c.bench_function("cache_hit_get_class", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let cls = get_or_build(py, &CACHE, black_box(spec)).unwrap();
                black_box(cls);
            });
        });
    });
}

fn bench_to_py_member(c: &mut Criterion) {
    Python::initialize();
    // Prime the derive's internal cache so the benchmarked call is always a
    // hot-path member lookup.
    Python::attach(|py| {
        let _ = BenchColor::py_enum_class(py).unwrap();
        let _ = BenchColor::Red.to_py_member(py).unwrap();
    });
    c.bench_function("to_py_member_hotpath", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let member = BenchColor::Green.to_py_member(py).unwrap();
                black_box(member);
            });
        });
    });
}

fn bench_from_py_member(c: &mut Criterion) {
    Python::initialize();
    // Materialise the Python member object once; every benchmarked call just
    // round-trips it through `FromPyObject`.
    let owned = Python::attach(|py| BenchColor::Blue.to_py_member(py).unwrap().unbind());
    c.bench_function("from_py_member_hotpath", |b| {
        b.iter(|| {
            Python::attach(|py| {
                let variant = BenchColor::from_py_member(owned.bind(py)).unwrap();
                black_box(variant);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_first_build_32,
    bench_first_build_1024,
    bench_cache_hit,
    bench_to_py_member,
    bench_from_py_member,
);
criterion_main!(benches);
