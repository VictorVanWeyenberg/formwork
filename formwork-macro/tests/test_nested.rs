use formwork::Formwork;
use formwork::slot::SingleSlot;
use formwork_macro::formwork;

#[formwork]
struct Leaf {
    #[slot]
    name: String,
}

#[formwork]
struct NestedRoot {
    #[slot(cascade)]
    leaf: Leaf,
    #[slot(collection, cascade)]
    leafs: Vec<Leaf>,
}

#[test]
fn test_nested_formworks() {
    let root: NestedRootFormwork = NestedRoot {
        leaf: SingleSlot::FixedWet(LeafFormwork {
            name: SingleSlot::FixedWet("Leaf1".to_string()),
        }),
        leafs: vec![SingleSlot::FixedWet(LeafFormwork {
            name: SingleSlot::FixedWet("Leaf2".to_string()),
        })],
    };

    assert!(root.is_complete());
    let resolved = root.resolve().unwrap();
    assert_eq!(resolved.leaf.name, "Leaf1");
}
