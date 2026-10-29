use schematic::choice::{Cardinality, ChoiceState, MultiChoice, SingleChoice};
use schematic::slot::{SingleSlot, Slot};
use schematic::visitor::{AutoSelectFirst, ChoiceVisitor, VisitorFlow};
use schematic::{schematic, Schematic};

// ============================================================================
// 1. Deeply Nested Cascading Options (Depth 3: GreatGrandChild -> GrandChild -> Child -> Root)
// ============================================================================

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepLeaf {
    #[slot]
    pub val: String,
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepNodeBfs {
    #[slot]
    pub name: String,
    #[slot(option, cascade)]
    pub leaf: Option<DeepLeaf>,
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepMidBfs {
    pub mid_id: u32,
    #[slot(option, cascade)]
    pub node: Option<DeepNodeBfs>,
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepRootBfs {
    pub root_id: u32,
    #[slot(option, cascade)]
    pub mid: Option<DeepMidBfs>,
}

#[test]
fn test_deeply_nested_cascading_option_all_present_bfs() {
    let leaf_choice = SingleChoice::new("LeafVal", vec!["L1".to_string(), "L2".to_string()]);
    let leaf = DeepLeafSchematic::builder().val(leaf_choice).build();

    let node_choice = SingleChoice::new("NodeName", vec!["N1".to_string(), "N2".to_string()]);
    let node = DeepNodeBfsSchematic::builder()
        .name(node_choice)
        .leaf(leaf)
        .build();

    let mid = DeepMidBfsSchematic::builder()
        .mid_id(42u32)
        .node(node)
        .build();

    let mut root = DeepRootBfsSchematic::builder()
        .root_id(1u32)
        .mid(mid)
        .build();

    assert!(!root.is_complete());
    assert!(root.clone().resolve().is_err());

    // AutoSelectFirst visits all choices down the 3-tier cascade
    root.walk_choices(&mut AutoSelectFirst);
    assert!(root.is_complete());

    let resolved = root.resolve().expect("must resolve 3-tier cascade");
    assert_eq!(resolved.root_id, 1);
    let r_mid = resolved.mid.clone().expect("mid must be Some");
    assert_eq!(r_mid.mid_id, 42);
    let r_node = r_mid.node.expect("node must be Some");
    assert_eq!(r_node.name, "N1");
    let r_leaf = r_node.leaf.expect("leaf must be Some");
    assert_eq!(r_leaf.val, "L1");

    // From<Resolved> roundtrip
    let back: DeepRootBfsSchematic = resolved.into();
    assert!(back.is_complete());
    assert!(matches!(back.mid, Some(SingleSlot::FixedResolved(_))));
}

#[test]
fn test_deeply_nested_cascading_option_intermediate_none_bfs() {
    // mid is present, but node is None
    let mid = DeepMidBfsSchematic::builder()
        .mid_id(42u32)
        .build(); // node defaults to None

    let mut root = DeepRootBfsSchematic::builder()
        .root_id(1u32)
        .mid(mid)
        .build();

    assert!(root.is_complete());
    root.walk_choices(&mut AutoSelectFirst);
    assert!(root.is_complete());

    let resolved = root.resolve().expect("must resolve with intermediate None");
    assert_eq!(resolved.root_id, 1);
    let r_mid = resolved.mid.unwrap();
    assert_eq!(r_mid.mid_id, 42);
    assert_eq!(r_mid.node, None);
}

#[test]
fn test_deeply_nested_cascading_option_root_none_bfs() {
    // mid defaults to None
    let mut root = DeepRootBfsSchematic::builder()
        .root_id(99u32)
        .build();

    assert!(root.is_complete());
    root.walk_choices(&mut AutoSelectFirst);
    assert!(root.is_complete());

    let resolved = root.resolve().expect("must resolve root None");
    assert_eq!(resolved.root_id, 99);
    assert_eq!(resolved.mid, None);
}

// Depth 3 optional cascading hierarchy in DFS
#[schematic(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepNodeDfs {
    #[slot]
    pub name: String,
    #[slot(option, cascade)]
    pub leaf: Option<DeepLeaf>,
}

#[schematic(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepMidDfs {
    pub mid_id: u32,
    #[slot(option, cascade)]
    pub node: Option<DeepNodeDfs>,
}

#[schematic(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeepRootDfs {
    pub root_id: u32,
    #[slot(option, cascade)]
    pub mid: Option<DeepMidDfs>,
}

#[test]
fn test_deeply_nested_cascading_option_all_present_dfs() {
    let leaf_choice = SingleChoice::new("LeafVal", vec!["L_DFS".to_string()]);
    let leaf = DeepLeafSchematic::builder().val(leaf_choice).build();

    let node = DeepNodeDfsSchematic::builder()
        .name("NodeDfs".to_string())
        .leaf(leaf)
        .build();

    let mid = DeepMidDfsSchematic::builder()
        .mid_id(100u32)
        .node(node)
        .build();

    let mut root = DeepRootDfsSchematic::builder()
        .root_id(200u32)
        .mid(mid)
        .build();

    assert!(!root.is_complete());
    root.walk_choices(&mut AutoSelectFirst);
    assert!(root.is_complete());

    let resolved = root.resolve().expect("must resolve DFS 3-tier cascade");
    assert_eq!(
        resolved.mid.unwrap().node.unwrap().leaf.unwrap().val,
        "L_DFS"
    );
}

// Choice at the optional cascade level itself
#[test]
fn test_choice_at_optional_cascade_level() {
    let leaf_a = DeepLeafSchematic::builder().val("Alpha".to_string()).build();
    let leaf_b = DeepLeafSchematic::builder().val("Beta".to_string()).build();

    let leaf_choice = SingleChoice::new("PickLeaf", vec![leaf_a, leaf_b]);

    let mut node = DeepNodeBfsSchematic::builder()
        .name("Branch".to_string())
        .leaf(leaf_choice)
        .build();

    assert!(!node.is_complete());
    assert!(node.clone().resolve().is_err());

    node.walk_choices(&mut AutoSelectFirst);
    assert!(node.is_complete());

    let resolved = node.resolve().unwrap();
    assert_eq!(resolved.leaf.unwrap().val, "Alpha");
}

// ============================================================================
// 2. Interaction with Serde JSON Serialization / Deserialization
// ============================================================================

#[schematic(
    builder,
    derives(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerdeChild {
    #[slot]
    pub tag: String,
}

#[schematic(
    builder,
    derives(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerdeParent {
    pub label: String,
    #[slot(option)]
    pub opt_scalar: Option<String>,
    #[slot(option, cascade)]
    pub opt_child: Option<SerdeChild>,
}

#[test]
fn test_serde_json_roundtrip_none_slots() {
    let blueprint = SerdeParentSchematic::builder()
        .label("TestLabel".to_string())
        .build();

    assert!(blueprint.is_complete());

    let json = serde_json::to_string(&blueprint).expect("must serialize blueprint with None");
    let deserialized: SerdeParentSchematic =
        serde_json::from_str(&json).expect("must deserialize blueprint with None");

    assert_eq!(blueprint, deserialized);
    assert!(deserialized.is_complete());

    let resolved = deserialized.resolve().expect("must resolve");
    assert_eq!(resolved.label, "TestLabel");
    assert_eq!(resolved.opt_scalar, None);
    assert_eq!(resolved.opt_child, None);

    // Resolved struct serde roundtrip
    let res_json = serde_json::to_string(&resolved).expect("must serialize resolved");
    let res_deserialized: SerdeParent =
        serde_json::from_str(&res_json).expect("must deserialize resolved");
    assert_eq!(resolved, res_deserialized);
}

#[test]
fn test_serde_json_roundtrip_fixed_slots() {
    let child = SerdeChildSchematic::builder()
        .tag("ChildTag".to_string())
        .build();

    let blueprint = SerdeParentSchematic::builder()
        .label("FixedLabel".to_string())
        .opt_scalar("ScalarVal".to_string())
        .opt_child(child)
        .build();

    assert!(blueprint.is_complete());

    let json = serde_json::to_string(&blueprint).expect("must serialize blueprint with Fixed");
    let deserialized: SerdeParentSchematic =
        serde_json::from_str(&json).expect("must deserialize blueprint with Fixed");

    assert_eq!(blueprint, deserialized);
    assert!(deserialized.is_complete());

    let resolved = deserialized.resolve().unwrap();
    assert_eq!(resolved.opt_scalar, Some("ScalarVal".to_string()));
    assert_eq!(
        resolved.opt_child,
        Some(SerdeChild {
            tag: "ChildTag".to_string()
        })
    );
}

#[test]
fn test_serde_json_roundtrip_choice_slots() {
    let choice = SingleChoice::new("ChooseOpt", vec!["ChoiceA".to_string(), "ChoiceB".to_string()]);
    let blueprint = SerdeParentSchematic::builder()
        .label("ChoiceLabel".to_string())
        .opt_scalar(choice)
        .build();

    assert!(!blueprint.is_complete());

    // Serialize uncompleted choice in blueprint
    let json = serde_json::to_string(&blueprint).expect("must serialize uncompleted choice");
    let mut deserialized: SerdeParentSchematic =
        serde_json::from_str(&json).expect("must deserialize uncompleted choice");

    assert_eq!(blueprint, deserialized);
    assert!(!deserialized.is_complete());

    // Make choice on deserialized blueprint
    deserialized.walk_choices(&mut AutoSelectFirst);
    assert!(deserialized.is_complete());

    let resolved = deserialized.resolve().unwrap();
    assert_eq!(resolved.opt_scalar, Some("ChoiceA".to_string()));
}

// ============================================================================
// 3. Visitor Flow Control Break on Optional Choices
// ============================================================================

struct FlowBreakOnPromptVisitor {
    pub target_prompt: String,
    pub visited_prompts: Vec<String>,
}

impl FlowBreakOnPromptVisitor {
    pub fn new(target: &str) -> Self {
        Self {
            target_prompt: target.to_string(),
            visited_prompts: Vec::new(),
        }
    }
}

impl<T: Clone> ChoiceVisitor<T> for FlowBreakOnPromptVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        let prompt = choice.prompt().to_string();
        self.visited_prompts.push(prompt.clone());
        if prompt == self.target_prompt {
            VisitorFlow::Break
        } else {
            VisitorFlow::Continue
        }
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        let prompt = choice.prompt().to_string();
        self.visited_prompts.push(prompt.clone());
        if prompt == self.target_prompt {
            VisitorFlow::Break
        } else {
            VisitorFlow::Continue
        }
    }
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequentialSlotsBfs {
    #[slot]
    pub first: String,
    #[slot(option)]
    pub middle_opt: Option<String>,
    #[slot]
    pub last: String,
}

#[test]
fn test_visitor_flow_control_break_on_optional_leaf() {
    let mut seq = SequentialSlotsBfsSchematic::builder()
        .first(SingleChoice::new("First", vec!["1".to_string()]))
        .middle_opt(SingleChoice::new("Middle", vec!["2".to_string()]))
        .last(SingleChoice::new("Last", vec!["3".to_string()]))
        .build();

    let mut visitor = FlowBreakOnPromptVisitor::new("Middle");
    let flow = seq.walk_choices(&mut visitor);

    assert_eq!(flow, VisitorFlow::Break);
    // "Last" must NEVER be visited!
    assert_eq!(visitor.visited_prompts, vec!["First", "Middle"]);
}

#[test]
fn test_visitor_flow_control_skips_none_and_continues() {
    let mut seq = SequentialSlotsBfsSchematic::builder()
        .first(SingleChoice::new("First", vec!["1".to_string()]))
        // middle_opt omitted -> None
        .last(SingleChoice::new("Last", vec!["3".to_string()]))
        .build();

    let mut visitor = FlowBreakOnPromptVisitor::new("NonExistent");
    let flow = seq.walk_choices(&mut visitor);

    assert_eq!(flow, VisitorFlow::Continue);
    // Middle is skipped cleanly without breaking
    assert_eq!(visitor.visited_prompts, vec!["First", "Last"]);
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildForBreakBfs {
    #[slot]
    pub child_choice: String,
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentForBreakBfs {
    #[slot]
    pub parent_first: String,
    #[slot(option, cascade)]
    pub child_opt: Option<ChildForBreakBfs>,
    #[slot]
    pub parent_after: String,
}

#[test]
fn test_visitor_flow_control_break_inside_cascading_option_bfs() {
    let child = ChildForBreakBfsSchematic::builder()
        .child_choice(SingleChoice::new("ChildSlot", vec!["C".to_string()]))
        .build();

    let mut parent = ParentForBreakBfsSchematic::builder()
        .parent_first(SingleChoice::new("ParentFirst", vec!["P".to_string()]))
        .child_opt(child)
        .parent_after(SingleChoice::new("ParentAfter", vec!["A".to_string()]))
        .build();

    // In BFS:
    // Level 0: ParentFirst, ParentAfter
    // Level 1: ChildSlot
    // If break is on ChildSlot at level 1:
    let mut visitor = FlowBreakOnPromptVisitor::new("ChildSlot");
    let flow = parent.walk_choices(&mut visitor);

    assert_eq!(flow, VisitorFlow::Break);
    assert_eq!(
        visitor.visited_prompts,
        vec!["ParentFirst", "ParentAfter", "ChildSlot"]
    );
}

#[schematic(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildForBreakDfs {
    #[slot]
    pub child_choice: String,
}

#[schematic(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentForBreakDfs {
    #[slot]
    pub parent_first: String,
    #[slot(option, cascade)]
    pub child_opt: Option<ChildForBreakDfs>,
    #[slot]
    pub parent_after: String,
}

#[test]
fn test_visitor_flow_control_break_inside_cascading_option_dfs() {
    let child = ChildForBreakDfsSchematic::builder()
        .child_choice(SingleChoice::new("ChildSlot", vec!["C".to_string()]))
        .build();

    let mut parent = ParentForBreakDfsSchematic::builder()
        .parent_first(SingleChoice::new("ParentFirst", vec!["P".to_string()]))
        .child_opt(child)
        .parent_after(SingleChoice::new("ParentAfter", vec!["A".to_string()]))
        .build();

    // In DFS:
    // ParentFirst, then child's ChildSlot, then ParentAfter.
    // If break on ChildSlot, ParentAfter must NOT be visited!
    let mut visitor = FlowBreakOnPromptVisitor::new("ChildSlot");
    let flow = parent.walk_choices(&mut visitor);

    assert_eq!(flow, VisitorFlow::Break);
    assert_eq!(visitor.visited_prompts, vec!["ParentFirst", "ChildSlot"]);
}

// ============================================================================
// 4. Lifecycle: unmake_all() and confirm_all() on Optional Choices
// ============================================================================

#[test]
fn test_unmake_all_and_confirm_all_lifecycle_with_option_slots() {
    let mut seq = SequentialSlotsBfsSchematic::builder()
        .first(SingleChoice::new("First", vec!["1".to_string()]))
        .middle_opt(SingleChoice::new("Middle", vec!["2".to_string()]))
        .last(SingleChoice::new("Last", vec!["3".to_string()]))
        .build();

    assert!(!seq.is_complete());

    // 1. AutoSelectFirst satisfies all choices
    seq.walk_choices(&mut AutoSelectFirst);
    assert!(seq.is_complete());

    let resolved = seq.clone().resolve().expect("must resolve after choice made");
    assert_eq!(resolved.first, "1");
    assert_eq!(resolved.middle_opt, Some("2".to_string()));
    assert_eq!(resolved.last, "3");

    // 2. unmake_all() reverts choices to UnmadeWithHistory, making blueprint incomplete
    seq.unmake_all();
    assert!(!seq.is_complete());
    assert!(seq.clone().resolve().is_err());

    if let Some(Slot::Choose(choice)) = &seq.middle_opt {
        assert!(matches!(choice.state(), ChoiceState::UnmadeWithHistory(_)));
    } else {
        panic!("middle_opt must be Some(Slot::Choose)");
    }

    // 3. confirm_all() restores choices to Made, making blueprint complete again
    seq.confirm_all();
    assert!(seq.is_complete());

    if let Some(Slot::Choose(choice)) = &seq.middle_opt {
        assert!(matches!(choice.state(), ChoiceState::Made(_)));
    } else {
        panic!("middle_opt must be Some(Slot::Choose)");
    }

    let resolved_restored = seq.resolve().expect("must resolve after confirm_all");
    assert_eq!(resolved, resolved_restored);
}

#[test]
fn test_unmake_all_and_confirm_all_on_none_option_slot() {
    let mut seq = SequentialSlotsBfsSchematic::builder()
        .first(SingleChoice::new("First", vec!["1".to_string()]))
        // middle_opt is None
        .last(SingleChoice::new("Last", vec!["3".to_string()]))
        .build();

    seq.walk_choices(&mut AutoSelectFirst);
    assert!(seq.is_complete());

    // unmake_all should not affect None
    seq.unmake_all();
    assert_eq!(seq.middle_opt, None);

    // confirm_all should not affect None
    seq.confirm_all();
    assert_eq!(seq.middle_opt, None);
    assert!(seq.is_complete());
}

// ============================================================================
// 5. Complete 2x4 Slot Matrix in a Single Struct
// ============================================================================

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatrixChild {
    #[slot]
    pub val: String,
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullEightSlotMatrix {
    // 1. Single leaf
    #[slot]
    pub single_leaf: String,
    // 2. Single cascade
    #[slot(cascade)]
    pub single_cascade: MatrixChild,
    // 3. Option leaf
    #[slot(option)]
    pub option_leaf: Option<String>,
    // 4. Option cascade
    #[slot(option, cascade)]
    pub option_cascade: Option<MatrixChild>,
    // 5. Collection leaf
    #[slot(collection)]
    pub coll_leaf: Vec<String>,
    // 6. Collection cascade
    #[slot(collection, cascade)]
    pub coll_cascade: Vec<MatrixChild>,
    // 7. Pick leaf
    #[slot(pick)]
    pub pick_leaf: Vec<String>,
    // 8. Pick cascade
    #[slot(pick, cascade)]
    pub pick_cascade: Vec<MatrixChild>,
}

#[test]
fn test_full_eight_slot_matrix_minimal_omissions() {
    // Test minimal defaults: options are None, collections/picks are empty (or provided as empty)
    let blueprint = FullEightSlotMatrixSchematic::builder()
        .single_leaf("SLeaf".to_string())
        .single_cascade(MatrixChildSchematic::builder().val("SCascade".to_string()).build())
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<MatrixChildSchematic>::new())
        .build();

    assert!(blueprint.is_complete());
    let resolved = blueprint.resolve().expect("must resolve minimal 8-slot matrix");
    assert_eq!(resolved.single_leaf, "SLeaf");
    assert_eq!(resolved.single_cascade.val, "SCascade");
    assert_eq!(resolved.option_leaf, None);
    assert_eq!(resolved.option_cascade, None);
    assert!(resolved.coll_leaf.is_empty());
    assert!(resolved.coll_cascade.is_empty());
    assert!(resolved.pick_leaf.is_empty());
    assert!(resolved.pick_cascade.is_empty());

    let back: FullEightSlotMatrixSchematic = resolved.into();
    assert!(back.is_complete());
}

#[test]
fn test_full_eight_slot_matrix_all_populated_with_choices() {
    let single_leaf_choice = SingleChoice::new("SLeaf", vec!["sl_val".to_string()]);
    let single_cascade_child = MatrixChildSchematic::builder()
        .val(SingleChoice::new("SCascade", vec!["sc_val".to_string()]))
        .build();

    let opt_leaf_choice = SingleChoice::new("OptLeaf", vec!["ol_val".to_string()]);
    let opt_cascade_child = MatrixChildSchematic::builder()
        .val(SingleChoice::new("OptCascade", vec!["oc_val".to_string()]))
        .build();

    let coll_leaf_choice = SingleChoice::new("CollLeaf", vec!["cl_val".to_string()]);
    let coll_cascade_child = MatrixChildSchematic::builder()
        .val(SingleChoice::new("CollCascade", vec!["cc_val".to_string()]))
        .build();

    let pick_leaf_choice = MultiChoice::new("PickLeaf", vec!["pl_1".to_string(), "pl_2".to_string()])
        .with_cardinality(Cardinality::Exact(1));

    let pick_cascade_cand = MatrixChildSchematic::builder()
        .val(SingleChoice::new("PickCascade", vec!["pc_val".to_string()]))
        .build();
    let pick_cascade_choice = MultiChoice::new("PickCascade", vec![pick_cascade_cand])
        .with_cardinality(Cardinality::Exact(1));

    let mut blueprint = FullEightSlotMatrixSchematic::builder()
        .single_leaf(single_leaf_choice)
        .single_cascade(single_cascade_child)
        .option_leaf(opt_leaf_choice)
        .option_cascade(opt_cascade_child)
        .with_coll_leaf(coll_leaf_choice)
        .with_coll_cascade(coll_cascade_child)
        .pick_leaf(pick_leaf_choice)
        .pick_cascade(pick_cascade_choice)
        .build();

    assert!(!blueprint.is_complete());

    // AutoSelectFirst satisfies all 8 slots (including single, option, collection, pick, leaf & cascading)
    blueprint.walk_choices(&mut AutoSelectFirst);
    assert!(blueprint.is_complete());

    let resolved = blueprint.resolve().expect("must resolve all 8 slots");
    assert_eq!(resolved.single_leaf, "sl_val");
    assert_eq!(resolved.single_cascade.val, "sc_val");
    assert_eq!(resolved.option_leaf, Some("ol_val".to_string()));
    assert_eq!(resolved.option_cascade, Some(MatrixChild { val: "oc_val".to_string() }));
    assert_eq!(resolved.coll_leaf, vec!["cl_val".to_string()]);
    assert_eq!(resolved.coll_cascade, vec![MatrixChild { val: "cc_val".to_string() }]);
    assert_eq!(resolved.pick_leaf, vec!["pl_1".to_string()]);
    assert_eq!(resolved.pick_cascade, vec![MatrixChild { val: "pc_val".to_string() }]);

    // From<Resolved> roundtrip
    let back: FullEightSlotMatrixSchematic = resolved.into();
    assert!(back.is_complete());
}

// ============================================================================
// 6. Direct Resolved Builder Ergonomics for Optional Slots
// ============================================================================

#[test]
fn test_resolved_builder_omission_and_explicit_variants() {
    // 1. Omission defaults to None
    let res_omitted: FullEightSlotMatrix = FullEightSlotMatrix::builder()
        .single_leaf("sl".to_string())
        .single_cascade(MatrixChild { val: "sc".to_string() })
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<MatrixChild>::new())
        .build();
    assert_eq!(res_omitted.option_leaf, None);
    assert_eq!(res_omitted.option_cascade, None);

    // 2. Explicit None
    let res_none: FullEightSlotMatrix = FullEightSlotMatrix::builder()
        .single_leaf("sl".to_string())
        .single_cascade(MatrixChild { val: "sc".to_string() })
        .option_leaf(None)
        .option_cascade(None)
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<MatrixChild>::new())
        .build();
    assert_eq!(res_none.option_leaf, None);
    assert_eq!(res_none.option_cascade, None);

    // 3. Explicit Some(val)
    let res_some: FullEightSlotMatrix = FullEightSlotMatrix::builder()
        .single_leaf("sl".to_string())
        .single_cascade(MatrixChild { val: "sc".to_string() })
        .option_leaf(Some("opt_str".to_string()))
        .option_cascade(Some(MatrixChild { val: "child_val".to_string() }))
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<MatrixChild>::new())
        .build();
    assert_eq!(res_some.option_leaf, Some("opt_str".to_string()));
    assert_eq!(res_some.option_cascade, Some(MatrixChild { val: "child_val".to_string() }));

    // 4. Bare value passed directly into setter
    let res_bare: FullEightSlotMatrix = FullEightSlotMatrix::builder()
        .single_leaf("sl".to_string())
        .single_cascade(MatrixChild { val: "sc".to_string() })
        .option_leaf("bare_str".to_string())
        .option_cascade(MatrixChild { val: "bare_child".to_string() })
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<MatrixChild>::new())
        .build();
    assert_eq!(res_bare.option_leaf, Some("bare_str".to_string()));
    assert_eq!(res_bare.option_cascade, Some(MatrixChild { val: "bare_child".to_string() }));
}

// ============================================================================
// 7. Adversarial Choice State Transitions & Incomplete Cascades
// ============================================================================

#[test]
fn test_adversarial_staged_vs_confirmed_choice_in_optional_slot() {
    let mut choice = SingleChoice::new("OptPrompt", vec!["A".to_string(), "B".to_string()]);
    // Stage candidate 0 without confirming
    choice.select(0).unwrap();
    assert!(!choice.is_made());

    let mut blueprint = SequentialSlotsBfsSchematic::builder()
        .first("first".to_string())
        .middle_opt(choice)
        .last("last".to_string())
        .build();

    // Staged but not confirmed means slot is incomplete!
    assert!(!blueprint.is_complete());
    assert!(blueprint.clone().resolve().is_err());

    // Confirm choice
    if let Some(Slot::Choose(c)) = &mut blueprint.middle_opt {
        c.confirm().unwrap();
        assert!(c.is_made());
    }

    assert!(blueprint.is_complete());
    let resolved = blueprint.resolve().expect("must resolve confirmed choice");
    assert_eq!(resolved.middle_opt, Some("A".to_string()));
}

#[test]
fn test_adversarial_empty_choice_in_optional_slot() {
    let empty_choice: SingleChoice<String> = SingleChoice::new("EmptyPrompt", Vec::<String>::new());

    let blueprint = SequentialSlotsBfsSchematic::builder()
        .first("first".to_string())
        .middle_opt(empty_choice)
        .last("last".to_string())
        .build();

    assert!(!blueprint.is_complete());
    assert!(blueprint.clone().resolve().is_err());
}

#[test]
fn test_cascading_option_with_candidate_selection_completeness() {
    // Child A is complete
    let child_a = DeepLeafSchematic::builder().val("ValA".to_string()).build();
    assert!(child_a.is_complete());

    // Child B has an unmade choice, so it is incomplete
    let child_b = DeepLeafSchematic::builder()
        .val(SingleChoice::new("ChoiceB", vec!["ValB".to_string()]))
        .build();
    assert!(!child_b.is_complete());

    let mut choice = SingleChoice::new("PickChild", vec![child_a, child_b]);

    // Select candidate 1 (which is incomplete child_b) and confirm the choice itself
    choice.select(1).unwrap();
    choice.confirm().unwrap();
    assert!(choice.is_made());

    let mut node = DeepNodeBfsSchematic::builder()
        .name("Node".to_string())
        .leaf(choice)
        .build();

    // Node is incomplete because selected child is incomplete!
    assert!(!node.is_complete());
    assert!(node.clone().resolve().is_err());

    // AutoSelectFirst will traverse into the selected child and satisfy ChoiceB
    node.walk_choices(&mut AutoSelectFirst);
    assert!(node.is_complete());

    let resolved = node.resolve().expect("must resolve once child choice is made");
    assert_eq!(resolved.leaf.unwrap().val, "ValB");
}

#[test]
fn test_schematic_builder_accepts_pre_resolved_cascading_child() {
    // Pre-resolved MatrixChild passed directly to schematic builder
    let pre_resolved_child: MatrixChild = MatrixChild { val: "PreResolved".to_string() };

    let blueprint = FullEightSlotMatrixSchematic::builder()
        .single_leaf("sl".to_string())
        .single_cascade(MatrixChildSchematic::builder().val("sc".to_string()).build())
        .option_cascade(pre_resolved_child)
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<MatrixChildSchematic>::new())
        .build();

    assert!(blueprint.is_complete());
    assert!(matches!(blueprint.option_cascade, Some(SingleSlot::FixedResolved(_))));

    let resolved = blueprint.resolve().unwrap();
    assert_eq!(resolved.option_cascade, Some(MatrixChild { val: "PreResolved".to_string() }));
}

#[test]
fn test_raw_json_deserialization_edge_cases() {
    // 1. JSON with null for opt_scalar
    let json_null = r#"{"label":"foo","opt_scalar":null,"opt_child":null}"#;
    let de_null: SerdeParentSchematic = serde_json::from_str(json_null).unwrap();
    assert_eq!(de_null.opt_scalar, None);
    assert_eq!(de_null.opt_child, None);
    assert!(de_null.is_complete());

    // 2. JSON with fixed string value for opt_scalar
    let json_val = r#"{"label":"foo","opt_scalar":"hello","opt_child":{"tag":"bar"}}"#;
    let de_val: SerdeParentSchematic = serde_json::from_str(json_val).unwrap();
    assert!(de_val.is_complete());
    let resolved = de_val.resolve().unwrap();
    assert_eq!(resolved.opt_scalar, Some("hello".to_string()));
    assert_eq!(resolved.opt_child, Some(SerdeChild { tag: "bar".to_string() }));
}

