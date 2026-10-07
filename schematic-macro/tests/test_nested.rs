use schematic_macro::schematic;
use schematic::Schematic;

#[schematic]
struct Leaf {
    #[slot]
    name: String,
}

#[schematic]
struct NestedRoot {
    #[slot(cascade)]
    leaf: Leaf,
    #[slot(collection, cascade)]
    leafs: Vec<Leaf>,
}

#[test]
fn test_nested_schematics() {
    let root: NestedRootSchematic = NestedRoot {
        leaf: schematic::Slot::Fixed(Leaf {
            name: schematic::Slot::Fixed("Leaf1".to_string()),
        }),
        leafs: vec![
            schematic::Slot::Fixed(Leaf {
                name: schematic::Slot::Fixed("Leaf2".to_string()),
            }),
        ],
    };
    
    assert!(root.is_complete());
    
    // root.resolve().unwrap(); // Note: currently triggers type error based on old comment.
}
