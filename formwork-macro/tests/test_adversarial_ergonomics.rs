use formwork_macro::formwork;

#[formwork(builder)]
pub struct Child {
    #[slot]
    name: String,

    non_slot_name: String,
}

#[test]
fn test_adversarial_builder() {
    let _c: ChildFormwork = Child::builder()
        .non_slot_name("hello")
        .name("wet".to_string())
        .build();
}
