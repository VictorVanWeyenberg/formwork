use schematic::{SingleChoice, schematic};

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parent {
    first_child: Child,
    #[slot(collection)]
    second_child: Vec<Child>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Child;

pub fn parent() -> ParentSchematic {
    ParentSchematic::builder()
        .first_child(Child)
        .with_second_child(Child)
        .with_second_child(Child)
        .with_second_child(SingleChoice::new("child", vec![Child, Child, Child]))
        .second_child(vec![SingleChoice::new("child", vec![Child, Child, Child])])
        .build()
}
