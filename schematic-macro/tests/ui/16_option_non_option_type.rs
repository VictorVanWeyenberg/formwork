use schematic_macro::schematic;

#[schematic]
struct BadStruct {
    #[slot(option)]
    bad_field: String,
}

fn main() {}
