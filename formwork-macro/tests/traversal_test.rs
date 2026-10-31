use formwork::choice::{ChoiceState, MultiChoice, SingleChoice};
use formwork::visitor::{AutoSelectFirst, ChoiceVisitor, VisitorFlow};
use formwork::{Formwork, formwork};

// ---------------------------------------------------------------------------
// Traversal Models: BFS (Depth >= 2 Branching Tree)
// ---------------------------------------------------------------------------

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct LeafBfs {
    #[slot]
    pub choice: String,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct BranchBfs {
    #[slot]
    pub choice: String,
    #[slot(cascade)]
    pub leaf1: LeafBfs,
    #[slot(cascade)]
    pub leaf2: LeafBfs,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct RootBfs {
    #[slot]
    pub choice: String,
    #[slot(cascade)]
    pub branch_a: BranchBfs,
    #[slot(cascade)]
    pub branch_b: BranchBfs,
}

fn build_bfs_tree() -> RootBfsFormwork {
    RootBfs::builder()
        .choice(SingleChoice::new("R0", vec!["r0_val".to_string()]))
        .branch_a(
            BranchBfs::builder()
                .choice(SingleChoice::new("C1", vec!["c1_val".to_string()]))
                .leaf1(
                    LeafBfs::builder()
                        .choice(SingleChoice::new("L1_1", vec!["l1_1_val".to_string()]))
                        .build(),
                )
                .leaf2(
                    LeafBfs::builder()
                        .choice(SingleChoice::new("L1_2", vec!["l1_2_val".to_string()]))
                        .build(),
                )
                .build(),
        )
        .branch_b(
            BranchBfs::builder()
                .choice(SingleChoice::new("C2", vec!["c2_val".to_string()]))
                .leaf1(
                    LeafBfs::builder()
                        .choice(SingleChoice::new("L2_1", vec!["l2_1_val".to_string()]))
                        .build(),
                )
                .leaf2(
                    LeafBfs::builder()
                        .choice(SingleChoice::new("L2_2", vec!["l2_2_val".to_string()]))
                        .build(),
                )
                .build(),
        )
        .build()
}

// ---------------------------------------------------------------------------
// Traversal Models: DFS (Depth >= 2 Branching Tree)
// ---------------------------------------------------------------------------

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct LeafDfs {
    #[slot]
    pub choice: String,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct BranchDfs {
    #[slot]
    pub choice: String,
    #[slot(cascade)]
    pub leaf1: LeafDfs,
    #[slot(cascade)]
    pub leaf2: LeafDfs,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct RootDfs {
    #[slot]
    pub choice: String,
    #[slot(cascade)]
    pub branch_a: BranchDfs,
    #[slot(cascade)]
    pub branch_b: BranchDfs,
}

fn build_dfs_tree() -> RootDfsFormwork {
    RootDfs::builder()
        .choice(SingleChoice::new("R0", vec!["r0_val".to_string()]))
        .branch_a(
            BranchDfs::builder()
                .choice(SingleChoice::new("C1", vec!["c1_val".to_string()]))
                .leaf1(
                    LeafDfs::builder()
                        .choice(SingleChoice::new("L1_1", vec!["l1_1_val".to_string()]))
                        .build(),
                )
                .leaf2(
                    LeafDfs::builder()
                        .choice(SingleChoice::new("L1_2", vec!["l1_2_val".to_string()]))
                        .build(),
                )
                .build(),
        )
        .branch_b(
            BranchDfs::builder()
                .choice(SingleChoice::new("C2", vec!["c2_val".to_string()]))
                .leaf1(
                    LeafDfs::builder()
                        .choice(SingleChoice::new("L2_1", vec!["l2_1_val".to_string()]))
                        .build(),
                )
                .leaf2(
                    LeafDfs::builder()
                        .choice(SingleChoice::new("L2_2", vec!["l2_2_val".to_string()]))
                        .build(),
                )
                .build(),
        )
        .build()
}

// ---------------------------------------------------------------------------
// Tracing Visitors
// ---------------------------------------------------------------------------

#[derive(Default)]
struct TracingVisitor {
    visited: Vec<String>,
}

impl<T> ChoiceVisitor<T> for TracingVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        self.visited.push(choice.prompt().to_string());
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        self.visited.push(choice.prompt().to_string());
        VisitorFlow::Continue
    }
}

struct BreakAtVisitor {
    break_target: String,
    visited: Vec<String>,
}

impl BreakAtVisitor {
    fn new(target: impl Into<String>) -> Self {
        Self {
            break_target: target.into(),
            visited: Vec::new(),
        }
    }
}

impl<T> ChoiceVisitor<T> for BreakAtVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        self.visited.push(choice.prompt().to_string());
        if choice.prompt() == self.break_target {
            VisitorFlow::Break
        } else {
            VisitorFlow::Continue
        }
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        self.visited.push(choice.prompt().to_string());
        if choice.prompt() == self.break_target {
            VisitorFlow::Break
        } else {
            VisitorFlow::Continue
        }
    }
}

// ---------------------------------------------------------------------------
// Test Cases 11–13
// ---------------------------------------------------------------------------

#[test]
fn test_11_deterministic_bfs_vs_dfs_traversal_order() {
    // 1. BFS Traversal Order Proof
    let mut bfs_tree = build_bfs_tree();
    let mut bfs_visitor = TracingVisitor::default();
    let flow = bfs_tree.walk_choices(&mut bfs_visitor);
    assert_eq!(flow, VisitorFlow::Continue);
    assert_eq!(
        bfs_visitor.visited,
        vec!["R0", "C1", "C2", "L1_1", "L1_2", "L2_1", "L2_2"]
    );

    // 2. DFS Traversal Order Proof
    let mut dfs_tree = build_dfs_tree();
    let mut dfs_visitor = TracingVisitor::default();
    let flow = dfs_tree.walk_choices(&mut dfs_visitor);
    assert_eq!(flow, VisitorFlow::Continue);
    assert_eq!(
        dfs_visitor.visited,
        vec!["R0", "C1", "L1_1", "L1_2", "C2", "L2_1", "L2_2"]
    );
}

#[test]
fn test_12_visitor_flow_control_break_at_root() {
    // Returning Break at root stops immediately (vec!["R0"])
    let mut bfs_tree = build_bfs_tree();
    let mut visitor = BreakAtVisitor::new("R0");
    let flow = bfs_tree.walk_choices(&mut visitor);
    assert_eq!(flow, VisitorFlow::Break);
    assert_eq!(visitor.visited, vec!["R0"]);

    let mut dfs_tree = build_dfs_tree();
    let mut visitor = BreakAtVisitor::new("R0");
    let flow = dfs_tree.walk_choices(&mut visitor);
    assert_eq!(flow, VisitorFlow::Break);
    assert_eq!(visitor.visited, vec!["R0"]);
}

#[test]
fn test_12_visitor_flow_control_break_at_intermediate_node() {
    // Returning Break at intermediate node ("C1") stops immediately without visiting "C2" or leaves
    let mut bfs_tree = build_bfs_tree();
    let mut visitor = BreakAtVisitor::new("C1");
    let flow = bfs_tree.walk_choices(&mut visitor);
    assert_eq!(flow, VisitorFlow::Break);
    assert_eq!(visitor.visited, vec!["R0", "C1"]);

    let mut dfs_tree = build_dfs_tree();
    let mut visitor = BreakAtVisitor::new("C1");
    let flow = dfs_tree.walk_choices(&mut visitor);
    assert_eq!(flow, VisitorFlow::Break);
    assert_eq!(visitor.visited, vec!["R0", "C1"]);
}

#[test]
fn test_13_built_in_visitors_lifecycle_bfs() {
    let mut tree = build_bfs_tree();
    assert!(!tree.is_complete());

    // AutoSelectFirst on multi-level tree satisfies all choices
    tree.walk_choices(&mut AutoSelectFirst);
    assert!(tree.is_complete());

    let resolved_initial = tree
        .clone()
        .resolve()
        .expect("Tree should resolve after AutoSelectFirst");
    assert_eq!(resolved_initial.choice, "r0_val");
    assert_eq!(resolved_initial.branch_a.choice, "c1_val");
    assert_eq!(resolved_initial.branch_a.leaf1.choice, "l1_1_val");
    assert_eq!(resolved_initial.branch_a.leaf2.choice, "l1_2_val");
    assert_eq!(resolved_initial.branch_b.choice, "c2_val");
    assert_eq!(resolved_initial.branch_b.leaf1.choice, "l2_1_val");
    assert_eq!(resolved_initial.branch_b.leaf2.choice, "l2_2_val");

    // unmake_all() reverts choices to UnmadeWithHistory, making tree incomplete
    tree.unmake_all();
    assert!(!tree.is_complete());

    assert!(matches!(
        tree.choice.as_choice().unwrap().state,
        ChoiceState::UnmadeWithHistory(_)
    ));

    // confirm_all() restores choices to Made, making tree complete again
    tree.confirm_all();
    assert!(tree.is_complete());

    assert!(matches!(
        tree.choice.as_choice().unwrap().state,
        ChoiceState::Made(_)
    ));

    // Resolves to the exact same domain data
    let resolved_restored = tree
        .resolve()
        .expect("Tree should resolve after confirm_all");
    assert_eq!(resolved_initial, resolved_restored);
}

#[test]
fn test_13_built_in_visitors_lifecycle_dfs() {
    let mut tree = build_dfs_tree();
    assert!(!tree.is_complete());

    // AutoSelectFirst on multi-level tree satisfies all choices
    tree.walk_choices(&mut AutoSelectFirst);
    assert!(tree.is_complete());

    let resolved_initial = tree
        .clone()
        .resolve()
        .expect("Tree should resolve after AutoSelectFirst");
    assert_eq!(resolved_initial.choice, "r0_val");
    assert_eq!(resolved_initial.branch_a.choice, "c1_val");
    assert_eq!(resolved_initial.branch_a.leaf1.choice, "l1_1_val");
    assert_eq!(resolved_initial.branch_a.leaf2.choice, "l1_2_val");
    assert_eq!(resolved_initial.branch_b.choice, "c2_val");
    assert_eq!(resolved_initial.branch_b.leaf1.choice, "l2_1_val");
    assert_eq!(resolved_initial.branch_b.leaf2.choice, "l2_2_val");

    // unmake_all() reverts choices to UnmadeWithHistory, making tree incomplete
    tree.unmake_all();
    assert!(!tree.is_complete());

    assert!(matches!(
        tree.choice.as_choice().unwrap().state,
        ChoiceState::UnmadeWithHistory(_)
    ));

    // confirm_all() restores choices to Made, making tree complete again
    tree.confirm_all();
    assert!(tree.is_complete());

    assert!(matches!(
        tree.choice.as_choice().unwrap().state,
        ChoiceState::Made(_)
    ));

    // Resolves to the exact same domain data
    let resolved_restored = tree
        .resolve()
        .expect("Tree should resolve after confirm_all");
    assert_eq!(resolved_initial, resolved_restored);
}

// ---------------------------------------------------------------------------
// Traversal Tests: Cascading Collection and Pick Verification
// ---------------------------------------------------------------------------

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct CascadingChildBfs {
    #[slot]
    pub leaf: String,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct CascadingParentBfs {
    #[slot]
    pub root_leaf: String,
    #[slot(collection, cascade)]
    pub coll: Vec<CascadingChildBfs>,
    #[slot(pick, cascade)]
    pub pick: Vec<CascadingChildBfs>,
}

#[test]
fn test_bfs_with_cascading_collection_and_pick() {
    let child1 = CascadingChildBfs::builder()
        .leaf(SingleChoice::new("CollChild1", vec!["c1".to_string()]))
        .build();

    let child2 = CascadingChildBfs::builder()
        .leaf(SingleChoice::new("CollChild2", vec!["c2".to_string()]))
        .build();

    let coll_choice = SingleChoice::new("CollChoice2", vec![child2]);

    let pick_cand1 = CascadingChildBfs::builder()
        .leaf(SingleChoice::new("PickChild1", vec!["p1".to_string()]))
        .build();

    let pick_cand2 = CascadingChildBfs::builder()
        .leaf(SingleChoice::new("PickChild2", vec!["p2".to_string()]))
        .build();

    let pick_choice = MultiChoice::new("PickChoice", vec![pick_cand1, pick_cand2])
        .with_cardinality(formwork::choice::Cardinality::Exact(1));

    let mut parent = CascadingParentBfs::builder()
        .root_leaf(SingleChoice::new("RootTitle", vec!["root".to_string()]))
        .with_coll(child1)
        .with_coll(coll_choice)
        .pick(pick_choice)
        .build();

    // Select candidate 0 in pick_choice and candidate 0 in coll_choice so children exist in selected
    if let Some(c) = parent.coll[1].as_choice_mut() {
        c.select(0).unwrap();
        c.confirm().unwrap();
    }
    if let Some(p) = parent.pick.as_choice_mut() {
        p.select(0).unwrap();
        p.confirm().unwrap();
    }

    let mut visitor = TracingVisitor::default();
    parent.walk_choices(&mut visitor);

    // Level-order BFS visits:
    // Level 0: "RootTitle", "CollChoice2", "PickChoice"
    // Level 1: "CollChild1", "CollChild2", "PickChild1"
    assert_eq!(
        visitor.visited,
        vec![
            "RootTitle",
            "CollChoice2",
            "PickChoice",
            "CollChild1",
            "CollChild2",
            "PickChild1",
        ]
    );
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct CascadingChildDfs {
    #[slot]
    pub leaf: String,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct CascadingParentDfs {
    #[slot]
    pub root_leaf: String,
    #[slot(collection, cascade)]
    pub coll: Vec<CascadingChildDfs>,
    #[slot(pick, cascade)]
    pub pick: Vec<CascadingChildDfs>,
}

#[test]
fn test_dfs_with_cascading_collection_and_pick() {
    let child1 = CascadingChildDfs::builder()
        .leaf(SingleChoice::new("CollChild1", vec!["c1".to_string()]))
        .build();

    let child2 = CascadingChildDfs::builder()
        .leaf(SingleChoice::new("CollChild2", vec!["c2".to_string()]))
        .build();

    let coll_choice = SingleChoice::new("CollChoice2", vec![child2]);

    let pick_cand1 = CascadingChildDfs::builder()
        .leaf(SingleChoice::new("PickChild1", vec!["p1".to_string()]))
        .build();

    let pick_cand2 = CascadingChildDfs::builder()
        .leaf(SingleChoice::new("PickChild2", vec!["p2".to_string()]))
        .build();

    let pick_choice = MultiChoice::new("PickChoice", vec![pick_cand1, pick_cand2])
        .with_cardinality(formwork::choice::Cardinality::Exact(1));

    let mut parent = CascadingParentDfs::builder()
        .root_leaf(SingleChoice::new("RootTitle", vec!["root".to_string()]))
        .with_coll(child1)
        .with_coll(coll_choice)
        .pick(pick_choice)
        .build();

    // Select candidate 0 in pick_choice and candidate 0 in coll_choice so children exist in selected
    if let Some(c) = parent.coll[1].as_choice_mut() {
        c.select(0).unwrap();
        c.confirm().unwrap();
    }
    if let Some(p) = parent.pick.as_choice_mut() {
        p.select(0).unwrap();
        p.confirm().unwrap();
    }

    let mut visitor = TracingVisitor::default();
    parent.walk_choices(&mut visitor);

    // Depth-first DFS visits:
    // Root title -> coll choices (CollChoice2) -> coll cascading children (CollChild1, CollChild2) -> pick choices (PickChoice) -> pick cascading children (PickChild1)
    // Crucially: all coll children are visited before pick is visited!
    assert_eq!(
        visitor.visited,
        vec![
            "RootTitle",
            "CollChoice2",
            "CollChild1",
            "CollChild2",
            "PickChoice",
            "PickChild1",
        ]
    );
}
