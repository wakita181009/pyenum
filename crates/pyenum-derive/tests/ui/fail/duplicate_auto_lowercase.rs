use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Bad {
    Hello,
    HELLO,
}

fn main() {}
