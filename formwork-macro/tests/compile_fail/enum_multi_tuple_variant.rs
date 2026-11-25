use formwork::formwork;

#[formwork]
pub struct InnerA {
    #[slot]
    pub name: String,
}

#[formwork]
pub struct InnerB {
    #[slot]
    pub value: i32,
}

#[formwork]
pub enum WithMultiTuple {
    A(InnerA),
    B(InnerA, InnerB),
}

fn main() {}
