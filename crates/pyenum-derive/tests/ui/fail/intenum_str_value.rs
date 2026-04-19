use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
#[pyenum(base = "IntEnum")]
pub enum Bad {
    #[pyenum(value = "ok")]
    Ok,
    #[pyenum(value = "no")]
    No,
}

fn main() {}
