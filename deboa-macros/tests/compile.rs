#[test]
#[cfg(not(tarpaulin))]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/resource-pass-1.rs");
    t.compile_fail("tests/ui/resource-compile-fail-1.rs");
    t.compile_fail("tests/ui/resource-compile-fail-2.rs");
    t.compile_fail("tests/ui/resource-compile-fail-3.rs");
    t.compile_fail("tests/ui/resource-compile-fail-4.rs");
}
