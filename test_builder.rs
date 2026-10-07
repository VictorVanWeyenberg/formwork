use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(mutators(
    pub fn do_thing(&mut self, x: i32) where i32: Into<T> {
        self.val = x.into();
    }
))]
struct Foo<T> {
    val: T,
}
fn main() {}
