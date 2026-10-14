use schematic_core::choice::{Cardinality, ChoiceState, MultiChoice, SingleChoice};
use schematic_core::error::SchematicError;
use schematic_core::slot::{MultiSlot, SingleSlot, Slot};
use schematic_core::visitor::{AutoSelectFirst, ChoiceVisitor, ConfirmAll, UnmakeAll, VisitorFlow};

#[test]
fn test_adversarial_single_choice_transitions() {
    let mut choice = SingleChoice::new("Weapon", vec!["Sword", "Bow", "Dagger"]);

    // Pending: verify error contracts
    assert!(choice.state().is_pending());
    assert!(!choice.is_made());
    assert_eq!(choice.selected_index(), None);
    assert!(choice.selected_indices().is_empty());
    assert_eq!(choice.selected_value(), None);
    assert!(choice.selected_values().is_empty());

    // Error on pending
    assert!(matches!(
        choice.unmake(),
        Err(SchematicError::CannotUnmakeNotMade { .. })
    ));
    assert!(matches!(
        choice.deselect(0),
        Err(SchematicError::NotSelected { .. })
    ));
    assert!(matches!(
        choice.confirm(),
        Err(SchematicError::InvalidCardinality { actual: 0, .. })
    ));
    assert!(matches!(
        choice.clone().resolve_exact(),
        Err(SchematicError::UnresolvedChoice { .. })
    ));

    // Bounds check
    assert!(matches!(
        choice.select(3),
        Err(SchematicError::IndexOutOfBounds {
            index: 3,
            total_options: 3,
            ..
        })
    ));
    assert!(choice.state().is_pending());

    // Selection stages candidate
    choice.select(2).unwrap();
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(2));
    assert!(!choice.is_made(), "Staging must NEVER set Made");
    assert_eq!(choice.selected_index(), Some(2));
    assert_eq!(choice.selected_value(), Some(&"Dagger"));
    assert!(choice.clone().resolve_exact().is_err());

    // In-place replacement
    choice.select(1).unwrap();
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(1));
    assert_eq!(choice.selected_index(), Some(1));
    assert_eq!(choice.selected_value(), Some(&"Bow"));

    // Deselect wrong index
    assert!(matches!(
        choice.deselect(0),
        Err(SchematicError::NotSelected { .. })
    ));
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(1));

    // Deselect right index returns to Pending
    choice.deselect(1).unwrap();
    assert!(choice.state().is_pending());
    assert_eq!(choice.selected_index(), None);

    // Stage and Confirm
    choice.select(0).unwrap();
    choice.confirm().unwrap();
    assert!(choice.is_made());
    assert_eq!(choice.state(), &ChoiceState::Made(0));

    // Idempotent confirm
    assert!(choice.confirm().is_ok());
    assert!(choice.is_made());

    // Selection on Made choice demotes to UnmadeWithHistory
    choice.select(1).unwrap();
    assert!(
        !choice.is_made(),
        "select on Made must demote to UnmadeWithHistory"
    );
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(1));
    assert!(choice.clone().resolve_exact().is_err());

    // Confirm again
    choice.confirm().unwrap();
    assert!(choice.is_made());

    // Deselect on Made demotes to Pending
    choice.deselect(1).unwrap();
    assert!(choice.state().is_pending());

    // Re-stage and confirm
    choice.select(2).unwrap();
    choice.confirm().unwrap();

    // Unmake transitions to UnmadeWithHistory
    choice.unmake().unwrap();
    assert!(!choice.is_made());
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(2));

    // Calling unmake again is idempotent
    assert!(choice.unmake().is_ok());
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(2));

    // Re-confirm restores Made
    choice.confirm().unwrap();
    assert!(choice.is_made());

    // Resolve returns exact value
    let val = choice.resolve_exact().unwrap();
    assert_eq!(val, "Dagger");
}

#[test]
fn test_adversarial_single_choice_empty_options() {
    let mut choice = SingleChoice::new("Empty", Vec::<String>::new());
    assert!(choice.options().is_empty());
    assert!(choice.state().is_pending());

    assert!(matches!(
        choice.select(0),
        Err(SchematicError::IndexOutOfBounds {
            index: 0,
            total_options: 0,
            ..
        })
    ));
    assert!(matches!(
        choice.confirm(),
        Err(SchematicError::InvalidCardinality { actual: 0, .. })
    ));
    assert!(matches!(
        choice.deselect(0),
        Err(SchematicError::NotSelected { .. })
    ));
    assert!(matches!(
        choice.unmake(),
        Err(SchematicError::CannotUnmakeNotMade { .. })
    ));
    assert!(matches!(
        choice.resolve_exact(),
        Err(SchematicError::UnresolvedChoice { .. })
    ));
}

#[test]
fn test_adversarial_multi_choice_cardinality_boundaries() {
    // 1. Cardinality::Exact(0)
    let mut choice_zero =
        MultiChoice::new("None", vec!["A", "B"]).with_cardinality(Cardinality::Exact(0));
    assert!(!choice_zero.is_made());
    // Confirm on Pending with Exact(0) should succeed!
    choice_zero.confirm().unwrap();
    assert!(choice_zero.is_made());
    assert_eq!(choice_zero.resolve().unwrap(), Vec::<&str>::new());

    // 2. Cardinality::UpTo(0)
    let mut choice_upto_zero =
        MultiChoice::new("None", vec!["A", "B"]).with_cardinality(Cardinality::UpTo(0));
    choice_upto_zero.confirm().unwrap();
    assert!(choice_upto_zero.is_made());
    assert_eq!(choice_upto_zero.resolve().unwrap(), Vec::<&str>::new());

    // 3. Cardinality::UpTo(2) allows 0, 1, or 2 items
    let mut choice_upto = MultiChoice::new("Skills", vec!["Athletics", "Stealth", "Arcana"])
        .with_cardinality(Cardinality::UpTo(2));

    // Confirm with 0 items works for UpTo(2)
    choice_upto.confirm().unwrap();
    assert!(choice_upto.is_made());
    assert!(choice_upto.selected_indices().is_empty());

    // Select on Made demotes to UnmadeWithHistory
    choice_upto.select(0).unwrap();
    assert!(!choice_upto.is_made());
    assert_eq!(
        choice_upto.state(),
        &ChoiceState::UnmadeWithHistory(vec![0])
    );

    choice_upto.select(1).unwrap();
    assert_eq!(
        choice_upto.state(),
        &ChoiceState::UnmadeWithHistory(vec![0, 1])
    );

    choice_upto.confirm().unwrap();
    assert!(choice_upto.is_made());

    // Selecting a 3rd exceeds UpTo(2)
    choice_upto.select(2).unwrap();
    assert_eq!(
        choice_upto.state(),
        &ChoiceState::UnmadeWithHistory(vec![0, 1, 2])
    );
    assert!(matches!(
        choice_upto.confirm(),
        Err(SchematicError::InvalidCardinality { actual: 3, .. })
    ));

    // Deselect index 2 brings it back to 2 items
    choice_upto.deselect(2).unwrap();
    assert_eq!(
        choice_upto.state(),
        &ChoiceState::UnmadeWithHistory(vec![0, 1])
    );
    choice_upto.confirm().unwrap();
    assert!(choice_upto.is_made());

    // Deselect index 1 leaves 1 item, demoted to UnmadeWithHistory
    choice_upto.deselect(1).unwrap();
    assert!(!choice_upto.is_made());
    assert_eq!(
        choice_upto.state(),
        &ChoiceState::UnmadeWithHistory(vec![0])
    );

    // Deselect remaining item 0 transitions to Pending
    choice_upto.deselect(0).unwrap();
    assert_eq!(choice_upto.state(), &ChoiceState::Pending);

    // 4. Cardinality::AtLeast(2)
    let mut choice_atleast = MultiChoice::new("Languages", vec!["Common", "Elvish", "Dwarvish"])
        .with_cardinality(Cardinality::AtLeast(2));
    assert!(choice_atleast.confirm().is_err());
    choice_atleast.select(0).unwrap();
    assert!(choice_atleast.confirm().is_err());
    choice_atleast.select(1).unwrap();
    assert!(choice_atleast.confirm().is_ok());
    choice_atleast.select(2).unwrap();
    assert!(choice_atleast.confirm().is_ok());
}

#[test]
fn test_adversarial_multi_choice_error_resilience() {
    let mut choice = MultiChoice::new("Tools", vec!["Smith", "Brewer", "Mason"])
        .with_cardinality(Cardinality::Exact(2));

    // Out of bounds error
    assert!(matches!(
        choice.select(3),
        Err(SchematicError::IndexOutOfBounds {
            index: 3,
            total_options: 3,
            ..
        })
    ));
    assert!(choice.state().is_pending());

    // Stage 0
    choice.select(0).unwrap();

    // Duplicate selection error
    assert!(matches!(
        choice.select(0),
        Err(SchematicError::DuplicateSelection { index: 0, .. })
    ));
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(vec![0]));

    // Deselect non-existent
    assert!(matches!(
        choice.deselect(2),
        Err(SchematicError::NotSelected { index: 2, .. })
    ));
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(vec![0]));

    // Stage 1
    choice.select(1).unwrap();
    choice.confirm().unwrap();
    assert!(choice.is_made());

    // Lossless unmake
    choice.unmake().unwrap();
    assert!(!choice.is_made());
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(vec![0, 1]));

    // Re-confirm and resolve
    choice.confirm().unwrap();
    let res = choice.resolve().unwrap();
    assert_eq!(res, vec!["Smith", "Brewer"]);
}

#[test]
fn test_adversarial_multi_choice_resolution_order() {
    // Options declared in order [0, 1, 2]
    let mut choice = MultiChoice::new("Letters", vec!["Alpha", "Beta", "Gamma"])
        .with_cardinality(Cardinality::Exact(3));

    // Stage selections out of declaration order: 2, 0, 1
    choice.select(2).unwrap();
    choice.select(0).unwrap();
    choice.select(1).unwrap();
    choice.confirm().unwrap();

    // Resolve MUST preserve declaration order (Alpha, Beta, Gamma)
    let resolved = choice.resolve().unwrap();
    assert_eq!(resolved, vec!["Alpha", "Beta", "Gamma"]);
}

#[test]
fn test_adversarial_mutation_via_selected_values_mut() {
    // SingleChoice mutation
    let mut single = SingleChoice::new("Color", vec!["red".to_string(), "blue".to_string()]);
    single.select(1).unwrap();
    for val in single.selected_values_mut() {
        val.make_ascii_uppercase();
    }
    single.confirm().unwrap();
    assert_eq!(single.resolve_exact().unwrap(), "BLUE");

    // MultiChoice mutation
    let mut multi = MultiChoice::new(
        "Spells",
        vec![
            "spark".to_string(),
            "heal".to_string(),
            "shield".to_string(),
        ],
    )
    .with_cardinality(Cardinality::Exact(2));
    multi.select(0).unwrap();
    multi.select(2).unwrap();
    for val in multi.selected_values_mut() {
        *val = format!("{}_cast", val);
    }
    multi.confirm().unwrap();
    assert_eq!(
        multi.resolve().unwrap(),
        vec!["spark_cast".to_string(), "shield_cast".to_string()]
    );
}

#[test]
fn test_adversarial_visitor_edge_cases() {
    // 1. AutoSelectFirst on MultiChoice with Exact(3) but only 2 options available
    let mut impossible_multi =
        MultiChoice::new("Traits", vec!["A", "B"]).with_cardinality(Cardinality::Exact(3));
    let mut visitor = AutoSelectFirst;
    assert_eq!(
        visitor.visit_multi(&mut impossible_multi),
        VisitorFlow::Continue
    );
    // Cannot be satisfied, so must NOT be made!
    assert!(!impossible_multi.is_made());
    assert_eq!(impossible_multi.selected_indices(), &[0, 1]);

    // 2. AutoSelectFirst on empty SingleChoice
    let mut empty_single = SingleChoice::new("None", Vec::<String>::new());
    assert_eq!(
        visitor.visit_single(&mut empty_single),
        VisitorFlow::Continue
    );
    assert!(!empty_single.is_made());

    // 3. AutoSelectFirst does NOT overwrite an already made choice
    let mut made_single = SingleChoice::new("Class", vec!["Fighter", "Wizard"]);
    made_single.select(1).unwrap();
    made_single.confirm().unwrap();
    assert_eq!(
        visitor.visit_single(&mut made_single),
        VisitorFlow::Continue
    );
    assert_eq!(made_single.selected_index(), Some(1));

    // 4. UnmakeAll on already Pending choices does not fail
    let mut pending_single = SingleChoice::new("Skill", vec!["Stealth"]);
    let mut unmake_visitor = UnmakeAll;
    assert_eq!(
        unmake_visitor.visit_single(&mut pending_single),
        VisitorFlow::Continue
    );
    assert!(pending_single.state().is_pending());

    // 5. ConfirmAll on pending choice cannot confirm Exact(1)
    let mut confirm_visitor = ConfirmAll;
    assert_eq!(
        confirm_visitor.visit_single(&mut pending_single),
        VisitorFlow::Continue
    );
    assert!(pending_single.state().is_pending());
}

#[test]
fn test_adversarial_slot_resolutions() {
    // SingleSlot FixedUnresolved
    let slot_unres: SingleSlot<i32, i32> = Slot::FixedUnresolved(42);
    assert!(slot_unres.is_complete());
    assert_eq!(slot_unres.resolve_exact().unwrap(), 42);

    // SingleSlot FixedResolved
    let slot_res: SingleSlot<i32, i32> = Slot::FixedResolved(99);
    assert!(slot_res.is_complete());
    assert_eq!(slot_res.resolve_exact().unwrap(), 99);

    // MultiSlot FixedUnresolved
    let multi_unres: MultiSlot<String, String> = Slot::FixedUnresolved(vec!["alpha".to_string()]);
    assert!(multi_unres.is_complete());
    assert_eq!(multi_unres.resolve().unwrap(), vec!["alpha".to_string()]);

    // MultiSlot FixedResolved
    let multi_res: MultiSlot<String, String> = Slot::FixedResolved(vec!["beta".to_string()]);
    assert!(multi_res.is_complete());
    assert_eq!(multi_res.resolve().unwrap(), vec!["beta".to_string()]);

    // Accessors
    let choose_slot: SingleSlot<i32, i32> = Slot::Choose(SingleChoice::new("num", vec![1, 2]));
    assert!(choose_slot.as_choice().is_some());
    assert!(choose_slot.as_unresolved().is_none());
    assert!(choose_slot.as_resolved().is_none());
}

#[test]
fn test_adversarial_serde_unmade_with_history() {
    // 1. SingleChoice UnmadeWithHistory roundtrip
    let mut single = SingleChoice::new("Weapon", vec!["Sword".to_string(), "Bow".to_string()]);
    single.select(1).unwrap();
    assert_eq!(single.state(), &ChoiceState::UnmadeWithHistory(1));
    let json = serde_json::to_string(&single).unwrap();
    let deserialized: SingleChoice<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(single, deserialized);
    assert_eq!(deserialized.state(), &ChoiceState::UnmadeWithHistory(1));
    assert!(!deserialized.is_made());

    // 2. MultiChoice UnmadeWithHistory roundtrip
    let mut multi = MultiChoice::new(
        "Skills",
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
    )
    .with_cardinality(Cardinality::Exact(2));
    multi.select(0).unwrap();
    multi.select(2).unwrap();
    assert_eq!(multi.state(), &ChoiceState::UnmadeWithHistory(vec![0, 2]));
    let json = serde_json::to_string(&multi).unwrap();
    let deserialized: MultiChoice<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(multi, deserialized);
    assert_eq!(
        deserialized.state(),
        &ChoiceState::UnmadeWithHistory(vec![0, 2])
    );
    assert!(!deserialized.is_made());
}

#[test]
fn test_adversarial_slot_serde_untagged_ordering() {
    // FixedResolved vs FixedUnresolved when U == R
    let slot_res: SingleSlot<i32, i32> = Slot::FixedResolved(42);
    let json = serde_json::to_string(&slot_res).unwrap();
    let deserialized: SingleSlot<i32, i32> = serde_json::from_str(&json).unwrap();

    // With #[serde(untagged)], Slot tries variants in declaration order:
    // 1. Choose, 2. FixedUnresolved, 3. FixedResolved.
    // When U == R, FixedUnresolved matches 42, so deserialization yields FixedUnresolved(42)!
    // Notice that both FixedResolved and FixedUnresolved are complete and resolve to 42,
    // so resolution is preserved even if the enum variant is normalized.
    assert!(deserialized.is_complete());
    assert_eq!(deserialized.resolve_exact().unwrap(), 42);
}

#[test]
fn test_adversarial_multichoice_aliased_mut_hazard() {
    // When state contains duplicate indices (e.g. from untrusted JSON payload),
    // selected_values_mut automatically deduplicates indices to guarantee 100% disjoint mutable references!
    let json =
        r#"{"prompt":"spell","options":["spark"],"choose":{"Exact":2},"state":{"Made":[0,0]}}"#;
    let mut choice: MultiChoice<String> = serde_json::from_str(json).unwrap();
    let vals = choice.selected_values_mut();
    // Only 1 reference returned because index 0 was deduplicated!
    assert_eq!(vals.len(), 1);
    assert_eq!(vals[0], "spark");
}

#[test]
fn test_adversarial_single_choice_oob_state_hazard() {
    // When state contains an out-of-bounds index (e.g. from untrusted JSON deserialization),
    // resolve_exact returns a clean SchematicError::IndexOutOfBounds without panicking!
    let json = r#"{"prompt":"weapon","options":["sword"],"state":{"Made":999}}"#;
    let choice: SingleChoice<String> = serde_json::from_str(json).unwrap();
    let result = choice.resolve_exact();
    assert_eq!(
        result,
        Err(SchematicError::IndexOutOfBounds {
            prompt: "weapon".to_string(),
            index: 999,
            total_options: 1,
        })
    );
}

#[test]
fn test_adversarial_multi_choice_oob_state_silent_omission() {
    // When state contains an out-of-bounds index, resolve() detects the invalid index
    // and returns SchematicError::IndexOutOfBounds rather than silently omitting it!
    let json = r#"{"prompt":"skills","options":["athletics"],"choose":{"Exact":2},"state":{"Made":[0,999]}}"#;
    let choice: MultiChoice<String> = serde_json::from_str(json).unwrap();
    let res = choice.resolve();
    assert_eq!(
        res,
        Err(SchematicError::IndexOutOfBounds {
            prompt: "skills".to_string(),
            index: 999,
            total_options: 1,
        })
    );
}
