use schematic_macro::schematic;
use schematic::Schematic;

#[schematic(builder)]
struct ErgonomicRoot {
    #[slot]
    leaf: String,
    #[slot(collection)]
    dwarf_traits: Vec<String>,
}

#[test]
fn test_ergonomic_builder() {
    let r: ErgonomicRootSchematic = ErgonomicRoot::builder()
        .leaf("hello".to_string())
        .with_dwarf_traits("trait1".to_string())
        .with_dwarf_traits("trait2".to_string())
        .build();

    assert!(r.is_complete());
    let resolved = r.resolve().unwrap();
    assert_eq!(resolved.leaf, "hello");
    assert_eq!(resolved.dwarf_traits, vec!["trait1".to_string(), "trait2".to_string()]);
}

#[test]
fn test_ergonomic_builder_choice() {
    use schematic::choice::MultiChoice;
    
    let mut r: ErgonomicRootSchematic = ErgonomicRoot::builder()
        .leaf("hello".to_string())
        .with_dwarf_traits(MultiChoice::new("Pick", vec!["trait1".to_string(), "trait2".to_string()]))
        .build();

    assert!(!r.is_complete());
    if let Some(c) = r.dwarf_traits[0].as_choice_mut() {
        c.select(0).unwrap();
        c.confirm().unwrap();
    }
    let resolved = r.resolve().unwrap();
    assert_eq!(resolved.dwarf_traits, vec!["trait1".to_string()]);
}
