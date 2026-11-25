#[test]
fn test_enum_compile_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/enum_*.rs");
    t.compile_fail("tests/compile_fail/union_rejected.rs");
}
