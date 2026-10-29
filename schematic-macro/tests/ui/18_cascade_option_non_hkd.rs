use schematic_macro::schematic;

#[schematic]
struct BadStruct {
    #[slot(option, cascade)]
    bad_field: Option<i32>,
}

fn main() {}
