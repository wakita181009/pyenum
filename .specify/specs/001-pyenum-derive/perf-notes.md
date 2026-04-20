# Performance notes (SC-004)

Measured via `cargo bench -p pyenum --bench cache` (see
`crates/pyenum/benches/cache.rs`). Criterion 0.8, release profile.
Environment: macOS (Darwin 25.3.0), CPython 3.11+ via PyO3 0.28.

## SC-004 targets vs. measurements

| Benchmark | Target | Measured (median) | Margin | Status |
| --- | --- | --- | --- | --- |
| First build, 32 variants   | `< 2 ms`   | ~173 µs   | ~11.5× under | ✅ |
| First build, 1,024 variants | `< 20 ms`  | ~8.86 ms  | ~2.3× under  | ✅ |
| Cache-hit class lookup     | `< 1 µs`   | ~64 ns    | ~15.5× under | ✅ |

All three absolute targets are met on the reference host with healthy
headroom. No optimisation work is required before v1.

## Scaling observation

The spec also requires construction to be "linear in variant count, no
worse." Comparing the two first-build benches:

- 32 → 1,024 variants is a **32× increase** in input size.
- 173 µs → 8.86 ms is a **~51× increase** in time (≈ 1.6× worse than
  linear).

This super-linearity comes from CPython's `enum` metaclass bookkeeping
(per-member descriptor setup, `_value2member_map_` population), not from
the PyO3 bridge. Even at 1,024 variants we still sit at ~44% of the hard
20 ms budget, so the super-linear constant is not load-bearing for v1.

Action: **no change**. If future users add >~2,000-variant enums, revisit
— candidate optimisations include:

- Caching the `enum` module / base-class / `auto` lookups at derive-time
  instead of re-importing them inside `build_py_enum`.
- Building the members list via `PyList::new(py, iter)` in one pass
  instead of `append()`-per-variant.

## Reproduction

```shell
cargo bench -p pyenum --bench cache
```

Criterion writes HTML reports to `target/criterion/`. Re-run after any
change to `crates/pyenum/src/construct.rs` or `cache.rs` to confirm no
regression.
