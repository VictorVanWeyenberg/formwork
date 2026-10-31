use formwork::Formwork;
use formwork::choice::{Cardinality, MultiChoice, SingleChoice};
use formwork::slot::{MultiSlot, SingleSlot};
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
fn test_all_slot_types() {
    let mut _r: RootFormwork = Root {
        leaf: SingleSlot::FixedWet("hi".to_string()),
        dwarf_traits: vec![
            SingleSlot::Choose(SingleChoice::new(
                "Pick trait 1",
                vec!["trait1".to_string()],
            )),
            SingleSlot::FixedWet("trait2".to_string()),
        ],
        class_skills: MultiSlot::Choose(
            MultiChoice::new(
                "Pick skills",
                vec!["A".to_string(), "B".to_string(), "C".to_string()],
            )
            .with_cardinality(Cardinality::Exact(2)),
        ),
    };

    assert!(!_r.is_complete());

    if let Some(c) = _r.dwarf_traits[0].as_choice_mut() {
        c.select(0).unwrap();
        c.confirm().unwrap();
    }

    if let Some(c) = _r.class_skills.as_choice_mut() {
        c.select(0).unwrap();
        c.select(1).unwrap();
        c.confirm().unwrap();
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
