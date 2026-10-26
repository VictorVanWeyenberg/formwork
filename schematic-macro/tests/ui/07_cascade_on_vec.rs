use schematic_macro::schematic;

#[schematic]
#[derive(Clone)]
struct Child {
    #[slot]
    pub id: u32,
}

#[schematic]
struct BadCascade {
    #[slot(cascade)]
    pub children: Vec<Child>,
}

fn main() {}
