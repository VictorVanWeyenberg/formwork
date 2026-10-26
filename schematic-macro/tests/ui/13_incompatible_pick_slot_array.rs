use schematic_macro::schematic;

#[schematic(builder)]
struct Target {
    #[slot(pick)]
    pub nums: Vec<u16>,
}

fn main() {
    // Attempting to pass [u32; 2] into Vec<u16> pick slot
    let _ = Target::builder().nums([10u32, 20u32]).build();
}
