use spike_derive::blueprint;

#[blueprint(derive(Clone))]
#[derive()]
struct Test {
    #[slots]
    a: Vec<String>,
}