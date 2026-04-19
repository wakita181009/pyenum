//! Compile-time contract for `#[derive(PyEnum)]`.
//!
//! Runs every fixture under `tests/ui/accept/` (must compile) and every
//! fixture under `tests/ui/fail/` (must fail to compile against the
//! committed `.stderr` snapshot).
//!
//! Regenerate snapshots explicitly:
//!     TRYBUILD=overwrite cargo test -p pyenum-derive --test trybuild

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/accept/*.rs");
    t.compile_fail("tests/ui/fail/*.rs");
}
