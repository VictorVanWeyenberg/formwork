use schematic_macro::schematic;

#[schematic]
struct BadPick {
    #[slot(pick)]
    pub item: String,
}

fn main() {}
