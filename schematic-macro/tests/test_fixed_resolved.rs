use schematic::slot::SingleSlot;

#[test]
fn test_fixed_resolved_error() {
    let slot: SingleSlot<String, String> =
        schematic::slot::Slot::FixedResolved("already resolved".to_string());

    assert!(slot.is_complete());
    let res = slot.resolve_exact();
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "Slot already resolved");
}
