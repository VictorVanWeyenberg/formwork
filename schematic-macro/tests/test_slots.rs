use schematic_macro::schematic;
use schematic::Schematic;

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
fn test_all_slot_types() {
    let _d = RootUnresolved;
    let _f = RootResolved;
    
    let mut _r: RootSchematic = Root {
        leaf: schematic::Slot::Fixed("hi".to_string()),
        dwarf_traits: vec![
            schematic::Slot::Fixed("trait1".to_string()),
            schematic::Slot::Choose(schematic::Choice::new("Pick trait", vec![
                "trait2".to_string(),
                "trait3".to_string(),
            ])),
        ],
        class_skills: schematic::Slot::Choose(
            schematic::Choice::new("Pick skills", vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
            ]).with_cardinality(schematic::Cardinality::Exact(2))
        ),
    };
    
    assert!(!_r.is_complete());
    
    if let Some(c) = _r.dwarf_traits[1].as_choice_mut() {
        c.select(0).unwrap();
    }
    
    if let Some(c) = _r.class_skills.as_choice_mut() {
        c.select(0).unwrap();
        c.select(1).unwrap();
    }
    
    assert!(_r.is_complete());
    
    let resolved_root = _r.resolve().unwrap();
    assert_eq!(resolved_root.leaf, "hi");
    assert_eq!(resolved_root.dwarf_traits, vec!["trait1".to_string(), "trait2".to_string()]);
    assert_eq!(resolved_root.class_skills, vec!["A".to_string(), "B".to_string()]);
}
