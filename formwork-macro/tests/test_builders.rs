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
