use schematic::Schematic;
use schematic::choice::{Cardinality, MultiChoice};
use schematic::slot::{MultiSlot, SingleSlot};
use schematic_macro::schematic;

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
    let mut _r: RootSchematic = Root {
        leaf: SingleSlot::FixedUnresolved("hi".to_string()),
        dwarf_traits: vec![MultiSlot::Choose(
            MultiChoice::new(
                "Pick traits",
                vec!["trait1".to_string(), "trait2".to_string()],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )],
        class_skills: vec![MultiSlot::Choose(
            MultiChoice::new(
                "Pick skills",
                vec!["A".to_string(), "B".to_string(), "C".to_string()],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )],
    };

    assert!(!_r.is_complete());

    if let Some(c) = _r.dwarf_traits[0].as_choice_mut() {
        c.select(0).unwrap();
        c.select(1).unwrap();
    }

    if let Some(c) = _r.class_skills[0].as_choice_mut() {
        c.select(0).unwrap();
        c.select(1).unwrap();
    }

    assert!(_r.is_complete());

    let resolved_root = _r.resolve().unwrap();
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
