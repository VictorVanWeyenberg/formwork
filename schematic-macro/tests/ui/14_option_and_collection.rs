use schematic_macro::schematic;

#[schematic]
struct BadStruct {
    #[slot(option, collection)]
    bad_field: Option<String>,
}

fn main() {}
