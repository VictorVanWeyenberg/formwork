use formwork::choice::{MultiChoice, SingleChoice};
use formwork::visitor::{ChoiceVisitor, VisitorFlow};
use formwork::{Formwork, formwork};

// ----------------------------------------------------------------------------
// 1. Struct with mixed fields (slot and non-slot)
// ----------------------------------------------------------------------------

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct MixedStruct {
    pub name: String,
    pub count: usize,
    pub tags: Vec<String>,
    #[slot]
    pub active_tag: String,
}

#[test]
fn test_mixed_struct_behavior() {
    let mixed_bp = MixedStruct::builder()
        .name("test")
        .count(42usize)
        .with_tags("rust")
        .active_tag(SingleChoice::new("select tag", vec!["tag1".to_string(), "tag2".to_string()]))
        .build();

    assert!(!mixed_bp.is_complete());

    struct ActiveTagVisitor;
    impl ChoiceVisitor<String> for ActiveTagVisitor {
        fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
            let _ = choice.select(0);
            let _ = choice.confirm();
            VisitorFlow::Continue
        }

        fn visit_multi(&mut self, _choice: &mut MultiChoice<String>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    let mut bp = mixed_bp;
    bp.walk_choices(&mut ActiveTagVisitor);
    assert!(bp.is_complete());

    let res = bp.resolve().unwrap();
    assert_eq!(res.name, "test");
    assert_eq!(res.count, 42);
    assert_eq!(res.tags, vec!["rust"]);
    assert_eq!(res.active_tag, "tag1");
}

// ----------------------------------------------------------------------------
// 2. Traversal interleaving: BFS parent with DFS child and DFS grandchild
// ----------------------------------------------------------------------------

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepDfsGrandchild {
    #[slot]
    pub gc_name: String,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct IntermediateDfsChild {
    #[slot]
    pub child_name: String,
    #[slot(cascade)]
    pub grandchild: DeepDfsGrandchild,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct TopBfsParent {
    #[slot]
    pub parent_name: String,
    #[slot(cascade)]
    pub child: IntermediateDfsChild,
    #[slot(collection, cascade)]
    pub coll_children: Vec<IntermediateDfsChild>,
    #[slot(pick, cascade)]
    pub pick_children: Vec<IntermediateDfsChild>,
}

struct TracingVisitor {
    visited: Vec<String>,
    break_on: Option<String>,
}

impl ChoiceVisitor<String> for TracingVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
        let label = choice.options().join(",");
        self.visited.push(label.clone());
        if self.break_on.as_ref() == Some(&label) {
            return VisitorFlow::Break;
        }
        let _ = choice.select(0);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<String>) -> VisitorFlow {
        let label = choice.options().join(",");
        self.visited.push(label.clone());
        if self.break_on.as_ref() == Some(&label) {
            return VisitorFlow::Break;
        }
        let _ = choice.select(0);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<IntermediateDfsChildFormwork> for TracingVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<IntermediateDfsChildFormwork>) -> VisitorFlow {
        let _ = choice.select(0);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<IntermediateDfsChildFormwork>) -> VisitorFlow {
        let _ = choice.select(0);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<DeepDfsGrandchildFormwork> for TracingVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<DeepDfsGrandchildFormwork>) -> VisitorFlow {
        let _ = choice.select(0);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<DeepDfsGrandchildFormwork>) -> VisitorFlow {
        let _ = choice.select(0);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

#[test]
fn test_mixed_bfs_dfs_traversal_and_break_propagation() {
    let make_grandchild = |id: &str| {
        DeepDfsGrandchild::builder()
            .gc_name(SingleChoice::new("choose gc", vec![format!("gc_{}", id)]))
            .build()
    };

    let make_child = |id: &str| {
        IntermediateDfsChild::builder()
            .child_name(SingleChoice::new("choose child", vec![format!("child_{}", id)]))
            .grandchild(make_grandchild(id))
            .build()
    };

    let mut parent = TopBfsParent::builder()
        .parent_name(SingleChoice::new("choose parent", vec!["parent_choice".to_string()]))
        .child(make_child("single"))
        .with_coll_children(make_child("coll_0"))
        .with_coll_children(make_child("coll_1"))
        .pick_children(vec![make_child("pick_0")])
        .build();

    let mut visitor = TracingVisitor {
        visited: Vec::new(),
        break_on: None,
    };

    let flow = parent.walk_choices(&mut visitor);
    assert_eq!(flow, VisitorFlow::Continue);
    // Verify that parent choice was visited first (BFS level 0)
    assert_eq!(visitor.visited[0], "parent_choice");

    // Now test break propagation at child level
    let mut parent2 = TopBfsParent::builder()
        .parent_name(SingleChoice::new("choose p2", vec!["p_break".to_string()]))
        .child(make_child("break_target"))
        .pick_children(Vec::<IntermediateDfsChildFormwork>::new())
        .build();

    let mut break_visitor = TracingVisitor {
        visited: Vec::new(),
        break_on: Some("child_break_target".to_string()),
    };

    let break_flow = parent2.walk_choices(&mut break_visitor);
    assert_eq!(break_flow, VisitorFlow::Break);
    assert_eq!(break_visitor.visited, vec!["p_break", "child_break_target"]);
}

#[test]
fn test_full_tree_resolution_after_walk() {
    let make_grandchild = |id: &str| {
        DeepDfsGrandchild::builder()
            .gc_name(SingleChoice::new("choose gc", vec![format!("resolved_gc_{}", id)]))
            .build()
    };

    let make_child = |id: &str| {
        IntermediateDfsChild::builder()
            .child_name(SingleChoice::new("choose child", vec![format!("resolved_child_{}", id)]))
            .grandchild(make_grandchild(id))
            .build()
    };

    let mut parent = TopBfsParent::builder()
        .parent_name(SingleChoice::new("choose parent", vec!["resolved_parent".to_string()]))
        .child(make_child("1"))
        .with_coll_children(make_child("2"))
        .pick_children(vec![make_child("3")])
        .build();

    assert!(!parent.is_complete());

    let mut visitor = TracingVisitor {
        visited: Vec::new(),
        break_on: None,
    };
    parent.walk_choices(&mut visitor);

    assert!(parent.is_complete());
    let resolved = parent.resolve().unwrap();
    assert_eq!(resolved.parent_name, "resolved_parent");
    assert_eq!(resolved.child.child_name, "resolved_child_1");
    assert_eq!(resolved.child.grandchild.gc_name, "resolved_gc_1");
    assert_eq!(resolved.coll_children[0].child_name, "resolved_child_2");
    assert_eq!(resolved.coll_children[0].grandchild.gc_name, "resolved_gc_2");
    assert_eq!(resolved.pick_children[0].child_name, "resolved_child_3");
    assert_eq!(resolved.pick_children[0].grandchild.gc_name, "resolved_gc_3");
}


// ----------------------------------------------------------------------------
// 4. Cascading pick with arrays [T; N] and Vec<T>
// ----------------------------------------------------------------------------

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ChildEntity {
    #[slot]
    pub val: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ParentPickEntity {
    #[slot(pick, cascade)]
    pub children: Vec<ChildEntity>,
}

#[test]
fn test_cascading_pick_array_setter() {
    let c1 = ChildEntity::builder()
        .val(SingleChoice::new("choose val", vec!["v1".to_string()]))
        .build();
    let c2 = ChildEntity::builder()
        .val(SingleChoice::new("choose val", vec!["v2".to_string()]))
        .build();

    // Passing array of formworks [ChildEntityFormwork; 2] to cascading pick builder
    let mut parent = ParentPickEntity::builder()
        .children([c1, c2])
        .build();

    assert!(!parent.is_complete());

    struct ChildValVisitor;
    impl ChoiceVisitor<String> for ChildValVisitor {
        fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
            let _ = choice.select(0);
            let _ = choice.confirm();
            VisitorFlow::Continue
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<String>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    impl ChoiceVisitor<ChildEntityFormwork> for ChildValVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<ChildEntityFormwork>) -> VisitorFlow {
            VisitorFlow::Continue
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<ChildEntityFormwork>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    let flow = parent.walk_choices(&mut ChildValVisitor);
    assert_eq!(flow, VisitorFlow::Continue);
    assert!(parent.is_complete());

    let res = parent.resolve().unwrap();
    assert_eq!(res.children.len(), 2);
    assert_eq!(res.children[0].val, "v1");
    assert_eq!(res.children[1].val, "v2");
}

#[test]
fn test_cascading_pick_resolved_child_array_setter() {
    let resolved_c1 = ChildEntity { val: "c1_done".to_string() };
    let resolved_c2 = ChildEntity { val: "c2_done".to_string() };

    // Passing array of already resolved children [ChildEntity; 2]
    let parent = ParentPickEntity::builder()
        .children([resolved_c1.clone(), resolved_c2.clone()])
        .build();

    assert!(parent.is_complete());
    let res = parent.resolve().unwrap();
    assert_eq!(res.children, vec![resolved_c1, resolved_c2]);
}
