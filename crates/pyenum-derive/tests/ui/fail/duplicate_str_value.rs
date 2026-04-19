use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Bad {
    #[pyenum(value = "red")]
    Red,
    #[pyenum(value = "red")]
    Crimson,
}

fn main() {}
