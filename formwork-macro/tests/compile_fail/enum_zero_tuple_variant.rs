use formwork::formwork;

#[formwork]
pub struct Inner {
    #[slot]
    pub name: String,
}

#[formwork]
pub enum WithZeroTuple {
    A(Inner),
    B(),
}

fn main() {}
