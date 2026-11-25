use formwork::formwork;

#[formwork]
pub struct Inner {
    #[slot]
    pub name: String,
}

#[formwork(builder)]
pub enum EnumWithBuilderTrue {
    A(Inner),
}

#[formwork(builder = false)]
pub enum EnumWithBuilderFalse {
    A(Inner),
}

fn main() {}
