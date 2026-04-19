use pyenum_derive::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "StrEnum")]
pub enum Language {
    #[pyenum(value = "Rust")]
    Rust,
    #[pyenum(value = "Python")]
    Python,
}

fn main() {}
