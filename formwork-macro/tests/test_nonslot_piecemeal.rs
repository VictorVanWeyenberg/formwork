use formwork::Formwork;
use formwork_macro::formwork;

#[formwork(builder)]
struct NonslotCollectionExample {
    #[slot]
    id: String,
    items: Vec<String>,
}

#[test]
fn test_nonslot_piecemeal_mutators() {
    let example = NonslotCollectionExample::builder()
        .id("test".to_string())
        .with_items("Fixed 1".to_string())
        .with_items("Fixed 2".to_string())
        .build();

    let resolved = example.resolve().unwrap();
    assert_eq!(
        resolved.items,
        vec!["Fixed 1".to_string(), "Fixed 2".to_string(),]
    );
}
