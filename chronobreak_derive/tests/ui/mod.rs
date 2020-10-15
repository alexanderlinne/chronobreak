extern crate trybuild;

fn main() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fails/*.rs");
    t.pass("tests/ui/successes/*.rs");
}
