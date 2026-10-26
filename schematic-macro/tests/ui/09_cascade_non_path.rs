use schematic_macro::schematic;

#[schematic]
#[derive(Clone)]
struct Child {
    #[slot]
    pub id: u32,
}

#[schematic]
struct BadCascadeNonPath {
    #[slot(cascade)]
    pub pair: (Child, Child),
}

fn main() {}
