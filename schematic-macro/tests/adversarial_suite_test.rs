use schematic::choice::{Cardinality, MultiChoice, SingleChoice};
use schematic::error::SchematicError;
use schematic::slot::{MultiSlot, Slot};
use schematic::visitor::AutoSelectFirst;
use schematic::{Schematic, schematic};

// ============================================================================
// Model Hierarchy: Combining all 6 slot kinds in a multi-tier tree
// ============================================================================

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct AdversarialLeaf {
    pub id: u32,
    #[slot]
    pub leaf_val: String,
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct AdversarialBranchBfs {
    #[slot]
    pub branch_name: String,
    #[slot(collection, cascade)]
    pub coll_children: Vec<AdversarialLeaf>,
    #[slot(pick, cascade)]
    pub pick_children: Vec<AdversarialLeaf>,
}

#[schematic(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct AdversarialBranchDfs {
    #[slot]
    pub branch_name: String,
    #[slot(collection, cascade)]
    pub coll_children: Vec<AdversarialLeaf>,
    #[slot(pick, cascade)]
    pub pick_children: Vec<AdversarialLeaf>,
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct AdversarialRootBfs {
    pub code: usize,
    #[slot]
    pub single_leaf: String,
    #[slot(collection)]
    pub coll_leaf: Vec<String>,
    #[slot(pick)]
    pub pick_leaf: Vec<String>,
    #[slot(cascade)]
    pub single_cascade: AdversarialLeaf,
    #[slot(collection, cascade)]
    pub coll_cascade: Vec<AdversarialBranchBfs>,
    #[slot(pick, cascade)]
    pub pick_cascade: Vec<AdversarialBranchBfs>,
}

#[schematic(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct AdversarialRootDfs {
    pub code: usize,
    #[slot]
    pub single_leaf: String,
    #[slot(collection)]
    pub coll_leaf: Vec<String>,
    #[slot(pick)]
    pub pick_leaf: Vec<String>,
    #[slot(cascade)]
    pub single_cascade: AdversarialLeaf,
    #[slot(collection, cascade)]
    pub coll_cascade: Vec<AdversarialBranchDfs>,
    #[slot(pick, cascade)]
    pub pick_cascade: Vec<AdversarialBranchDfs>,
}

// ============================================================================
// 1. Direct Setter Ergonomics for #[slot(pick)]
// ============================================================================

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct PickErgonomicsHost {
    #[slot(pick)]
    pub str_items: Vec<String>,
    #[slot(pick)]
    pub num_items: Vec<u64>,
}

#[test]
fn test_pick_direct_setter_variations() {
    // 1. Array [__V; __N]
    let bp_slice = PickErgonomicsHost::builder()
        .str_items(["alpha", "beta"])
        .num_items(vec![10u64, 20u64])
        .build();
    assert!(bp_slice.is_complete());
    let res_slice = bp_slice.resolve().unwrap();
    assert_eq!(res_slice.str_items, vec!["alpha", "beta"]);
    assert_eq!(res_slice.num_items, vec![10, 20]);

    // 2. Vec<&str>
    let bp_vec_str = PickErgonomicsHost::builder()
        .str_items(vec!["gamma", "delta"])
        .num_items(vec![30u64])
        .build();
    assert!(bp_vec_str.is_complete());
    let res_vec_str = bp_vec_str.resolve().unwrap();
    assert_eq!(res_vec_str.str_items, vec!["gamma", "delta"]);

    // 3. MultiChoice directly passed to .<field>()
    let choice_str = MultiChoice::new("P1", vec!["x".to_string(), "y".to_string()])
        .with_cardinality(Cardinality::Exact(1));
    let mut bp_choice = PickErgonomicsHost::builder()
        .str_items(choice_str)
        .num_items(vec![99u64])
        .build();
    assert!(!bp_choice.is_complete());
    bp_choice
        .str_items
        .as_choice_mut()
        .unwrap()
        .select(1)
        .unwrap();
    bp_choice
        .str_items
        .as_choice_mut()
        .unwrap()
        .confirm()
        .unwrap();
    assert!(bp_choice.is_complete());
    let res_choice = bp_choice.resolve().unwrap();
    assert_eq!(res_choice.str_items, vec!["y"]);

    // 4. MultiSlot directly passed to .<field>()
    let slot_raw: MultiSlot<String, String> = Slot::FixedUnresolved(vec!["direct".to_string()]);
    let bp_raw_slot = PickErgonomicsHost::builder()
        .str_items(slot_raw)
        .num_items(vec![100u64])
        .build();
    assert!(bp_raw_slot.is_complete());
    let res_raw_slot = bp_raw_slot.resolve().unwrap();
    assert_eq!(res_raw_slot.str_items, vec!["direct"]);
}

#[test]
fn test_pick_cardinality_zero_or_empty() {
    // Exact(0) cardinality
    let choice_empty =
        MultiChoice::new("Pick none", vec!["skip1".to_string(), "skip2".to_string()])
            .with_cardinality(Cardinality::Exact(0));

    let mut bp = PickErgonomicsHost::builder()
        .str_items(choice_empty)
        .num_items(Vec::<u64>::new())
        .build();

    assert!(!bp.is_complete());
    // Confirm with 0 selections is valid for Exact(0)
    bp.str_items.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());
    let resolved = bp.resolve().unwrap();
    assert!(resolved.str_items.is_empty());
    assert!(resolved.num_items.is_empty());
}

// ============================================================================
// 2. Collection Slot Interleaving and Dynamic Append
// ============================================================================

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct CollInterleavingHost {
    #[slot(collection)]
    pub items: Vec<String>,
}

#[test]
fn test_collection_slot_interleaving_and_chaining() {
    let ch1 = SingleChoice::new("C1", vec!["c1_val".to_string()]);
    let ch2 = SingleChoice::new("C2", vec!["c2_val".to_string()]);

    let mut bp = CollInterleavingHost::builder()
        .with_items("start".to_string())
        .items(vec!["bulk1".to_string(), "bulk2".to_string()])
        .with_items(ch1)
        .with_items("mid".to_string())
        .items(vec![ch2])
        .with_items("end".to_string())
        .build();

    assert!(!bp.is_complete());
    assert_eq!(bp.items.len(), 7);

    // Satisfy open choices at index 3 (ch1) and index 5 (ch2)
    bp.items[3].as_choice_mut().unwrap().select(0).unwrap();
    bp.items[3].as_choice_mut().unwrap().confirm().unwrap();
    assert!(!bp.is_complete());

    bp.items[5].as_choice_mut().unwrap().select(0).unwrap();
    bp.items[5].as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().unwrap();
    assert_eq!(
        resolved.items,
        vec!["start", "bulk1", "bulk2", "c1_val", "mid", "c2_val", "end"]
    );
}

// ============================================================================
// 3. Deep Incomplete Detection in Cascading Pick & Collection
// ============================================================================

#[test]
fn test_cascading_pick_unselected_incomplete_does_not_block_but_selected_does() {
    let cand1 = AdversarialLeaf::builder()
        .id(1u32)
        .leaf_val(SingleChoice::new("L1", vec!["val1".to_string()]))
        .build();
    let cand2: AdversarialLeafSchematic =
        AdversarialLeaf::builder().id(2u32).leaf_val("val2".to_string()).build();

    let pick_choice =
        MultiChoice::new("Pick 1", vec![cand1, cand2]).with_cardinality(Cardinality::Exact(1));

    let coll_child: AdversarialLeafSchematic =
        AdversarialLeaf::builder().id(0u32).leaf_val("ok".to_string()).build();
    let mut host: AdversarialBranchBfsSchematic = AdversarialBranchBfs::builder()
        .branch_name("B1".to_string())
        .with_coll_children(coll_child)
        .pick_children(pick_choice)
        .build();

    assert!(!host.is_complete());

    // Selecting candidate 0 (cand1 which has an unsatisfied leaf choice):
    host.pick_children
        .as_choice_mut()
        .unwrap()
        .select(0)
        .unwrap();
    host.pick_children
        .as_choice_mut()
        .unwrap()
        .confirm()
        .unwrap();

    // Choice is confirmed, but selected child is incomplete -> host MUST be incomplete!
    assert!(!host.is_complete());
    match host.clone().resolve() {
        Err(SchematicError::UnresolvedChoice { prompt }) => {
            assert_eq!(prompt, "L1");
        }
        other => panic!("Expected UnresolvedChoice L1, got {:?}", other),
    }

    // Now deselect candidate 0, and select candidate 1 (cand2 which IS complete):
    host.pick_children
        .as_choice_mut()
        .unwrap()
        .deselect(0)
        .unwrap();
    host.pick_children
        .as_choice_mut()
        .unwrap()
        .select(1)
        .unwrap();
    host.pick_children
        .as_choice_mut()
        .unwrap()
        .confirm()
        .unwrap();

    // Now candidate 1 is selected, candidate 0 (incomplete) is ignored!
    assert!(host.is_complete());
    let resolved = host.resolve().unwrap();
    assert_eq!(resolved.branch_name, "B1");
    assert_eq!(resolved.pick_children.len(), 1);
    assert_eq!(resolved.pick_children[0].id, 2);
    assert_eq!(resolved.pick_children[0].leaf_val, "val2");
}

#[test]
fn test_cascading_collection_incomplete_detection_in_choice_and_fixed() {
    let incomplete_leaf = AdversarialLeaf::builder()
        .id(10u32)
        .leaf_val(SingleChoice::new("PromptX", vec!["vx".to_string()]))
        .build();

    // Case A: incomplete child directly added as FixedUnresolved
    let host_fixed: AdversarialBranchBfsSchematic = AdversarialBranchBfs::builder()
        .branch_name("FixedIncomplete".to_string())
        .with_coll_children(incomplete_leaf.clone())
        .pick_children(Vec::<AdversarialLeafSchematic>::new())
        .build();

    assert!(!host_fixed.is_complete());
    assert!(matches!(
        host_fixed.resolve(),
        Err(SchematicError::UnresolvedChoice { .. })
    ));

    // Case B: incomplete child inside a SingleChoice that is confirmed
    let child_choice = SingleChoice::new("Choose Child", vec![incomplete_leaf]);
    let mut host_choice: AdversarialBranchBfsSchematic = AdversarialBranchBfs::builder()
        .branch_name("ChoiceIncomplete".to_string())
        .with_coll_children(child_choice)
        .pick_children(Vec::<AdversarialLeafSchematic>::new())
        .build();

    assert!(!host_choice.is_complete());
    host_choice.coll_children[0]
        .as_choice_mut()
        .unwrap()
        .select(0)
        .unwrap();
    host_choice.coll_children[0]
        .as_choice_mut()
        .unwrap()
        .confirm()
        .unwrap();

    // The SingleChoice is Made, but the inner child is incomplete -> host MUST be incomplete
    assert!(!host_choice.is_complete());
    assert!(matches!(
        host_choice.resolve(),
        Err(SchematicError::UnresolvedChoice { .. })
    ));
}

// ============================================================================
// 4. Full BFS vs DFS Traversal on All 6 Slot Kinds Combined
// ============================================================================

#[test]
fn test_bfs_all_six_slot_kinds_traversal_and_autoselect() {
    let leaf_cand1 = AdversarialLeaf::builder()
        .id(101u32)
        .leaf_val(SingleChoice::new(
            "P_PickChild1_Val",
            vec!["pc1_val".to_string()],
        ))
        .build();
    let leaf_cand2 = AdversarialLeaf::builder()
        .id(102u32)
        .leaf_val(SingleChoice::new(
            "P_PickChild2_Val",
            vec!["pc2_val".to_string()],
        ))
        .build();

    let branch1 = AdversarialBranchBfs::builder()
        .branch_name(SingleChoice::new("P_Branch1_Name", vec!["B1".to_string()]))
        .with_coll_children(
            AdversarialLeaf::builder()
                .id(11u32)
                .leaf_val(SingleChoice::new("P_B1_CollLeaf", vec!["cl1".to_string()]))
                .build(),
        )
        .pick_children(
            MultiChoice::new("P_B1_PickKids", vec![leaf_cand1, leaf_cand2])
                .with_cardinality(Cardinality::Exact(1)),
        )
        .build();

    let single_cascade_leaf: AdversarialLeafSchematic = AdversarialLeaf::builder()
        .id(1u32)
        .leaf_val(SingleChoice::new(
            "P_Root_SingleCascadeLeaf",
            vec!["scl".to_string()],
        ))
        .build();

    let mut root = AdversarialRootBfs::builder()
        .code(1000usize)
        .single_leaf(SingleChoice::new(
            "P_Root_SingleLeaf",
            vec!["sl".to_string()],
        ))
        .with_coll_leaf(SingleChoice::new("P_Root_CollLeaf", vec!["cl".to_string()]))
        .pick_leaf(
            MultiChoice::new(
                "P_Root_PickLeaf",
                vec!["pl1".to_string(), "pl2".to_string()],
            )
            .with_cardinality(Cardinality::Exact(1)),
        )
        .single_cascade(single_cascade_leaf)
        .with_coll_cascade(branch1)
        .pick_cascade(Vec::<AdversarialBranchBfsSchematic>::new())
        .build();

    assert!(!root.is_complete());

    // AutoSelectFirst satisfies every level in BFS automatically
    root.walk_choices(&mut AutoSelectFirst);
    assert!(root.is_complete());

    let resolved = root.resolve().expect("Full BFS root must resolve cleanly");
    assert_eq!(resolved.code, 1000);
    assert_eq!(resolved.single_leaf, "sl");
    assert_eq!(resolved.coll_leaf, vec!["cl"]);
    assert_eq!(resolved.pick_leaf, vec!["pl1"]);
    assert_eq!(resolved.single_cascade.leaf_val, "scl");
    assert_eq!(resolved.coll_cascade.len(), 1);
    assert_eq!(resolved.coll_cascade[0].branch_name, "B1");
    assert_eq!(resolved.coll_cascade[0].coll_children[0].leaf_val, "cl1");
    assert_eq!(resolved.coll_cascade[0].pick_children.len(), 1);
    assert_eq!(
        resolved.coll_cascade[0].pick_children[0].leaf_val,
        "pc1_val"
    );
}

#[test]
fn test_dfs_all_six_slot_kinds_traversal_and_autoselect() {
    let coll_child_dfs: AdversarialLeafSchematic = AdversarialLeaf::builder()
        .id(11u32)
        .leaf_val(SingleChoice::new("P_Dfs_CollLeaf", vec!["cl1".to_string()]))
        .build();

    let branch1 = AdversarialBranchDfs::builder()
        .branch_name(SingleChoice::new(
            "P_Dfs_Branch1_Name",
            vec!["B1".to_string()],
        ))
        .with_coll_children(coll_child_dfs)
        .pick_children(Vec::<AdversarialLeafSchematic>::new())
        .build();

    let single_cascade_dfs: AdversarialLeafSchematic = AdversarialLeaf::builder()
        .id(1u32)
        .leaf_val(SingleChoice::new(
            "P_Dfs_SingleCascadeLeaf",
            vec!["scl".to_string()],
        ))
        .build();

    let mut root = AdversarialRootDfs::builder()
        .code(2000usize)
        .single_leaf(SingleChoice::new(
            "P_Dfs_Root_SingleLeaf",
            vec!["sl".to_string()],
        ))
        .with_coll_leaf(SingleChoice::new(
            "P_Dfs_Root_CollLeaf",
            vec!["cl".to_string()],
        ))
        .pick_leaf(
            MultiChoice::new("P_Dfs_Root_PickLeaf", vec!["pl1".to_string()])
                .with_cardinality(Cardinality::Exact(1)),
        )
        .single_cascade(single_cascade_dfs)
        .with_coll_cascade(branch1)
        .pick_cascade(Vec::<AdversarialBranchDfsSchematic>::new())
        .build();

    assert!(!root.is_complete());

    root.walk_choices(&mut AutoSelectFirst);
    assert!(root.is_complete());

    let resolved = root.resolve().expect("Full DFS root must resolve cleanly");
    assert_eq!(resolved.code, 2000);
    assert_eq!(resolved.single_leaf, "sl");
    assert_eq!(resolved.coll_leaf, vec!["cl"]);
    assert_eq!(resolved.pick_leaf, vec!["pl1"]);
    assert_eq!(resolved.single_cascade.leaf_val, "scl");
    assert_eq!(resolved.coll_cascade[0].branch_name, "B1");
    assert_eq!(resolved.coll_cascade[0].coll_children[0].leaf_val, "cl1");
}

// ============================================================================
// 5. From<Resolved> Symmetric Roundtrip
// ============================================================================

#[test]
fn test_from_resolved_roundtrip_all_slot_kinds() {
    let resolved_model = AdversarialRootBfs {
        code: 9999usize,
        single_leaf: "leaf_final".to_string(),
        coll_leaf: vec!["c1".to_string(), "c2".to_string()],
        pick_leaf: vec!["p1".to_string(), "p2".to_string()],
        single_cascade: AdversarialLeaf {
            id: 77,
            leaf_val: "single_casc".to_string(),
        },
        coll_cascade: vec![AdversarialBranchBfs {
            branch_name: "branch_alpha".to_string(),
            coll_children: vec![AdversarialLeaf {
                id: 1,
                leaf_val: "c_child".to_string(),
            }],
            pick_children: vec![AdversarialLeaf {
                id: 2,
                leaf_val: "p_child".to_string(),
            }],
        }],
        pick_cascade: vec![AdversarialBranchBfs {
            branch_name: "branch_beta".to_string(),
            coll_children: vec![],
            pick_children: vec![],
        }],
    };

    // 1. Convert resolved struct directly into Schematic blueprint
    let converted_schematic = AdversarialRootBfsSchematic::from(resolved_model.clone());

    // 2. Blueprint must immediately be complete because all slots are FixedResolved
    assert!(converted_schematic.is_complete());

    // 3. Re-resolving blueprint produces the exact identical domain model
    let re_resolved = converted_schematic
        .resolve()
        .expect("Re-resolving FixedResolved schematic must succeed");
    assert_eq!(resolved_model, re_resolved);
}
