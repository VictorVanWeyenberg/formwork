use schematic_macro::schematic;

#[schematic]
struct BadCollection {
    #[slot(collection)]
    pub item: i32,
}

fn main() {}
