use schematic_macro::schematic;

#[schematic]
struct BadStruct {
    #[slot(invalid_arg)]
    bad_field: String,
}

fn main() {}
