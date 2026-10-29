use schematic_macro::schematic;

#[schematic]
struct BadStruct {
    #[slot(cascade)]
    bad_field: Option<String>,
}

fn main() {}
