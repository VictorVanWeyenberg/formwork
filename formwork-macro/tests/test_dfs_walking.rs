use formwork::choice::{MultiChoice, SingleChoice};
use formwork::slot::SingleSlot;
use formwork::{ChoiceVisitor, Formwork, VisitorFlow};
use formwork_macro::formwork;

// 1. Node with ONLY choices (zero children)
#[formwork(traverse = "dfs")]
struct Leaf {
    #[slot]
    c1: String,

    #[slot]
    c2: String,
}

// 2. Node with BOTH choices and children
#[formwork(traverse = "dfs")]
struct Node {
    #[slot]
    n1: String,

    #[slot(cascade)]
    child: Leaf,

    #[slot]
    n2: String,
}

// 3. Node with ONLY children (zero choices), testing optional/collection variants
#[formwork(traverse = "dfs")]
struct Root {
    #[slot(cascade)]
    left: Node,

    #[slot(cascade)]
    right: Node,

    #[slot(collection, cascade)]
    list: Vec<Node>,
}

// Visitor to record DFS order and mutate the items
struct TestVisitor {
    visited_order: Vec<String>,
}

impl TestVisitor {
    fn new() -> Self {
        Self {
            visited_order: Vec::new(),
        }
    }
}

impl ChoiceVisitor<String> for TestVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
        // Collect current value from the first option
        let val = choice.options[0].clone();
        self.visited_order.push(val.clone());

        // Test mutation: update the choice's value in-place
        choice.options[0] = format!("{}_mutated", val);

        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<String>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<NodeFormwork> for TestVisitor {
    fn visit_single(&mut self, _choice: &mut SingleChoice<NodeFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }
    fn visit_multi(&mut self, _choice: &mut MultiChoice<NodeFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<LeafFormwork> for TestVisitor {
    fn visit_single(&mut self, _choice: &mut SingleChoice<LeafFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }
    fn visit_multi(&mut self, _choice: &mut MultiChoice<LeafFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

#[test]
fn test_walk_choices_dfs_and_mutation() {
    // Initialize the auto-generated formwork structs directly
    let mut root = RootFormwork {
        left: SingleSlot::FixedWet(NodeFormwork {
            n1: SingleChoice::new("n1", vec!["left.n1".to_string()]).into(),
            child: SingleSlot::FixedWet(LeafFormwork {
                c1: SingleChoice::new("c1", vec!["left.child.c1".to_string()]).into(),
                c2: SingleChoice::new("c2", vec!["left.child.c2".to_string()]).into(),
            }),
            n2: SingleChoice::new("n2", vec!["left.n2".to_string()]).into(),
        }),
        right: SingleSlot::FixedWet(NodeFormwork {
            n1: SingleChoice::new("n1", vec!["right.n1".to_string()]).into(),
            child: SingleSlot::FixedWet(LeafFormwork {
                c1: SingleChoice::new("c1", vec!["right.child.c1".to_string()]).into(),
                c2: SingleChoice::new("c2", vec!["right.child.c2".to_string()]).into(),
            }),
            n2: SingleChoice::new("n2", vec!["right.n2".to_string()]).into(),
        }),
        list: vec![SingleSlot::FixedWet(NodeFormwork {
            n1: SingleChoice::new("n1", vec!["list0.n1".to_string()]).into(),
            child: SingleSlot::FixedWet(LeafFormwork {
                c1: SingleChoice::new("c1", vec!["list0.child.c1".to_string()]).into(),
                c2: SingleChoice::new("c2", vec!["list0.child.c2".to_string()]).into(),
            }),
            n2: SingleChoice::new("n2", vec!["list0.n2".to_string()]).into(),
        })],
    };

    let mut visitor = TestVisitor::new();
    root.walk_choices(&mut visitor);

    // 1. Verify TRUE DFS ordering
    let expected_order = vec![
        "left.n1",
        "left.child.c1",
        "left.child.c2",
        "left.n2",
        "right.n1",
        "right.child.c1",
        "right.child.c2",
        "right.n2",
        "list0.n1",
        "list0.child.c1",
        "list0.child.c2",
        "list0.n2",
    ];

    assert_eq!(
        visitor.visited_order, expected_order,
        "DFS ordering across branches is incorrect"
    );

    // 2. Verify Mutation Persisted
    let mut check_visitor = TestVisitor::new();
    root.walk_choices(&mut check_visitor);

    let expected_mutated_order: Vec<String> = expected_order
        .into_iter()
        .map(|s| format!("{}_mutated", s))
        .collect();

    assert_eq!(
        check_visitor.visited_order, expected_mutated_order,
        "Mutation via ChoiceVisitor was not correctly applied or persisted"
    );
}
