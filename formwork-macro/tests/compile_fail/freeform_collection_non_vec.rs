use formwork::formwork;

#[formwork]
pub struct InvalidCollection {
    #[slot(freeform, collection)]
    pub field: String,
}

fn main() {}
