//! Adversarial test suite for slot type preservation, scalar vector slots,
//! nested vector containers, and string ergonomics boundaries.

use formwork::{
    choice::{MultiChoice, SingleChoice},
    error::FormworkError,
    formwork,
    visitor::{AutoSelectFirst, ChoiceVisitor, VisitorFlow},
    Formwork,
};

// ============================================================================
// 1. Scalar slot holding Vec<i32>
// ============================================================================

#[formwork(
    builder,
    derives(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScalarIntVecTarget {
    pub tag: String,
    #[slot]
    pub numbers: Vec<i32>,
}

#[test]
fn test_scalar_int_vec_raw_and_resolved() {
    // 1. Raw Vec<i32> in wet mode
    let bp = ScalarIntVecTargetFormwork::builder()
        .tag("integers".to_string())
        .numbers(vec![1, 2, 3, 4])
        .build();
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve");
    assert_eq!(resolved.tag, "integers");
    assert_eq!(resolved.numbers, vec![1, 2, 3, 4]);

    // 2. Direct Resolved mode builder
    let direct_resolved: ScalarIntVecTarget = ScalarIntVecTarget::builder()
        .tag("integers".to_string())
        .numbers(vec![10, 20, 30])
        .build();
    assert_eq!(direct_resolved.numbers, vec![10, 20, 30]);
}

#[test]
fn test_scalar_int_vec_choice_lifecycle_and_traversal() {
    let choice = SingleChoice::new(
        "Choose number set",
        vec![vec![1, 2], vec![3, 4, 5], vec![6]],
    );
    let mut bp = ScalarIntVecTargetFormwork::builder()
        .tag("numbers".to_string())
        .numbers(choice)
        .build();

    assert!(!bp.is_complete());

    // Resolve before choice resolution must fail
    match bp.clone().resolve() {
        Err(FormworkError::WetChoice { prompt }) => {
            assert_eq!(prompt, "Choose number set");
        }
        other => panic!("Expected WetChoice, got {:?}", other),
    }

    // Manual candidate staging and confirmation
    bp.numbers.as_choice_mut().unwrap().select(1).unwrap();
    assert!(!bp.is_complete(), "select() only stages; choice remains unconfirmed");

    bp.numbers.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve after confirmation");
    assert_eq!(resolved.numbers, vec![3, 4, 5]);

    // Test AutoSelectFirst traversal
    let mut bp_auto = ScalarIntVecTargetFormwork::builder()
        .tag("auto".to_string())
        .numbers(SingleChoice::new("Auto set", vec![vec![100], vec![200]]))
        .build();
    assert!(!bp_auto.is_complete());
    bp_auto.walk_choices(&mut AutoSelectFirst);
    assert!(bp_auto.is_complete());
    assert_eq!(bp_auto.resolve().unwrap().numbers, vec![100]);
}

// ============================================================================
// 2. Scalar slot holding Vec<String> & String Ergonomics Isolation
// ============================================================================

#[formwork(
    builder,
    derives(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScalarStringVecTarget {
    #[slot]
    pub texts: Vec<String>,
}

#[test]
fn test_scalar_string_vec_slot_preservation() {
    let choice = SingleChoice::new(
        "Choose texts",
        vec![
            vec!["alpha".to_string(), "beta".to_string()],
            vec!["gamma".to_string()],
        ],
    );
    let mut bp = ScalarStringVecTargetFormwork::builder()
        .texts(choice)
        .build();

    assert!(!bp.is_complete());
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve");
    assert_eq!(resolved.texts, vec!["alpha".to_string(), "beta".to_string()]);

    // Raw Vec<String> in wet builder
    let bp_raw = ScalarStringVecTargetFormwork::builder()
        .texts(vec!["direct".to_string()])
        .build();
    assert!(bp_raw.is_complete());
    assert_eq!(bp_raw.resolve().unwrap().texts, vec!["direct".to_string()]);

    // Direct Dry mode builder
    let direct: ScalarStringVecTarget = ScalarStringVecTarget::builder()
        .texts(vec!["dry_mode".to_string()])
        .build();
    assert_eq!(direct.texts, vec!["dry_mode".to_string()]);
}

// ============================================================================
// 3. Scalar slot holding Vec<Child>
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomItem {
    pub name: String,
    pub power: u32,
}

#[formwork(
    builder,
    derives(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScalarChildVecTarget {
    pub label: String,
    #[slot]
    pub items: Vec<CustomItem>,
}

#[test]
fn test_scalar_child_vec_slot_preservation() {
    let item1 = CustomItem {
        name: "Sword".to_string(),
        power: 10,
    };
    let item2 = CustomItem {
        name: "Shield".to_string(),
        power: 8,
    };
    let item3 = CustomItem {
        name: "Bow".to_string(),
        power: 12,
    };

    let choice = SingleChoice::new(
        "Loadout",
        vec![
            vec![item1.clone(), item2.clone()],
            vec![item3.clone()],
        ],
    );

    let mut bp = ScalarChildVecTargetFormwork::builder()
        .label("Adventurer".to_string())
        .items(choice)
        .build();

    assert!(!bp.is_complete());

    // Select second candidate
    bp.items.as_choice_mut().unwrap().select(1).unwrap();
    bp.items.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve loadout");
    assert_eq!(resolved.label, "Adventurer");
    assert_eq!(resolved.items, vec![item3]);
}

// ============================================================================
// 4. Scalar slot holding Vec<Vec<T>> (Matrix / Nested Collections)
// ============================================================================

#[formwork(
    builder,
    derives(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MatrixTarget {
    pub dim: usize,
    #[slot]
    pub grid: Vec<Vec<u32>>,
}

#[test]
fn test_scalar_nested_vec_matrix() {
    let identity_2x2 = vec![vec![1, 0], vec![0, 1]];
    let ones_2x2 = vec![vec![1, 1], vec![1, 1]];

    // Wet mode with SingleChoice of nested vectors
    let choice = SingleChoice::new("Select matrix", vec![identity_2x2.clone(), ones_2x2.clone()]);
    let mut bp = MatrixTargetFormwork::builder()
        .dim(2usize)
        .grid(choice)
        .build();

    assert!(!bp.is_complete());
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("matrix must resolve");
    assert_eq!(resolved.dim, 2);
    assert_eq!(resolved.grid, identity_2x2);

    // Raw assignment in wet mode
    let bp_raw = MatrixTargetFormwork::builder()
        .dim(2usize)
        .grid(ones_2x2.clone())
        .build();
    assert!(bp_raw.is_complete());
    assert_eq!(bp_raw.resolve().unwrap().grid, ones_2x2);

    // Direct Resolved mode builder
    let direct: MatrixTarget = MatrixTarget::builder()
        .dim(2usize)
        .grid(vec![vec![9, 9], vec![9, 9]])
        .build();

    assert_eq!(direct.grid, vec![vec![9, 9], vec![9, 9]]);
}

// ============================================================================
// 5. Deeply nested Vec<Vec<Vec<String>>>
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TensorTarget {
    #[slot]
    pub tensor: Vec<Vec<Vec<String>>>,
}

#[test]
fn test_deeply_nested_tensor_slot() {
    let t1 = vec![vec![vec!["x".to_string()]]];
    let t2 = vec![vec![vec!["y1".to_string(), "y2".to_string()]]];

    let choice = SingleChoice::new("Tensor choice", vec![t1.clone(), t2.clone()]);
    let mut bp = TensorTargetFormwork::builder().tensor(choice).build();
    assert!(!bp.is_complete());

    bp.tensor.as_choice_mut().unwrap().select(1).unwrap();
    bp.tensor.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("tensor resolves");
    assert_eq!(resolved.tensor, t2);
}

// ============================================================================
// 6. Comprehensive String Ergonomics Matrix Across Slot Kinds
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringErgonomicsMatrixTarget {
    /// Scalar String: accepts Into<String> in dry mode, String/Choice in wet mode
    #[slot]
    pub scalar_string: String,

    /// Scalar Vec<String>: accepts Vec<String>, SingleChoice<Vec<String>>, SingleSlot
    #[slot]
    pub scalar_vec_string: Vec<String>,

    /// Collection String: accepts Into<String> in dry mode, String/Choice in wet mode
    #[slot(collection)]
    pub collection_string: Vec<String>,

    /// Pick String: accepts Vec<__V>, [__V; __N], MultiChoice, MultiSlot
    #[slot(pick)]
    pub pick_string: Vec<String>,
}

#[test]
fn test_string_ergonomics_wet_mode() {
    let pick_slice = ["pick1", "pick2"];

    // Builder test using ergonomic conversions where permitted
    let bp = StringErgonomicsMatrixTargetFormwork::builder()
        .scalar_string("scalar_text".to_string())
        .scalar_vec_string(vec!["v1".to_string(), "v2".to_string()]) // Vec<String>
        .with_collection_string("elem1".to_string())
        .with_collection_string(String::from("elem2")) // String -> SingleSlot<String, String>
        .pick_string(pick_slice) // [__V; __N] -> MultiSlot<String, String>
        .build();

    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve string matrix");
    assert_eq!(resolved.scalar_string, "scalar_text");
    assert_eq!(resolved.scalar_vec_string, vec!["v1", "v2"]);
    assert_eq!(resolved.collection_string, vec!["elem1", "elem2"]);
    assert_eq!(resolved.pick_string, vec!["pick1", "pick2"]);
}

#[test]
fn test_string_ergonomics_dry_mode() {
    let pick_slice = ["p_res1", "p_res2"];

    let direct: StringErgonomicsMatrixTarget = StringErgonomicsMatrixTarget::builder()
        .scalar_string("res_scalar") // &str -> String via Into<String>
        .scalar_vec_string(vec!["res_v1".to_string()]) // Vec<String> -> Vec<String>
        .with_collection_string("res_coll1") // &str -> String via Into<String>
        .with_collection_string(String::from("res_coll2")) // String -> String
        .pick_string(pick_slice) // [__V; __N] -> Vec<String>
        .build();

    assert_eq!(direct.scalar_string, "res_scalar");
    assert_eq!(direct.scalar_vec_string, vec!["res_v1"]);
    assert_eq!(direct.collection_string, vec!["res_coll1", "res_coll2"]);
    assert_eq!(direct.pick_string, vec!["p_res1", "p_res2"]);
}

// ============================================================================
// 7. Custom ChoiceVisitor Traversal on Scalar Vector Slots
// ============================================================================

struct CountingVisitor {
    int_vec_choice_count: usize,
}

impl ChoiceVisitor<Vec<i32>> for CountingVisitor {
    fn visit_single(&mut self, _choice: &mut SingleChoice<Vec<i32>>) -> VisitorFlow {
        self.int_vec_choice_count += 1;
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<Vec<i32>>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

#[test]
fn test_custom_choice_visitor_on_scalar_vector_slot() {
    let choice = SingleChoice::new("v_nums", vec![vec![1, 2], vec![3, 4]]);
    let mut bp = ScalarIntVecTargetFormwork::builder()
        .tag("visitor_test".to_string())
        .numbers(choice)
        .build();

    let mut visitor = CountingVisitor {
        int_vec_choice_count: 0,
    };
    bp.walk_choices(&mut visitor);
    assert_eq!(visitor.int_vec_choice_count, 1);
}

// ============================================================================
// 8. Serde JSON Serialization Roundtrip with Scalar Vector Slots
// ============================================================================

#[test]
fn test_serde_roundtrip_scalar_vec_slot() {
    let choice = SingleChoice::new("s_nums", vec![vec![10, 20], vec![30, 40]]);
    let mut bp = ScalarIntVecTargetFormwork::builder()
        .tag("serde_test".to_string())
        .numbers(choice)
        .build();

    // Serialize in incomplete state
    let json_incomplete = serde_json::to_string(&bp).expect("must serialize incomplete");
    let deserialized_incomplete: ScalarIntVecTargetFormwork =
        serde_json::from_str(&json_incomplete).expect("must deserialize incomplete");
    assert_eq!(bp, deserialized_incomplete);
    assert!(!deserialized_incomplete.is_complete());

    // Select and serialize confirmed state
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let json_complete = serde_json::to_string(&bp).expect("must serialize complete");
    let deserialized_complete: ScalarIntVecTargetFormwork =
        serde_json::from_str(&json_complete).expect("must deserialize complete");
    assert_eq!(bp, deserialized_complete);
    assert!(deserialized_complete.is_complete());

    let resolved = deserialized_complete.resolve().expect("must resolve from deserialized");
    assert_eq!(resolved.numbers, vec![10, 20]);
}

// ============================================================================
// 9. Scalar slot holding Vec<FormworkStruct> (Dry Child Formworks as leaf)
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildFormworkTarget {
    #[slot]
    pub name: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScalarFormworkChildVecTarget {
    #[slot]
    pub children: Vec<ChildFormworkTarget>,
}

#[test]
fn test_scalar_formwork_child_vec() {
    let child1: ChildFormworkTarget = ChildFormworkTarget::builder().name("c1").build();
    let child2: ChildFormworkTarget = ChildFormworkTarget::builder().name("c2").build();

    let choice = SingleChoice::new("Pick children", vec![vec![child1.clone()], vec![child2.clone()]]);
    let mut bp = ScalarFormworkChildVecTargetFormwork::builder()
        .children(choice)
        .build();

    assert!(!bp.is_complete());
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve scalar formwork child vec");
    assert_eq!(resolved.children, vec![child1]);

    // Direct dry mode builder
    let direct: ScalarFormworkChildVecTarget = ScalarFormworkChildVecTarget::builder()
        .children(vec![child2.clone()])
        .build();
    assert_eq!(direct.children, vec![child2]);
}


// ============================================================================
// 10. Path-qualified std::vec::Vec<String>
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullyQualifiedVecTarget {
    #[slot]
    pub words: std::vec::Vec<String>,
}

#[test]
fn test_fully_qualified_vec_target() {
    let choice = SingleChoice::new("Words", vec![vec!["alpha".to_string()], vec!["beta".to_string()]]);
    let mut bp = FullyQualifiedVecTargetFormwork::builder()
        .words(choice)
        .build();

    assert!(!bp.is_complete());
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve fully qualified vec");
    assert_eq!(resolved.words, vec!["alpha".to_string()]);
}

// ============================================================================
// 11. Scalar slot holding Option<Vec<u32>> (Container wrapping Vec)
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionVecTarget {
    #[slot]
    pub maybe_vec: Option<Vec<u32>>,
}

#[test]
fn test_option_vec_slot_preservation() {
    let choice = SingleChoice::new("Option vec", vec![Some(vec![1, 2, 3]), None]);
    let mut bp = OptionVecTargetFormwork::builder()
        .maybe_vec(choice)
        .build();

    assert!(!bp.is_complete());
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve option vec");
    assert_eq!(resolved.maybe_vec, Some(vec![1, 2, 3]));
}

// ============================================================================
// 12. Nested Grid of FormworkStructs: Vec<Vec<ChildFormworkTarget>>
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormworkGridTarget {
    #[slot]
    pub grid: Vec<Vec<ChildFormworkTarget>>,
}

#[test]
fn test_formwork_grid_target() {
    let c: ChildFormworkTarget = ChildFormworkTarget::builder().name("node").build();
    let grid_data = vec![vec![c.clone()], vec![c.clone(), c.clone()]];

    let choice = SingleChoice::new("Grid", vec![grid_data.clone()]);
    let mut bp = FormworkGridTargetFormwork::builder()
        .grid(choice)
        .build();

    assert!(!bp.is_complete());
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve grid");
    assert_eq!(resolved.grid, grid_data);
}

