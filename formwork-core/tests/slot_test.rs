use formwork_core::choice::{MultiChoice, SingleChoice};
use formwork_core::error::FormworkError;
use formwork_core::slot::{MultiSlot, SingleSlot, Slot};

#[test]
fn test_single_slot_fixed_dry_resolves_cleanly() {
    let slot: SingleSlot<String, String> = Slot::FixedDry("already dry".to_string());

    assert!(slot.is_complete());
    assert!(!slot.is_pending());
    assert!(!slot.is_made());
    assert_eq!(slot.as_dry(), Some(&"already dry".to_string()));
    assert_eq!(slot.as_wet(), None);
    assert_eq!(slot.as_choice(), None);

    // FixedDry must resolve to Ok(val) when W == D!
    let res = slot
        .resolve_exact()
        .expect("FixedDry must resolve successfully");
    assert_eq!(res, "already dry");
}

#[test]
fn test_single_slot_fixed_wet_resolves_cleanly() {
    let slot: SingleSlot<String, String> = Slot::FixedWet("wet val".to_string());

    assert!(slot.is_complete());
    assert!(!slot.is_pending());
    assert!(!slot.is_made());
    assert_eq!(slot.as_wet(), Some(&"wet val".to_string()));
    assert_eq!(slot.as_dry(), None);
    assert_eq!(slot.as_choice(), None);

    let res = slot
        .resolve_exact()
        .expect("FixedWet must resolve successfully");
    assert_eq!(res, "wet val");
}

#[test]
fn test_multi_slot_fixed_dry_resolves_cleanly() {
    let slot: MultiSlot<String, String> = Slot::FixedDry(vec!["item_a".to_string()]);

    assert!(slot.is_complete());
    assert_eq!(slot.as_dry(), Some(&vec!["item_a".to_string()]));
    assert_eq!(slot.as_wet(), None);
    let res = slot
        .resolve()
        .expect("FixedDry must resolve successfully");
    assert_eq!(res, vec!["item_a".to_string()]);
}

#[test]
fn test_multi_slot_fixed_wet_resolves_cleanly() {
    let slot: MultiSlot<String, String> = Slot::FixedWet(vec!["item_b".to_string()]);

    assert!(slot.is_complete());
    assert_eq!(slot.as_wet(), Some(&vec!["item_b".to_string()]));
    assert_eq!(slot.as_dry(), None);
    let res = slot
        .resolve()
        .expect("FixedWet must resolve successfully");
    assert_eq!(res, vec!["item_b".to_string()]);
}

#[test]
fn test_single_slot_from_conversions() {
    let choice = SingleChoice::new("Single", vec!["X".to_string()]);
    let slot: SingleSlot<String, String> = choice.into();
    assert!(!slot.is_complete());
    assert!(slot.is_pending());
    assert!(slot.as_choice().is_some());

    let fixed: SingleSlot<String, String> = "Hello".to_string().into();
    assert!(fixed.is_complete());
    assert_eq!(fixed.as_wet(), Some(&"Hello".to_string()));
    assert_eq!(fixed.resolve_exact().unwrap(), "Hello");
}

#[test]
fn test_multi_slot_from_conversions() {
    let choice = MultiChoice::new("Pick", vec!["A".to_string()]);
    let slot: MultiSlot<String, String> = choice.into();
    assert!(!slot.is_complete());
    assert!(slot.is_pending());

    let fixed: MultiSlot<String, String> = vec!["A".to_string(), "B".to_string()].into();
    assert!(fixed.is_complete());
    assert_eq!(
        fixed.resolve().unwrap(),
        vec!["A".to_string(), "B".to_string()]
    );
}

#[test]
fn test_single_slot_choose_lifecycle() {
    let mut choice = SingleChoice::new("Class", vec!["Warrior".to_string()]);
    let slot: SingleSlot<String, String> = Slot::Choose(choice.clone());

    assert!(!slot.is_complete());
    assert!(slot.is_pending());

    // Stage selection in choice
    choice.select(0).unwrap();
    let staged_slot: SingleSlot<String, String> = Slot::Choose(choice.clone());
    assert!(
        !staged_slot.is_complete(),
        "Staged choice in slot is NOT complete until confirmed"
    );

    // Confirm choice
    choice.confirm().unwrap();
    let confirmed_slot: SingleSlot<String, String> = Slot::Choose(choice);
    assert!(confirmed_slot.is_complete());
    assert!(confirmed_slot.is_made());
    assert_eq!(confirmed_slot.resolve_exact().unwrap(), "Warrior");
}

#[test]
fn test_slot_unconfirmed_resolve_returns_wet_choice_error() {
    let unmade_single: SingleSlot<String, String> =
        Slot::Choose(SingleChoice::new("Weapon Choice", vec!["Axe".to_string()]));
    match unmade_single.resolve_exact() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Weapon Choice"),
        other => panic!("Expected WetChoice, got {:?}", other),
    }

    let unmade_multi: MultiSlot<String, String> =
        Slot::Choose(MultiChoice::new("Spells Choice", vec!["Fire".to_string()]));
    match unmade_multi.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Spells Choice"),
        other => panic!("Expected WetChoice, got {:?}", other),
    }
}
