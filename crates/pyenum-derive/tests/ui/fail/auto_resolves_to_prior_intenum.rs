use pyenum::PyEnum;

// `A = 5`, `B` auto-resolves to 6, `C = 6` collides with `B`. Python's
// `enum.IntEnum` would silently alias `C` to `B`, breaking Rust-side
// round-trip identity.
#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntEnum")]
pub enum Bad {
    A = 5,
    B,
    C = 6,
}

fn main() {}
