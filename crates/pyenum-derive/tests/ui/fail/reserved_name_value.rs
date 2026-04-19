use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
pub enum Bad {
    Ok,
    value,
}

fn main() {}
