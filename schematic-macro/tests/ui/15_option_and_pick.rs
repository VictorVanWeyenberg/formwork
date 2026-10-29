use schematic_macro::schematic;

#[schematic]
struct BadStruct {
    #[slot(option, pick)]
    bad_field: Option<String>,
}

fn main() {}
