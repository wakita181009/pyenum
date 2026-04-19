use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "Enum")]
pub enum Bad {
    #[pyenum(value = "one")]
    One = 1,
}

fn main() {}
