use pyenum::PyEnum;

// `Read` auto-resolves to 1 (Flag starts at 1); `Write = 1` collides and
// would be silently aliased to `Read` by Python's `enum.Flag` functional
// API, breaking Rust-side round-trip identity.
#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "Flag")]
pub enum Bad {
    Read,
    Write = 1,
}

fn main() {}
