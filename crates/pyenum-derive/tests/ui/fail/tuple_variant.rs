use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
pub enum Bad {
    Unit,
    Tupled(u8),
}

fn main() {}
