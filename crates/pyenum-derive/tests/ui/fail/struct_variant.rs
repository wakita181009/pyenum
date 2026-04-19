use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
pub enum Bad {
    Unit,
    Structured { inner: u8 },
}

fn main() {}
