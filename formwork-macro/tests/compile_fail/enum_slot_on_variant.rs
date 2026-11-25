use formwork::formwork;

#[formwork]
pub struct Inner {
    #[slot]
    pub name: String,
}

#[formwork]
pub enum SlotOnVariantBare {
    #[slot]
    A(Inner),
}

#[formwork]
pub enum SlotOnVariantCascade {
    #[slot(cascade)]
    A(Inner),
}

fn main() {}
