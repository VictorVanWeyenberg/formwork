use formwork::formwork;

#[formwork]
pub struct Inner {
    #[slot]
    pub name: String,
}

#[formwork]
pub enum WithStruct {
    A(Inner),
    B { inner: Inner },
}

fn main() {}
