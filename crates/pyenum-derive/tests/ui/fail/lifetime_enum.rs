use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
pub enum Bad<'a> {
    Borrowed(&'a str),
}

fn main() {}
