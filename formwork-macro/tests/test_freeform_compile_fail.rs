#[test]
fn test_freeform_compile_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/freeform_*.rs");
}
