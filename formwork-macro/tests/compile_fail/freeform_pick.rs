use formwork::formwork;

#[formwork]
pub struct InvalidPick {
    #[slot(freeform, pick)]
    pub field: Vec<String>,
}

fn main() {}
