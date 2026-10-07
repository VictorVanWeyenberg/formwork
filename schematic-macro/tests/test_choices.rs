use schematic_macro::schematic;
use schematic::Schematic;

#[schematic]
struct Invocation {
    #[slot]
    spell_choice: String,
}

#[schematic]
struct Warlock {
    #[slot(pick, cascade)]
    invocations: Vec<Invocation>,
}

#[test]
fn test_nested_choice() {
    let mut warlock: WarlockSchematic = Warlock {
        invocations: schematic::Slot::Choose(
            schematic::Choice::new("Pick 2 Invocations", vec![
                Invocation {
                    spell_choice: schematic::Slot::Fixed("Eldritch Blast".to_string()),
                },
                Invocation {
                    spell_choice: schematic::Slot::Fixed("Hex".to_string()),
                },
                Invocation {
                    spell_choice: schematic::Slot::Fixed("Armor of Agathys".to_string()),
                },
            ]).with_cardinality(schematic::Cardinality::Exact(2))
        ),
    };
    
    assert!(!warlock.is_complete());
    
    if let Some(c) = warlock.invocations.as_choice_mut() {
        c.select(0).unwrap();
        c.select(2).unwrap();
    }
    
    assert!(warlock.is_complete());
    
    let resolved_warlock = warlock.resolve().unwrap();
    assert_eq!(resolved_warlock.invocations.len(), 2);
    assert_eq!(resolved_warlock.invocations[0].spell_choice, "Eldritch Blast");
    assert_eq!(resolved_warlock.invocations[1].spell_choice, "Armor of Agathys");
}
