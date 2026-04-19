use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Bad {
    Ok = 1,
    No = 2,
}

fn main() {}
