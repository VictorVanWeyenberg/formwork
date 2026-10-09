use schematic_macro::schematic;
use schematic::choice::{SingleChoice, MultiChoice, Cardinality};
use schematic::slot::{SingleSlot, MultiSlot};

#[schematic(builder)]
struct Root {
    #[slot]
    leaf: String,
    #[slot(collection)]
    dwarf_traits: Vec<String>,
}

fn main() {
    let mut _r: RootSchematic = Root {
        leaf: MultiSlot::Choose(MultiChoice::new("Pick leaf", vec!["A".to_string()])), // Expected SingleSlot
        dwarf_traits: SingleSlot::Choose(SingleChoice::new("Pick trait", vec!["A".to_string()])), // Expected MultiSlot
    };
}
