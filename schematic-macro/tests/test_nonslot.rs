use schematic_macro::schematic;
use schematic::Schematic;

#[schematic(builder)]
struct Mixed {
    id: String,
    #[slot]
    value: i32,
    tags: Vec<String>,
}

#[test]
fn test_mixed_fields() {
    let _d = MixedUnresolved;
    let _f = MixedResolved;
    
    // The schematic type should have normal `id` and `tags`, but `value` is a Slot
    let m: MixedSchematic = Mixed {
        id: "my_id".to_string(),
        value: schematic::Slot::Fixed(42),
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
    let m: MixedSchematic = Mixed::builder()
        .id("my_id")
        .value(42)
        .with_tags("a")
        .with_tags("b")
        .build();
    
    assert!(m.is_complete());
    
    let resolved = m.resolve().unwrap();
    assert_eq!(resolved.id, "my_id");
    assert_eq!(resolved.value, 42);
    assert_eq!(resolved.tags, vec!["a".to_string(), "b".to_string()]);
}
