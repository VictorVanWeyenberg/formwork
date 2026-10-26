use schematic_macro::schematic;

#[schematic]
#[derive(Clone)]
struct Child {
    #[slot]
    pub val: u32,
}

#[schematic]
struct BadPickCascade {
    #[slot(pick, cascade)]
    pub child: Child,
}

fn main() {}
