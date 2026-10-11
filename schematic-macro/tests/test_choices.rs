use schematic::Schematic;
use schematic::choice::{Cardinality, MultiChoice};
use schematic::slot::{MultiSlot, SingleSlot};
use schematic_macro::schematic;

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
        invocations: vec![MultiSlot::Choose(
            MultiChoice::new(
                "Pick 2 Invocations",
                vec![
                    InvocationSchematic {
                        spell_choice: SingleSlot::FixedUnresolved("Eldritch Blast".to_string()),
                    },
                    InvocationSchematic {
                        spell_choice: SingleSlot::FixedUnresolved("Hex".to_string()),
                    },
                    InvocationSchematic {
                        spell_choice: SingleSlot::FixedUnresolved("Armor of Agathys".to_string()),
                    },
                ],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )],
    };

    assert!(!warlock.is_complete());

    if let Some(c) = warlock.invocations[0].as_choice_mut() {
        c.select(0).unwrap();
        c.select(2).unwrap();
    }

    assert!(warlock.is_complete());

    let resolved_warlock = warlock.resolve().unwrap();
    assert_eq!(resolved_warlock.invocations.len(), 2);
    assert_eq!(
        resolved_warlock.invocations[0].spell_choice,
        "Eldritch Blast"
    );
    assert_eq!(
        resolved_warlock.invocations[1].spell_choice,
        "Armor of Agathys"
    );
}
