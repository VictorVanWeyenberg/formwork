use formwork::Formwork;
use formwork::choice::{Cardinality, MultiChoice};
use formwork_macro::formwork;

#[formwork(builder)]
struct Root {
    #[slot]
    leaf: String,
    #[slot(collection)]
    dwarf_traits: Vec<String>,
    #[slot(pick)]
    class_skills: Vec<String>,
}

#[test]
fn test_builders() {
    let mut wet_root: RootFormwork = Root::builder()
        .leaf("hi".to_string())
        .with_dwarf_traits("trait1".to_string())
        .with_dwarf_traits("trait2".to_string())
        .class_skills(
            MultiChoice::new(
                "Pick skills",
                vec!["A".to_string(), "B".to_string(), "C".to_string()],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )
        .build();

    assert!(!wet_root.is_complete());

    if let Some(c) = wet_root.class_skills.as_choice_mut() {
        c.select(0).unwrap();
        c.select(1).unwrap();
        c.confirm().unwrap();
    }

    assert!(wet_root.is_complete());

    let resolved_root = wet_root.resolve().unwrap();
    assert_eq!(resolved_root.leaf, "hi");
    assert_eq!(
        resolved_root.dwarf_traits,
        vec!["trait1".to_string(), "trait2".to_string()]
    );
    assert_eq!(
        resolved_root.class_skills,
        vec!["A".to_string(), "B".to_string()]
    );
}

#[formwork(builder)]
struct EntityWithDefaults {
    #[builder(default)]
    optional_tag: Option<String>,
    #[builder(default = 42)]
    custom_number: i32,
    #[slot]
    name: String,
}

#[test]
fn test_builder_with_field_defaults() {
    // Both optional_tag and custom_number are omitted; they should take their defaults.
    let wet_entity: EntityWithDefaultsFormwork = EntityWithDefaults::builder()
        .name("hero".to_string())
        .build();

    let resolved = wet_entity.resolve().unwrap();
    assert_eq!(resolved.optional_tag, None);
    assert_eq!(resolved.custom_number, 42);
    assert_eq!(resolved.name, "hero");

    // Setting them explicitly should also work (and setter(into) should be preserved).
    let wet_entity2: EntityWithDefaultsFormwork = EntityWithDefaults::builder()
        .optional_tag("custom_tag".to_string())
        .custom_number(100)
        .name("villain".to_string())
        .build();

    let resolved2 = wet_entity2.resolve().unwrap();
    assert_eq!(resolved2.optional_tag, Some("custom_tag".to_string()));
    assert_eq!(resolved2.custom_number, 100);
    assert_eq!(resolved2.name, "villain");
}
