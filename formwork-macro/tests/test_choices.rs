use formwork::Formwork;
use formwork::choice::{Cardinality, MultiChoice};
use formwork::slot::{MultiSlot, SingleSlot};
use formwork_macro::formwork;

#[formwork]
struct Invocation {
    #[slot]
    spell_choice: String,
}

#[formwork]
struct Warlock {
    #[slot(pick, cascade)]
    invocations: Vec<Invocation>,
}

#[test]
fn test_nested_choice() {
    let mut warlock: WarlockFormwork = Warlock {
        invocations: MultiSlot::Choose(
            MultiChoice::new(
                "Pick 2 Invocations",
                vec![
                    InvocationFormwork {
                        spell_choice: SingleSlot::FixedWet("Eldritch Blast".to_string()),
                    },
                    InvocationFormwork {
                        spell_choice: SingleSlot::FixedWet("Hex".to_string()),
                    },
                    InvocationFormwork {
                        spell_choice: SingleSlot::FixedWet("Armor of Agathys".to_string()),
                    },
                ],
            )
            .with_cardinality(Cardinality::Exact(2)),
        ),
    };

    assert!(!warlock.is_complete());

    if let Some(c) = warlock.invocations.as_choice_mut() {
        c.select(0).unwrap();
        c.select(2).unwrap();
        c.confirm().unwrap();
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
