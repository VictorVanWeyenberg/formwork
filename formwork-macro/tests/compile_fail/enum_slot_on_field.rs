use formwork::formwork;

#[formwork]
pub struct Inner {
    #[slot]
    pub name: String,
}

#[formwork]
pub enum SlotOnFieldBare {
    A(#[slot] Inner),
}

#[formwork]
pub enum SlotOnFieldCascade {
    A(#[slot(cascade)] Inner),
}

fn main() {}
