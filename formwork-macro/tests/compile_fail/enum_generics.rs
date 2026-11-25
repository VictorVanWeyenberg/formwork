use formwork::formwork;

#[formwork]
pub struct Inner {
    #[slot]
    pub name: String,
}

#[formwork]
pub enum GenericType<T> {
    A(Inner),
}

#[formwork]
pub enum GenericLifetime<'a> {
    A(Inner),
}

#[formwork]
pub enum GenericConst<const N: usize> {
    A(Inner),
}

#[formwork]
pub enum WithWhereClause
where
    i32: Copy,
{
    A(Inner),
}

fn main() {}
