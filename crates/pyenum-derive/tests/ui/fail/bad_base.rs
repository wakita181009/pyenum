use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "NotARealBase")]
pub enum Bad {
    A,
    B,
}

fn main() {}
