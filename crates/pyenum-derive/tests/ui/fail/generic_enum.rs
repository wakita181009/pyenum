use pyenum::PyEnum;

#[derive(Clone, Copy, PyEnum)]
pub enum Bad<T> {
    A(std::marker::PhantomData<T>),
}

fn main() {}
