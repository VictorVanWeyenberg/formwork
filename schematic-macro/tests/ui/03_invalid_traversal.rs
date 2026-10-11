use schematic_macro::schematic;

#[schematic(traverse = "diagonal")]
struct BadStruct {
    #[slot]
    field: String,
}

fn main() {}
