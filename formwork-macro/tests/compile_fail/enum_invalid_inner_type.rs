use formwork::formwork;

#[formwork]
pub struct Inner {
    #[slot]
    pub name: String,
}

#[formwork]
pub enum TupleInner {
    A((Inner,)),
}

#[formwork]
pub enum RefInner {
    A(&'static Inner),
}

#[formwork]
pub enum ArrayInner {
    A([Inner; 1]),
}

#[formwork]
pub enum OptionInner {
    A(Option<Inner>),
}

#[formwork]
pub enum VecInner {
    A(Vec<Inner>),
}

#[formwork]
pub enum AlreadyParameterizedInner {
    A(Inner<InnerWet>),
}

#[formwork]
pub enum MidSegmentGenericInner {
    A(mod_a::Foo<i32>::Bar),
}

pub struct NotAFormwork;

#[formwork]
pub enum NonFormworkInner {
    A(NotAFormwork),
}

fn main() {}
