#[test]
fn ui_test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/01_collection_and_pick.rs");
    t.compile_fail("tests/ui/02_unsupported_slot_arg.rs");
    t.compile_fail("tests/ui/03_invalid_traversal.rs");
    t.compile_fail("tests/ui/04_enum_target.rs");
}
