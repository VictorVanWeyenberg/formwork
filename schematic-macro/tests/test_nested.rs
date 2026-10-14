use schematic::Schematic;
use schematic::slot::SingleSlot;
use schematic_macro::schematic;

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
        leaf: SingleSlot::FixedUnresolved(LeafSchematic {
            name: SingleSlot::FixedUnresolved("Leaf1".to_string()),
        }),
        leafs: vec![SingleSlot::FixedUnresolved(LeafSchematic {
            name: SingleSlot::FixedUnresolved("Leaf2".to_string()),
        })],
    };

    assert!(root.is_complete());
    let resolved = root.resolve().unwrap();
    assert_eq!(resolved.leaf.name, "Leaf1");
}
