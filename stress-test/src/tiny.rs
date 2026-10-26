use schematic::{schematic, SingleChoice};

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parent {
    pub first_child: Child,
    #[slot]
    pub second_child: Vec<Child>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Child;

pub fn parent() -> ParentSchematic {
    ParentSchematic::builder()
        .first_child(Child)
        .second_child(SingleChoice::new(
            "child",
            vec![vec![Child], vec![Child], vec![Child]],
        ))
        .build()
}

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parent2 {
    pub first_child: Child,
    #[slot]
    pub second_child: Child,
}

pub fn parent2() -> Parent2Schematic {
    Parent2Schematic::builder()
        .first_child(Child)
        .second_child(SingleChoice::new("child", vec![Child, Child, Child]))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use schematic::{visitor::AutoSelectFirst, Schematic};

    #[test]
    fn test_parent_vec_slot_preservation() {
        let mut p = parent();
        assert!(!p.is_complete());

        p.walk_choices(&mut AutoSelectFirst);
        assert!(p.is_complete());

        let resolved: Parent = p.resolve().expect("parent must resolve");
        assert_eq!(resolved.first_child, Child);
        assert_eq!(resolved.second_child, vec![Child]);
    }

    #[test]
    fn test_parent2_scalar_slot_preservation() {
        let mut p2 = parent2();
        assert!(!p2.is_complete());

        p2.walk_choices(&mut AutoSelectFirst);
        assert!(p2.is_complete());

        let resolved: Parent2 = p2.resolve().expect("parent2 must resolve");
        assert_eq!(resolved.first_child, Child);
        assert_eq!(resolved.second_child, Child);
    }
}
