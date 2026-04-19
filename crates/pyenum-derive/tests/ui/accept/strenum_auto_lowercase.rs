use pyenum_derive::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Greeting {
    Hello,
    World,
}

fn main() {}
