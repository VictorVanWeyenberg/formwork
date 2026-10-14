use schematic::Schematic;
use schematic::choice::SingleChoice;
use schematic_macro::schematic;

#[schematic(builder)]
struct CollectionExample {
    #[slot(collection)]
    items: Vec<String>,
}

#[test]
fn test_piecemeal_mutators() {
    let mut example: CollectionExampleSchematic = CollectionExample::builder()
        .with_items("Fixed 1".to_string())
        .with_items(SingleChoice::new(
            "Pick 1",
            vec![
                "Choice A".to_string(),
                "Choice B".to_string(),
                "Choice C".to_string(),
            ],
        ))
        .with_items("Fixed 2".to_string())
        .build();

    assert!(!example.is_complete());

    // Resolve the choice, which should be the second item in the items vector.
    if let Some(c) = example.items[1].as_choice_mut() {
        c.select(0).unwrap();
        c.confirm().unwrap();
    } else {
        panic!("Second item was not a choice as expected.");
    }

    assert!(example.is_complete());

    let resolved = example.resolve().unwrap();
    assert_eq!(
        resolved.items,
        vec![
            "Fixed 1".to_string(),
            "Choice A".to_string(),
            "Fixed 2".to_string(),
        ]
    );
}
