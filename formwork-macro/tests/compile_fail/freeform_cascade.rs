use formwork::formwork;

#[formwork]
pub struct Child {
    #[slot]
    pub name: String,
}

#[formwork]
pub struct InvalidCascade {
    #[slot(freeform, cascade)]
    pub child: Child,
}

fn main() {}
