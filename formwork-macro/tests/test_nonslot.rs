use formwork::Formwork;
use formwork::slot::SingleSlot;
use formwork_macro::formwork;

#[formwork(builder)]
struct Mixed {
    id: String,
    #[slot]
    value: i32,
    tags: Vec<String>,
}

#[test]
fn test_mixed_fields() {
    let m: MixedFormwork = Mixed {
        id: "my_id".to_string(),
        value: SingleSlot::FixedWet(42),
        tags: vec!["a".to_string(), "b".to_string()],
    };

    assert!(m.is_complete());

    let resolved = m.resolve().unwrap();
    assert_eq!(resolved.id, "my_id");
    assert_eq!(resolved.value, 42);
    assert_eq!(resolved.tags, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn test_mixed_builder() {
    let m: MixedFormwork = Mixed::builder()
        .id("my_id".to_string())
        .value(42)
        .with_tags("a".to_string())
        .with_tags("b".to_string())
        .build();

    assert!(m.is_complete());

    let resolved = m.resolve().unwrap();
    assert_eq!(resolved.id, "my_id");
    assert_eq!(resolved.value, 42);
    assert_eq!(resolved.tags, vec!["a".to_string(), "b".to_string()]);
}
