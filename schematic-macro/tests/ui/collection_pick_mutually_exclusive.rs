use schematic_macro::schematic;

#[schematic]
struct BadStruct {
    #[slot(collection, pick)]
    bad_field: Vec<String>,
}

fn main() {}
