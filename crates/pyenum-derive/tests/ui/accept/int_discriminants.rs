use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntEnum")]
pub enum Status {
    Ok = 200,
    NotFound = 404,
    Teapot = 418,
}

fn main() {}
