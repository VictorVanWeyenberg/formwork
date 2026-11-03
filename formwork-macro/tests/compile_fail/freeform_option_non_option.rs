use formwork::formwork;

#[formwork]
pub struct InvalidOption {
    #[slot(freeform, option)]
    pub field: String,
}

fn main() {}
