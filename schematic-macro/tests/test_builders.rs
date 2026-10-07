use schematic_macro::schematic;
use schematic::Schematic;
use schematic::choice::{MultiChoice, Cardinality};

#[schematic(builder)]
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
    let mut unresolved: RootSchematic = Root::builder()
        .leaf("hi".to_string())
        .with_dwarf_traits("trait1".to_string())
        .with_class_skills(MultiChoice::new("Pick skills", vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]).with_cardinality(Cardinality::Exact(2)))
        .build();

    unresolved.dwarf_traits.push(schematic::MultiSlot::FixedUnresolved("trait2".to_string()));

    assert!(!unresolved.is_complete());
    
    if let Some(c) = unresolved.class_skills[0].as_choice_mut() {
        c.select(0).unwrap();
        c.select(1).unwrap();
    }
    
    assert!(unresolved.is_complete());
    
    let resolved_root = unresolved.resolve().unwrap();
    assert_eq!(resolved_root.leaf, "hi");
    assert_eq!(resolved_root.dwarf_traits, vec!["trait1".to_string(), "trait2".to_string()]);
    assert_eq!(resolved_root.class_skills, vec!["A".to_string(), "B".to_string()]);
}
