use schematic_macro::schematic;

#[schematic(builder)]
pub struct Child {
    #[slot]
    name: String,

    non_slot_name: String,
}

#[test]
fn test_adversarial_builder() {
    let _c: ChildSchematic = Child::builder()
        .non_slot_name("hello")
        .name("unresolved")
        .build();
}
