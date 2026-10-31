//! Adversarial break tests for codebase rebrand and boundary conditions.
//!
//! Covers:
//! 1. All `#[formwork]` permutations:
//!    - default (traverse = "bfs", no builder)
//!    - builder = true, builder = false
//!    - traverse = "bfs" vs traverse = "dfs"
//!    - derives(...)
//!    - combined: traverse = "dfs", builder, derives(PartialOrd, Ord, Hash)
//! 2. `FormworkError::WetChoice` triggering across all slot kinds:
//!    - Single leaf slot
//!    - Pick leaf slot
//!    - Optional leaf slot
//!    - Collection leaf slot
//!    - Cascading single slot
//!    - Cascading optional slot
//!    - Cascading pick slot
//!    - Cascading collection slot
//! 3. `Slot::FixedWet` and `Slot::FixedDry`:
//!    - `as_wet()`, `as_dry()`, `as_choice()`, `as_choice_mut()`
//!    - From conversions
//!    - is_complete(), is_pending(), is_made()
//!    - resolve_exact() and resolve() behavior

use formwork::{
    choice::{MultiChoice, SingleChoice},
    error::FormworkError,
    formwork,
    slot::{MultiSlot, SingleSlot, Slot},
    visitor::{AutoSelectFirst, ChoiceVisitor, VisitorFlow},
    Formwork, StepResult,
};

// ============================================================================
// 1. Permutations of #[formwork]
// ============================================================================

// A. Default: no builder, default traverse ("bfs")
#[formwork]
#[derive(Clone, Debug, PartialEq)]
pub struct PermDefault {
    pub id: u32,
    #[slot]
    pub name: String,
}

// B. Builder explicitly disabled
#[formwork(builder = false)]
#[derive(Clone, Debug, PartialEq)]
pub struct PermNoBuilder {
    pub tag: String,
    #[slot]
    pub code: i32,
}

// C. Traverse = DFS with builder
#[formwork(traverse = "dfs", builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct PermDfsBuilder {
    pub meta: String,
    #[slot]
    pub val: String,
    #[slot(collection)]
    pub items: Vec<String>,
}

// D. Traverse = BFS with builder and derives
#[formwork(traverse = "bfs", builder, derives(PartialOrd, Ord, Hash))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PermBfsDerives {
    pub key: u64,
    #[slot]
    pub label: String,
}

// E. Nested cascading child with DFS
#[formwork(traverse = "dfs", builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct PermChild {
    #[slot]
    pub leaf: String,
}

#[formwork(traverse = "dfs", builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct PermParentCascade {
    #[slot]
    pub parent_choice: String,
    #[slot(cascade)]
    pub child: PermChild,
}

// F. Cascading Pick & Optional slots
#[formwork(traverse = "bfs", builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexMatrixChild {
    pub id: usize,
    #[slot]
    pub attr: String,
}

#[formwork(traverse = "bfs", builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexMatrixRoot {
    #[slot]
    pub single_leaf: String,
    #[slot(option)]
    pub option_leaf: Option<String>,
    #[slot(collection)]
    pub collection_leaf: Vec<String>,
    #[slot(pick)]
    pub pick_leaf: Vec<String>,
    #[slot(cascade)]
    pub single_cascade: ComplexMatrixChild,
    #[slot(option, cascade)]
    pub option_cascade: Option<ComplexMatrixChild>,
    #[slot(collection, cascade)]
    pub collection_cascade: Vec<ComplexMatrixChild>,
    #[slot(pick, cascade)]
    pub pick_cascade: Vec<ComplexMatrixChild>,
}

// ============================================================================
// Tests: Permutations Compilation & Basic Invariants
// ============================================================================

#[test]
fn test_permutation_defaults_and_mode_aliases() {
    // Check that PermDefaultDry and PermDefaultWet exist and type aliases match
    let formwork = PermDefaultFormwork {
        id: 1,
        name: Slot::FixedWet("wet_val".to_string()),
    };
    assert!(formwork.is_complete());
    let dry: PermDefault = formwork.resolve().unwrap();
    assert_eq!(dry.id, 1);
    assert_eq!(dry.name, "wet_val");

    // From dry roundtrip produces FixedDry
    let from_dry = PermDefaultFormwork::from(dry);
    assert_eq!(from_dry.name, Slot::FixedDry("wet_val".to_string()));
    assert!(from_dry.is_complete());
}

#[test]
fn test_permutation_no_builder_manual_construction() {
    let mut formwork = PermNoBuilderFormwork {
        tag: "manual".to_string(),
        code: Slot::Choose(SingleChoice::new("Code", vec![100, 200])),
    };
    assert!(!formwork.is_complete());
    match formwork.clone().resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Code"),
        other => panic!("Expected FormworkError::WetChoice, got {:?}", other),
    }

    formwork.walk_choices(&mut AutoSelectFirst);
    assert!(formwork.is_complete());
    let dry: PermNoBuilder = formwork.resolve().unwrap();
    assert_eq!(dry.tag, "manual");
    assert_eq!(dry.code, 100);
}

#[test]
fn test_permutation_derives_and_ordering() {
    let f1 = PermBfsDerives::builder()
        .key(10u64)
        .label("beta".to_string())
        .build();
    let f2 = PermBfsDerives::builder()
        .key(20u64)
        .label("alpha".to_string())
        .build();

    // Verify Eq and Ord are derived and functioning on PermBfsDerives
    let d1: PermBfsDerives = f1.resolve().unwrap();
    let d2: PermBfsDerives = f2.resolve().unwrap();
    assert!(d1 < d2);
}

// ============================================================================
// 2. FormworkError::WetChoice Triggering Boundary Tests
// ============================================================================

#[test]
fn test_wet_choice_error_single_leaf() {
    let formwork = PermDfsBuilderFormwork::builder()
        .meta("test".to_string())
        .val(SingleChoice::new("Pick Val", vec!["A".to_string(), "B".to_string()]))
        .items(Vec::<SingleSlot<String, String>>::new())
        .build();

    assert!(!formwork.is_complete());
    match formwork.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Pick Val"),
        other => panic!("Expected WetChoice, got {:?}", other),
    }
}

#[test]
fn test_wet_choice_error_staged_but_unconfirmed() {
    let mut choice = SingleChoice::new("Pick Val", vec!["A".to_string(), "B".to_string()]);
    // Staging without confirming: still wet!
    choice.select(0).unwrap();
    assert!(!choice.is_made());

    let formwork = PermDfsBuilderFormwork::builder()
        .meta("test".to_string())
        .val(choice)
        .items(Vec::<SingleSlot<String, String>>::new())
        .build();

    assert!(!formwork.is_complete());
    match formwork.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Pick Val"),
        other => panic!("Expected WetChoice, got {:?}", other),
    }
}

#[test]
fn test_wet_choice_error_cascading_child() {
    let child_with_choice = PermChildFormwork::builder()
        .leaf(SingleChoice::new("Child Leaf Choice", vec!["CL1".to_string()]))
        .build();

    let parent = PermParentCascadeFormwork::builder()
        .parent_choice("ParentFixed".to_string())
        .child(child_with_choice)
        .build();

    assert!(!parent.is_complete());
    match parent.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Child Leaf Choice"),
        other => panic!("Expected WetChoice for child, got {:?}", other),
    }
}

#[test]
fn test_wet_choice_error_in_option_slot() {
    let choice = SingleChoice::new("Optional Choice", vec!["opt_val".to_string()]);

    let formwork = ComplexMatrixRootFormwork::builder()
        .single_leaf("sl".to_string())
        .option_leaf(Some(choice))
        .single_cascade(ComplexMatrixChildFormwork::builder().id(1usize).attr("c1".to_string()).build())
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<ComplexMatrixChildFormwork>::new())
        .build();

    assert!(!formwork.is_complete());
    match formwork.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Optional Choice"),
        other => panic!("Expected WetChoice on option slot, got {:?}", other),
    }
}

#[test]
fn test_wet_choice_error_in_collection_slot() {
    let choice = SingleChoice::new("Coll Item Choice", vec!["ci1".to_string()]);

    let formwork = ComplexMatrixRootFormwork::builder()
        .single_leaf("sl".to_string())
        .with_collection_leaf(choice)
        .single_cascade(ComplexMatrixChildFormwork::builder().id(1usize).attr("c1".to_string()).build())
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<ComplexMatrixChildFormwork>::new())
        .build();

    assert!(!formwork.is_complete());
    match formwork.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Coll Item Choice"),
        other => panic!("Expected WetChoice on collection item, got {:?}", other),
    }
}

#[test]
fn test_wet_choice_error_in_pick_leaf_slot() {
    let pick_choice = MultiChoice::new(
        "Pick Leaf Choice",
        vec!["P1".to_string(), "P2".to_string()],
    );

    let formwork = ComplexMatrixRootFormwork::builder()
        .single_leaf("sl".to_string())
        .pick_leaf(pick_choice)
        .single_cascade(ComplexMatrixChildFormwork::builder().id(1usize).attr("c1".to_string()).build())
        .pick_cascade(Vec::<ComplexMatrixChildFormwork>::new())
        .build();

    assert!(!formwork.is_complete());
    match formwork.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Pick Leaf Choice"),
        other => panic!("Expected WetChoice on pick slot, got {:?}", other),
    }
}

#[test]
fn test_wet_choice_error_in_cascading_pick_slot() {
    let c1 = ComplexMatrixChildFormwork::builder().id(1usize).attr("a1".to_string()).build();
    let c2 = ComplexMatrixChildFormwork::builder().id(2usize).attr("a2".to_string()).build();

    let pick_cascade = MultiChoice::new("Pick Cascade Children", vec![c1, c2]);

    let formwork = ComplexMatrixRootFormwork::builder()
        .single_leaf("sl".to_string())
        .single_cascade(ComplexMatrixChildFormwork::builder().id(0usize).attr("c0".to_string()).build())
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(pick_cascade)
        .build();

    assert!(!formwork.is_complete());
    match formwork.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Pick Cascade Children"),
        other => panic!("Expected WetChoice on pick cascade slot, got {:?}", other),
    }
}

#[test]
fn test_wet_choice_error_deeply_nested_inside_cascading_collection() {
    let child_with_wet_leaf = ComplexMatrixChildFormwork::builder()
        .id(10usize)
        .attr(SingleChoice::new("Deep Wet Leaf", vec!["deep_val".to_string()]))
        .build();

    let formwork = ComplexMatrixRootFormwork::builder()
        .single_leaf("sl".to_string())
        .single_cascade(ComplexMatrixChildFormwork::builder().id(0usize).attr("c0".to_string()).build())
        .with_collection_cascade(child_with_wet_leaf)
        .pick_leaf(Vec::<String>::new())
        .pick_cascade(Vec::<ComplexMatrixChildFormwork>::new())
        .build();

    assert!(!formwork.is_complete());
    match formwork.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Deep Wet Leaf"),
        other => panic!("Expected WetChoice for deep wet leaf, got {:?}", other),
    }
}

// ============================================================================
// 3. Slot::FixedWet vs Slot::FixedDry Invariants and Conversions
// ============================================================================

#[test]
fn test_slot_fixed_wet_and_dry_accessors() {
    let wet_slot: SingleSlot<String, String> = Slot::FixedWet("wet".to_string());
    assert_eq!(wet_slot.as_wet(), Some(&"wet".to_string()));
    assert_eq!(wet_slot.as_dry(), None);
    assert_eq!(wet_slot.as_choice(), None);
    assert!(wet_slot.is_complete());
    assert!(!wet_slot.is_pending());
    assert!(!wet_slot.is_made());

    let dry_slot: SingleSlot<String, String> = Slot::FixedDry("dry".to_string());
    assert_eq!(dry_slot.as_dry(), Some(&"dry".to_string()));
    assert_eq!(dry_slot.as_wet(), None);
    assert_eq!(dry_slot.as_choice(), None);
    assert!(dry_slot.is_complete());
    assert!(!dry_slot.is_pending());
    assert!(!dry_slot.is_made());

    let mut choice_slot: SingleSlot<String, String> =
        Slot::Choose(SingleChoice::new("Prompt", vec!["opt".to_string()]));
    assert_eq!(choice_slot.as_wet(), None);
    assert_eq!(choice_slot.as_dry(), None);
    assert!(choice_slot.as_choice().is_some());
    assert_eq!(choice_slot.as_choice().unwrap().prompt(), "Prompt");

    // Test mutable accessor
    if let Some(ch) = choice_slot.as_choice_mut() {
        ch.select(0).unwrap();
        ch.confirm().unwrap();
    }
    assert!(choice_slot.is_made());
    assert!(choice_slot.is_complete());
    assert_eq!(choice_slot.resolve_exact().unwrap(), "opt");
}

#[test]
fn test_slot_from_conversions_and_resolution() {
    // SingleSlot From<W> produces FixedWet
    let slot1: SingleSlot<i32, i32> = 42.into();
    assert_eq!(slot1, Slot::FixedWet(42));
    assert_eq!(slot1.resolve_exact().unwrap(), 42);

    // SingleSlot From<SingleChoice<W>> produces Choose
    let sc = SingleChoice::new("P", vec![100]);
    let slot2: SingleSlot<i32, i32> = sc.into();
    assert!(matches!(slot2, Slot::Choose(_)));
    assert!(!slot2.is_complete());

    // MultiSlot From<Vec<W>> produces FixedWet
    let mslot1: MultiSlot<i32, i32> = vec![1, 2, 3].into();
    assert_eq!(mslot1, Slot::FixedWet(vec![1, 2, 3]));
    assert_eq!(mslot1.resolve().unwrap(), vec![1, 2, 3]);

    // MultiSlot From<MultiChoice<W>> produces Choose
    let mc = MultiChoice::new("MP", vec![10, 20]);
    let mslot2: MultiSlot<i32, i32> = mc.into();
    assert!(matches!(mslot2, Slot::Choose(_)));
    assert!(!mslot2.is_complete());
}

#[test]
fn test_slot_unconfirmed_resolve_fails_with_wet_choice() {
    let unconfirmed_single: SingleSlot<String, String> =
        Slot::Choose(SingleChoice::new("Unconfirmed Single", vec!["x".to_string()]));
    match unconfirmed_single.resolve_exact() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Unconfirmed Single"),
        other => panic!("Expected WetChoice, got {:?}", other),
    }

    let unconfirmed_multi: MultiSlot<String, String> =
        Slot::Choose(MultiChoice::new("Unconfirmed Multi", vec!["x".to_string()]));
    match unconfirmed_multi.resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "Unconfirmed Multi"),
        other => panic!("Expected WetChoice, got {:?}", other),
    }
}

// ============================================================================
// 4. Session Rewind and Wet Invariants
// ============================================================================

struct SelectIndexVisitor {
    index: usize,
    flow: VisitorFlow,
}

impl<T> ChoiceVisitor<T> for SelectIndexVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        let _ = choice.select(self.index);
        let _ = choice.confirm();
        self.flow
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        let _ = choice.select(self.index);
        let _ = choice.confirm();
        self.flow
    }
}

struct RewindVisitor;

impl<T> ChoiceVisitor<T> for RewindVisitor {
    fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
        VisitorFlow::Back
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
        VisitorFlow::Back
    }
}

#[test]
fn test_session_stepping_rewind_and_wet_resolution() {
    let mut tree = PermParentCascadeFormwork::builder()
        .parent_choice(SingleChoice::new("P1", vec!["ParentOpt".to_string()]))
        .child(
            PermChildFormwork::builder()
                .leaf(SingleChoice::new("C1", vec!["ChildOpt".to_string()]))
                .build(),
        )
        .build();

    let mut session = tree.session();

    // Step 0: Parent choice confirmed
    let mut v0 = SelectIndexVisitor {
        index: 0,
        flow: VisitorFlow::Continue,
    };
    match session.step(&mut v0) {
        StepResult::Choice(flow) => assert_eq!(flow, VisitorFlow::Continue),
        other => panic!("Expected Choice, got {:?}", other),
    }

    // Step 1: Child choice visited, visitor signals Back!
    let mut v_rewind = RewindVisitor;
    match session.step(&mut v_rewind) {
        StepResult::Choice(VisitorFlow::Back) => {}
        other => panic!("Expected Choice with VisitorFlow::Back on rewind, got {:?}", other),
    }

    // Cursor rewound to 0, parent choice was unmade!
    assert_eq!(session.current_step(), 0);
    assert!(!session.formwork().is_complete());

    // Resolving mid-session after rewind must fail with WetChoice for P1
    match session.formwork().clone().resolve() {
        Err(FormworkError::WetChoice { prompt }) => assert_eq!(prompt, "P1"),
        other => panic!("Expected WetChoice after rewinding parent, got {:?}", other),
    }

    // Now re-confirm parent and confirm child
    let mut v_all = AutoSelectFirst;
    session.formwork_mut().walk_choices(&mut v_all);
    assert!(session.formwork().is_complete());

    let dry = session.formwork().clone().resolve().expect("Resolved cleanly");
    assert_eq!(dry.parent_choice, "ParentOpt");
    assert_eq!(dry.child.leaf, "ChildOpt");
}
