use formwork_core::choice::{Cardinality, ChoiceState, MultiChoice, SingleChoice};
use formwork_core::error::FormworkError;

#[test]
fn test_single_choice_universal_calling_contract() {
    let mut choice: SingleChoice<&str> = SingleChoice::new("Pick weapon", vec!["Sword", "Bow"]);

    // 1. Initial state is Pending
    assert_eq!(choice.state(), &ChoiceState::Pending);
    assert!(!choice.is_made());
    assert_eq!(choice.selected_index(), None);
    assert!(choice.selected_indices().is_empty());
    assert!(choice.clone().resolve_exact().is_err());

    // 2. Calling select stages the candidate, but DOES NOT transition to Made!
    choice.select(1).unwrap();
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(1));
    assert_eq!(choice.selected_index(), Some(1));
    assert_eq!(choice.selected_indices(), &[1]);
    assert!(!choice.is_made(), "select must NEVER auto-make a choice!");
    assert!(
        choice.clone().resolve_exact().is_err(),
        "unconfirmed choice cannot resolve!"
    );

    // 3. In-place replacement: selecting another index replaces the staged selection
    choice.select(0).unwrap();
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(0));
    assert_eq!(choice.selected_index(), Some(0));
    assert!(!choice.is_made());

    // 4. Calling confirm transitions state to Made
    choice.confirm().unwrap();
    assert!(choice.is_made());
    assert_eq!(choice.state(), &ChoiceState::Made(0));

    // 5. Calling resolve_exact returns watertight value
    let resolved = choice.resolve_exact().unwrap();
    assert_eq!(resolved, "Sword");
}

#[test]
fn test_single_choice_zero_heap_allocation_size() {
    let choice: SingleChoice<&str> = SingleChoice::new("Class", vec!["Fighter", "Wizard"]);
    // ChoiceState<usize> is 16 bytes on 64-bit systems (tag + padding + usize).
    // It is Copy and stack-only (zero heap allocation for state).
    assert!(std::mem::size_of_val(&choice.state) <= 16);
}

#[test]
fn test_single_choice_deselect_and_unmake() {
    let mut choice: SingleChoice<&str> = SingleChoice::new("Color", vec!["Red", "Blue"]);
    choice.select(1).unwrap();
    choice.confirm().unwrap();
    assert!(choice.is_made());

    // Unmake preserves history in UnmadeWithHistory(1)
    choice.unmake().unwrap();
    assert!(!choice.is_made());
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(1));

    // Confirm restores Made
    choice.confirm().unwrap();
    assert!(choice.is_made());

    // Deselect clears state to Pending
    choice.deselect(1).unwrap();
    assert_eq!(choice.state(), &ChoiceState::Pending);
    assert!(!choice.is_made());
}

#[test]
fn test_multi_choice_universal_calling_contract() {
    let mut choice: MultiChoice<&str> = MultiChoice::new(
        "Skills",
        vec!["Athletics", "Stealth", "Arcana", "Perception"],
    )
    .with_cardinality(Cardinality::Exact(2));

    // 1. Initial Pending
    assert_eq!(choice.state(), &ChoiceState::Pending);
    assert!(!choice.is_made());

    // 2. Select 1 item: staged, not made
    choice.select(0).unwrap();
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(vec![0]));
    assert!(!choice.is_made());

    // Confirming too early fails cardinality constraint
    assert!(choice.confirm().is_err());

    // 3. Select 2nd item: exact cardinality reached, BUT STILL NOT MADE!
    choice.select(2).unwrap();
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(vec![0, 2]));
    assert!(
        !choice.is_made(),
        "Even when exact cardinality is met, choice must NOT auto-make without confirm!"
    );
    assert!(choice.clone().resolve().is_err());

    // 4. Calling confirm transitions to Made
    choice.confirm().unwrap();
    assert!(choice.is_made());
    assert_eq!(choice.state(), &ChoiceState::Made(vec![0, 2]));

    // 5. Resolve returns options in declared order
    let resolved = choice.resolve().unwrap();
    assert_eq!(resolved, vec!["Athletics", "Arcana"]);
}

#[test]
fn test_multi_choice_lossless_unmake_confirm_roundtrip() {
    let mut choice: MultiChoice<&str> = MultiChoice::new("Tools", vec!["Smith", "Brewer", "Mason"])
        .with_cardinality(Cardinality::Exact(2));

    choice.select(0).unwrap();
    choice.select(1).unwrap();
    choice.confirm().unwrap();
    assert!(choice.is_made());

    // unmake MUST preserve 100% of selection history (NO pop!)
    choice.unmake().unwrap();
    assert!(!choice.is_made());
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(vec![0, 1]));

    // confirm restores Made without error
    choice.confirm().unwrap();
    assert!(choice.is_made());
    let resolved = choice.resolve().unwrap();
    assert_eq!(resolved, vec!["Smith", "Brewer"]);
}

#[test]
fn test_multi_choice_deselect_demotes_made_choice() {
    let mut choice: MultiChoice<&str> = MultiChoice::new("Languages", vec!["Common", "Dwarvish", "Elvish"])
        .with_cardinality(Cardinality::Exact(2));

    choice.select(0).unwrap();
    choice.select(1).unwrap();
    choice.confirm().unwrap();
    assert!(choice.is_made());

    // Deselect index 1 from a Made choice:
    // MUST demote back to UnmadeWithHistory([0])
    choice.deselect(1).unwrap();
    assert!(!choice.is_made());
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(vec![0]));

    // Deselect index 0: becomes Pending
    choice.deselect(0).unwrap();
    assert_eq!(choice.state(), &ChoiceState::Pending);
}

#[test]
fn test_multi_choice_duplicate_selection_guard() {
    let mut choice: MultiChoice<&str> = MultiChoice::new("Spells", vec!["Shield", "Magic Missile"]);
    choice.select(0).unwrap();
    let err = choice.select(0).unwrap_err();
    match err {
        FormworkError::DuplicateSelection { prompt, index } => {
            assert_eq!(prompt, "Spells");
            assert_eq!(index, 0);
        }
        other => panic!("Expected DuplicateSelection, got {:?}", other),
    }
}

#[test]
fn test_serde_json_roundtrip_choice_states() {
    // 1. SingleChoice Pending
    let single_pending = SingleChoice::new("Color", vec!["Red".to_string(), "Blue".to_string()]);
    let json = serde_json::to_string(&single_pending).unwrap();
    let des: SingleChoice<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(single_pending, des);

    // 2. SingleChoice Made
    let mut single_made = SingleChoice::new("Color", vec!["Red".to_string(), "Blue".to_string()]);
    single_made.select(1).unwrap();
    single_made.confirm().unwrap();
    let json = serde_json::to_string(&single_made).unwrap();
    let des: SingleChoice<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(single_made, des);

    // 3. MultiChoice Made
    let mut multi_made = MultiChoice::new(
        "Tags",
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
    )
    .with_cardinality(Cardinality::UpTo(2));
    multi_made.select(0).unwrap();
    multi_made.select(2).unwrap();
    multi_made.confirm().unwrap();
    let json = serde_json::to_string(&multi_made).unwrap();
    let des: MultiChoice<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(multi_made, des);
}
