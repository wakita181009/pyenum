use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
pub enum Color {
    Red,
    Green,
    Blue,
}

fn main() {}
