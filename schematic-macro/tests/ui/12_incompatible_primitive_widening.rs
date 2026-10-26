use schematic_macro::schematic;

#[schematic(builder)]
struct Target {
    #[slot]
    pub num: u16,
}

fn main() {
    // Attempting to pass u32 into u16 slot (narrowing is not supported by Into)
    let _ = Target::builder().num(100u32).build();
}
