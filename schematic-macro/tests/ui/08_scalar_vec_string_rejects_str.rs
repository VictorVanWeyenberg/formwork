use schematic_macro::schematic;

#[schematic(builder)]
struct StringVecTarget {
    #[slot]
    pub texts: Vec<String>,
}

fn main() {
    let _ = StringVecTargetSchematic::builder()
        .texts("hello")
        .build();
}
