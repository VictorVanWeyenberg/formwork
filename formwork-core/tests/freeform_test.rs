use formwork_core::choice::{ChoiceState, Freeform};
use formwork_core::error::FormworkError;
use formwork_core::slot::{FreeformSlot, Slot};
use formwork_core::visitor::{AutoSelectFirst, ChoiceVisitor, ConfirmAll, UnmakeAll, VisitorFlow};
use serde::{Deserialize, Serialize};

#[test]
fn test_freeform_initial_pending_state() {
    let choice: Freeform<String> = Freeform::new("Enter character name");

    assert_eq!(choice.prompt(), "Enter character name");
    assert!(choice.is_pending());
    assert!(!choice.is_made());
    assert_eq!(choice.value(), None);
    assert_eq!(choice.state(), &ChoiceState::Pending);
}

#[test]
fn test_freeform_stage_confirm_resolve_lifecycle() {
    let mut choice: Freeform<String> = Freeform::new("Enter character name");

    // 1. Pending: cannot confirm, unmake, or resolve
    assert!(matches!(
        choice.confirm(),
        Err(FormworkError::MissingFreeformValue { ref prompt }) if prompt == "Enter character name"
    ));
    assert!(matches!(
        choice.unmake(),
        Err(FormworkError::CannotUnmakeNotMade { ref prompt }) if prompt == "Enter character name"
    ));
    assert!(matches!(
        choice.clone().resolve(),
        Err(FormworkError::WetChoice { ref prompt }) if prompt == "Enter character name"
    ));
    assert!(matches!(
        choice.clone().resolve_exact(),
        Err(FormworkError::WetChoice { ref prompt }) if prompt == "Enter character name"
    ));

    // 2. Stage candidate value
    choice.stage("Arthur Pendragon".to_string());
    assert!(!choice.is_pending());
    assert!(!choice.is_made(), "Staging must NOT mark choice as made");
    assert_eq!(choice.value(), Some(&"Arthur Pendragon".to_string()));
    assert_eq!(
        choice.state(),
        &ChoiceState::UnmadeWithHistory("Arthur Pendragon".to_string())
    );

    // Unconfirmed staged value still cannot resolve
    assert!(matches!(
        choice.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    // Staging again replaces the candidate in-place
    choice.stage("King Arthur".to_string());
    assert_eq!(choice.value(), Some(&"King Arthur".to_string()));
    assert!(!choice.is_made());

    // 3. Confirm transitions to Made
    assert!(choice.confirm().is_ok());
    assert!(choice.is_made());
    assert!(!choice.is_pending());
    assert_eq!(choice.value(), Some(&"King Arthur".to_string()));
    assert_eq!(
        choice.state(),
        &ChoiceState::Made("King Arthur".to_string())
    );

    // Confirm is idempotent
    assert!(choice.confirm().is_ok());
    assert!(choice.is_made());

    // 4. Resolve exact consumes the choice and yields the watertight value
    let resolved = choice.resolve_exact().unwrap();
    assert_eq!(resolved, "King Arthur");
}

#[test]
fn test_freeform_direct_set_and_unmake() {
    let mut choice: Freeform<i32> = Freeform::new("Enter score");

    // Set directly sets Made state
    choice.set(100);
    assert!(choice.is_made());
    assert_eq!(choice.value(), Some(&100));
    assert_eq!(choice.state(), &ChoiceState::Made(100));

    // Unmake preserves history in UnmadeWithHistory
    assert!(choice.unmake().is_ok());
    assert!(!choice.is_made());
    assert!(!choice.is_pending());
    assert_eq!(choice.value(), Some(&100));
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(100));

    // Unmake on already unmade is idempotent
    assert!(choice.unmake().is_ok());
    assert_eq!(choice.state(), &ChoiceState::UnmadeWithHistory(100));

    // Cannot resolve while unmade
    assert!(choice.clone().resolve().is_err());

    // Re-confirm restores Made state
    assert!(choice.confirm().is_ok());
    assert!(choice.is_made());
    assert_eq!(choice.resolve().unwrap(), 100);
}

#[test]
fn test_freeform_with_default_seeding() {
    let mut choice: Freeform<String> =
        Freeform::new("Enter hostname").with_default("localhost".to_string());

    // Default value is unmade with history (not yet confirmed)
    assert!(!choice.is_pending());
    assert!(!choice.is_made());
    assert_eq!(choice.value(), Some(&"localhost".to_string()));
    assert_eq!(
        choice.state(),
        &ChoiceState::UnmadeWithHistory("localhost".to_string())
    );

    // Resolving without confirming fails
    assert!(matches!(
        choice.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    // Confirming confirms the default value directly
    assert!(choice.confirm().is_ok());
    assert!(choice.is_made());
    assert_eq!(choice.resolve().unwrap(), "localhost");
}

#[test]
fn test_freeform_with_default_overridden_by_stage_or_set() {
    // Override via stage then confirm
    let mut choice1: Freeform<String> =
        Freeform::new("Hostname").with_default("127.0.0.1".to_string());
    choice1.stage("example.com".to_string());
    assert_eq!(choice1.value(), Some(&"example.com".to_string()));
    choice1.confirm().unwrap();
    assert_eq!(choice1.resolve().unwrap(), "example.com");

    // Override via set
    let mut choice2: Freeform<String> =
        Freeform::new("Hostname").with_default("127.0.0.1".to_string());
    choice2.set("api.example.com".to_string());
    assert!(choice2.is_made());
    assert_eq!(choice2.resolve().unwrap(), "api.example.com");
}

#[test]
fn test_freeform_value_mut_in_place_mutation() {
    let mut choice: Freeform<Vec<u8>> = Freeform::new("Payload");

    // Pending: value_mut is None
    assert!(choice.value_mut().is_none());

    // Staged: mutate in-place
    choice.stage(vec![1, 2, 3]);
    if let Some(payload) = choice.value_mut() {
        payload.push(4);
        payload.push(5);
    }
    assert_eq!(choice.value(), Some(&vec![1, 2, 3, 4, 5]));
    assert!(!choice.is_made());

    choice.confirm().unwrap();
    assert!(choice.is_made());

    // Made: mutate in-place
    if let Some(payload) = choice.value_mut() {
        payload.clear();
        payload.push(42);
    }
    assert_eq!(choice.resolve().unwrap(), vec![42]);
}

#[test]
fn test_freeform_slot_methods_and_resolution() {
    // 1. FixedWet slot
    let wet_slot: FreeformSlot<String, String> = Slot::FixedWet("fixed_wet".to_string());
    assert!(!wet_slot.is_pending());
    assert!(!wet_slot.is_made());
    assert!(wet_slot.is_complete());
    assert_eq!(wet_slot.resolve().unwrap(), "fixed_wet");

    // 2. FixedDry slot
    let dry_slot: FreeformSlot<String, String> = Slot::FixedDry("fixed_dry".to_string());
    assert!(!dry_slot.is_pending());
    assert!(!dry_slot.is_made());
    assert!(dry_slot.is_complete());
    assert_eq!(dry_slot.resolve().unwrap(), "fixed_dry");

    // 3. Choose slot: Pending
    let pending_slot: FreeformSlot<String, String> = Slot::Choose(Freeform::new("Prompt"));
    assert!(pending_slot.is_pending());
    assert!(!pending_slot.is_made());
    assert!(!pending_slot.is_complete());
    assert!(pending_slot.resolve().is_err());

    // 4. Choose slot: Staged
    let mut staged_choice = Freeform::new("Prompt");
    staged_choice.stage("Staged".to_string());
    assert!(!staged_choice.is_pending()); // Choice itself is UnmadeWithHistory
    let staged_slot: FreeformSlot<String, String> = Slot::Choose(staged_choice);
    // FreeformSlot::is_pending() returns true if the choice is unconfirmed (!c.is_made())
    assert!(staged_slot.is_pending());
    assert!(!staged_slot.is_made());
    assert!(!staged_slot.is_complete());
    assert!(staged_slot.resolve().is_err());

    // 5. Choose slot: Confirmed / Made
    let mut made_choice = Freeform::new("Prompt");
    made_choice.set("Confirmed".to_string());
    let made_slot: FreeformSlot<String, String> = Slot::Choose(made_choice);
    assert!(!made_slot.is_pending());
    assert!(made_slot.is_made());
    assert!(made_slot.is_complete());
    assert_eq!(made_slot.resolve().unwrap(), "Confirmed");
}

#[test]
fn test_freeform_serde_json_roundtrip_all_states() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
    struct ComplexData {
        name: String,
        count: usize,
    }

    // State 1: Pending
    let pending: Freeform<ComplexData> = Freeform::new("Enter complex data");
    let pending_json = serde_json::to_string(&pending).unwrap();
    // Default state should be omitted due to skip_serializing_if = "is_default_state"
    assert!(!pending_json.contains("state"));
    let deserialized_pending: Freeform<ComplexData> =
        serde_json::from_str(&pending_json).unwrap();
    assert_eq!(pending, deserialized_pending);
    assert!(deserialized_pending.is_pending());

    // State 2: UnmadeWithHistory
    let mut unmade: Freeform<ComplexData> = Freeform::new("Enter complex data");
    unmade.stage(ComplexData {
        name: "Test".to_string(),
        count: 7,
    });
    let unmade_json = serde_json::to_string(&unmade).unwrap();
    assert!(unmade_json.contains("UnmadeWithHistory"));
    let deserialized_unmade: Freeform<ComplexData> =
        serde_json::from_str(&unmade_json).unwrap();
    assert_eq!(unmade, deserialized_unmade);
    assert!(!deserialized_unmade.is_made());
    assert_eq!(
        deserialized_unmade.value(),
        Some(&ComplexData {
            name: "Test".to_string(),
            count: 7
        })
    );

    // State 3: Made
    let mut made: Freeform<ComplexData> = Freeform::new("Enter complex data");
    made.set(ComplexData {
        name: "Confirmed".to_string(),
        count: 42,
    });
    let made_json = serde_json::to_string(&made).unwrap();
    assert!(made_json.contains("Made"));
    let deserialized_made: Freeform<ComplexData> = serde_json::from_str(&made_json).unwrap();
    assert_eq!(made, deserialized_made);
    assert!(deserialized_made.is_made());
    assert_eq!(
        deserialized_made.resolve().unwrap(),
        ComplexData {
            name: "Confirmed".to_string(),
            count: 42
        }
    );
}

#[test]
fn test_freeform_visitor_unmake_all_and_confirm_all() {
    let mut choice: Freeform<String> = Freeform::new("Name");
    choice.set("Alice".to_string());
    assert!(choice.is_made());

    // UnmakeAll
    let mut unmake_visitor = UnmakeAll;
    assert_eq!(
        unmake_visitor.visit_freeform(&mut choice),
        VisitorFlow::Continue
    );
    assert!(!choice.is_made());
    assert_eq!(choice.value(), Some(&"Alice".to_string()));

    // ConfirmAll
    let mut confirm_visitor = ConfirmAll;
    assert_eq!(
        confirm_visitor.visit_freeform(&mut choice),
        VisitorFlow::Continue
    );
    assert!(choice.is_made());
    assert_eq!(choice.resolve().unwrap(), "Alice");

    // AutoSelectFirst has default no-op for Freeform
    let mut auto_visitor = AutoSelectFirst;
    let mut pending_choice: Freeform<String> = Freeform::new("Pending");
    assert_eq!(
        auto_visitor.visit_freeform(&mut pending_choice),
        VisitorFlow::Continue
    );
    assert!(pending_choice.is_pending());
}

#[test]
fn test_freeform_adversarial_reversals_and_transitions() {
    let mut choice: Freeform<String> = Freeform::new("Adversarial");

    // Unmake on pending must fail
    assert!(choice.unmake().is_err());

    // Stage "1" -> unmake succeeds and keeps "1" in history
    choice.stage("1".to_string());
    assert!(choice.unmake().is_ok());
    assert_eq!(choice.value(), Some(&"1".to_string()));

    // Unmake again on UnmadeWithHistory is idempotent
    assert!(choice.unmake().is_ok());
    assert_eq!(choice.value(), Some(&"1".to_string()));

    // Set overrides unmade history directly
    choice.set("2".to_string());
    assert!(choice.is_made());
    assert_eq!(choice.value(), Some(&"2".to_string()));

    // Stage over Made reverts Made to UnmadeWithHistory with new value
    choice.stage("3".to_string());
    assert!(!choice.is_made());
    assert_eq!(choice.value(), Some(&"3".to_string()));

    // Confirm transitions to Made
    assert!(choice.confirm().is_ok());
    assert!(choice.is_made());

    // Mutate made value in place
    *choice.value_mut().unwrap() = "4".to_string();
    assert_eq!(choice.value(), Some(&"4".to_string()));

    // Unmake -> Confirm cycle preserves mutated value
    assert!(choice.unmake().is_ok());
    assert!(!choice.is_made());
    assert_eq!(choice.value(), Some(&"4".to_string()));
    assert!(choice.confirm().is_ok());
    assert_eq!(choice.resolve().unwrap(), "4");
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct NonDefaultType {
    pub custom_id: u64,
}

#[test]
fn test_freeform_serde_without_default_bound() {
    let json = r#"{"prompt":"Non-default entity"}"#;
    let choice: Freeform<NonDefaultType> = serde_json::from_str(json).unwrap();
    assert_eq!(choice.prompt(), "Non-default entity");
    assert!(choice.is_pending());

    let mut choice = choice;
    choice.set(NonDefaultType { custom_id: 42 });
    let serialized = serde_json::to_string(&choice).unwrap();
    let deserialized: Freeform<NonDefaultType> = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.is_made());
    assert_eq!(
        deserialized.value(),
        Some(&NonDefaultType { custom_id: 42 })
    );
}
