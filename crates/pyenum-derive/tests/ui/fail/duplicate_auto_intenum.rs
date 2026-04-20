use pyenum::PyEnum;

// `A` auto-resolves to 1 (IntEnum start value); `B = 1` collides and would
// be silently aliased to `A` by Python's `enum.IntEnum` functional API,
// breaking Rust-side round-trip identity.
#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntEnum")]
pub enum Bad {
    A,
    B = 1,
}

fn main() {}
