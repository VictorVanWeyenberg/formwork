use schematic_core::choice::SingleChoice;
use schematic_core::slot::{MultiSlot, SingleSlot, Slot};

#[test]
fn test_single_slot_fixed_resolved_resolves_cleanly() {
    let slot: SingleSlot<String, String> = Slot::FixedResolved("already resolved".to_string());

    assert!(slot.is_complete());
    assert!(!slot.is_pending());
    assert!(!slot.is_made());
    // FixedResolved must resolve to Ok(val) when U == R!
    let res = slot
        .resolve_exact()
        .expect("FixedResolved must resolve successfully");
    assert_eq!(res, "already resolved");
}

#[test]
fn test_multi_slot_fixed_resolved_resolves_cleanly() {
    let slot: MultiSlot<String, String> = Slot::FixedResolved(vec!["item_a".to_string()]);

    assert!(slot.is_complete());
    let res = slot
        .resolve()
        .expect("FixedResolved must resolve successfully");
    assert_eq!(res, vec!["item_a".to_string()]);
}

#[test]
fn test_multi_slot_fixed_unresolved_resolves_cleanly() {
    let slot: MultiSlot<String, String> = Slot::FixedUnresolved(vec!["item_b".to_string()]);

    assert!(slot.is_complete());
    let res = slot
        .resolve()
        .expect("FixedUnresolved must resolve successfully");
    assert_eq!(res, vec!["item_b".to_string()]);
}

#[test]
fn test_multi_slot_from_conversions() {
    let choice = schematic_core::choice::MultiChoice::new("Pick", vec!["A".to_string()]);
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
