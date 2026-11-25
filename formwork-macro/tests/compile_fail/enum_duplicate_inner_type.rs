use formwork::formwork;

pub mod sub {
    use formwork::formwork;

    #[formwork]
    pub struct Inner {
        #[slot]
        pub name: String,
    }
}

#[formwork]
pub struct Plain {
    #[slot]
    pub x: i32,
}

#[formwork]
pub enum PlainDup {
    A(Plain),
    B(Plain),
}

#[formwork]
pub enum WhitespaceQualifiedDup {
    A(sub::Inner),
    B(sub :: Inner),
}

#[formwork]
pub enum CrateQualifiedDup {
    A(crate::sub::Inner),
    B(crate :: sub :: Inner),
}

fn main() {}
