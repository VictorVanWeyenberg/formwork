//! Integration test suite for all 6 slot kinds and configurations in the `formwork` ecosystem:
//!
//! 1. `#[slot]` (Single Leaf)
//! 2. `#[slot(collection)]` (Collection Leaf)
//! 3. `#[slot(pick)]` (Pick Leaf with Cardinality)
//! 4. `#[slot(cascade)]` (Single Cascading Child Formwork)
//! 5. `#[slot(collection, cascade)]` (Collection Cascading Children)
//! 6. `#[slot(pick, cascade)]` (Pick Cascading Children with Cardinality)

use formwork::{
    Formwork, Slot,
    choice::{Cardinality, ChoiceState, MultiChoice, SingleChoice},
    error::FormworkError,
    formwork,
    visitor::AutoSelectFirst,
};

// ============================================================================
// 1. #[slot] (Single Leaf Slot)
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SingleLeafTarget {
    pub id: u32,
    #[slot]
    pub weapon: String,
}

#[test]
fn test_1_1_single_leaf_raw_values() {
    // Direct raw value assignment
    let bp_str = SingleLeafTarget::builder()
        .id(1u32)
        .weapon("Warhammer".to_string())
        .build();
    assert!(bp_str.is_complete());
    let resolved_str = bp_str.resolve().unwrap();
    assert_eq!(
        resolved_str,
        SingleLeafTarget {
            id: 1,
            weapon: "Warhammer".to_string(),
        }
    );

    // Direct raw String assignment
    let bp_string = SingleLeafTarget::builder()
        .id(2u32)
        .weapon("Handaxe".to_string())
        .build();
    assert!(bp_string.is_complete());
    let resolved_string = bp_string.resolve().unwrap();
    assert_eq!(
        resolved_string,
        SingleLeafTarget {
            id: 2,
            weapon: "Handaxe".to_string(),
        }
    );
}

#[test]
fn test_1_2_single_leaf_pending_choice_failure() {
    let choice = SingleChoice::new(
        "Choose weapon",
        vec!["Warhammer".to_string(), "Handaxe".to_string()],
    );
    let bp = SingleLeafTarget::builder().id(1u32).weapon(choice).build();

    assert!(!bp.is_complete());
    match bp.resolve() {
        Err(FormworkError::WetChoice { prompt }) => {
            assert_eq!(prompt, "Choose weapon");
        }
        other => panic!("Expected WetChoice error, got {:?}", other),
    }
}

#[test]
fn test_1_3_single_leaf_selected_choice_resolution() {
    let choice = SingleChoice::new(
        "Choose weapon",
        vec!["Warhammer".to_string(), "Handaxe".to_string()],
    );
    let mut bp = SingleLeafTarget::builder().id(1u32).weapon(choice).build();

    assert!(!bp.is_complete());

    bp.weapon.as_choice_mut().unwrap().select(1).unwrap();
    assert!(!bp.is_complete(), "select only stages candidate");

    bp.weapon.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().unwrap();
    assert_eq!(
        resolved,
        SingleLeafTarget {
            id: 1,
            weapon: "Handaxe".to_string(),
        }
    );
}

#[test]
fn test_1_4_single_leaf_unmake_and_confirm_cycle() {
    let mut choice = SingleChoice::new(
        "Choose weapon",
        vec!["Warhammer".to_string(), "Handaxe".to_string()],
    );
    choice.select(1).unwrap();
    choice.confirm().unwrap();
    assert!(choice.is_made());

    let mut bp = SingleLeafTarget::builder().id(1u32).weapon(choice).build();
    assert!(bp.is_complete());

    // Unmake choice: becomes incomplete, preserves history 1
    bp.weapon.as_choice_mut().unwrap().unmake().unwrap();
    assert!(!bp.is_complete());
    assert_eq!(
        bp.weapon.as_choice_mut().unwrap().state,
        ChoiceState::UnmadeWithHistory(1)
    );

    // Confirm restores choice state to Made
    bp.weapon.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().unwrap();
    assert_eq!(
        resolved,
        SingleLeafTarget {
            id: 1,
            weapon: "Handaxe".to_string(),
        }
    );
}

// ============================================================================
// 2. #[slot(collection)] (Collection Leaf Slot)
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct CollectionLeafTarget {
    #[slot(collection)]
    pub tags: Vec<String>,
}

#[test]
fn test_2_1_collection_leaf_empty_default() {
    let bp = CollectionLeafTarget::builder().build();
    assert!(bp.is_complete());
    let resolved = bp.resolve().unwrap();
    assert!(resolved.tags.is_empty());
}

#[test]
fn test_2_2_collection_leaf_mutators_append() {
    let bp = CollectionLeafTarget::builder()
        .with_tags("alpha".to_string())
        .with_tags("beta".to_string())
        .tags(vec!["gamma".to_string()])
        .build();

    assert!(bp.is_complete());
    let resolved = bp.resolve().unwrap();
    assert_eq!(resolved.tags, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn test_2_3_collection_leaf_interleaved_items_and_singlechoice() {
    let choice = SingleChoice::new("Pick 1 tag", vec!["dyn1".to_string(), "dyn2".to_string()]);
    let mut bp = CollectionLeafTarget::builder()
        .with_tags("static".to_string())
        .with_tags(choice)
        .build();

    // Incomplete before selection
    assert!(!bp.is_complete());
    assert!(bp.clone().resolve().is_err());

    // Select index 0 ("dyn1")
    bp.tags[1].as_choice_mut().unwrap().select(0).unwrap();
    assert!(!bp.is_complete());
    bp.tags[1].as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().unwrap();
    assert_eq!(
        resolved.tags,
        vec!["static".to_string(), "dyn1".to_string()]
    );
}

#[test]
fn test_2_4_collection_leaf_preserves_1_to_1_order() {
    let choice1 = SingleChoice::new("Pick tag 1", vec!["mid1".to_string(), "alt1".to_string()]);
    let choice2 = SingleChoice::new("Pick tag 2", vec!["mid2".to_string(), "alt2".to_string()]);

    let mut bp = CollectionLeafTarget::builder()
        .with_tags("first".to_string())
        .with_tags(choice1)
        .with_tags(choice2)
        .with_tags("last".to_string())
        .build();

    bp.tags[1].as_choice_mut().unwrap().select(0).unwrap();
    bp.tags[1].as_choice_mut().unwrap().confirm().unwrap();
    assert!(!bp.is_complete());

    bp.tags[2].as_choice_mut().unwrap().select(0).unwrap();
    bp.tags[2].as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().unwrap();
    assert_eq!(
        resolved.tags,
        vec![
            "first".to_string(),
            "mid1".to_string(),
            "mid2".to_string(),
            "last".to_string()
        ]
    );
}

// ============================================================================
// 3. #[slot(pick)] (Pick Leaf Slot with Cardinality)
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct PickLeafTarget {
    pub max_skills: usize,
    #[slot(pick)]
    pub skills: Vec<String>,
}

#[test]
fn test_3_1_pick_leaf_exact_cardinality() {
    let choice = MultiChoice::new(
        "Pick 2 skills",
        vec![
            "History".to_string(),
            "Medicine".to_string(),
            "Insight".to_string(),
        ],
    )
    .with_cardinality(Cardinality::Exact(2));

    let mut bp = PickLeafTarget::builder()
        .max_skills(2usize)
        .skills(choice)
        .build();

    // 1. Initial pending
    assert!(!bp.is_complete());
    assert!(bp.clone().resolve().is_err());

    // 2. Select index 0 ("History"): only 1 selection, still incomplete
    bp.skills.as_choice_mut().unwrap().select(0).unwrap();
    assert!(!bp.is_complete());

    // 3. Select index 2 ("Insight"): 2 selections reached, but still incomplete until confirm
    bp.skills.as_choice_mut().unwrap().select(2).unwrap();
    assert!(!bp.is_complete());

    bp.skills.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    // 4. Resolve produces selected skills
    let resolved = bp.resolve().unwrap();
    assert_eq!(
        resolved.skills,
        vec!["History".to_string(), "Insight".to_string()]
    );
}

#[test]
fn test_3_2_pick_leaf_duplicate_selection_guard() {
    let mut choice: MultiChoice<String> = MultiChoice::new(
        "Pick 2 skills",
        vec![
            "History".to_string(),
            "Medicine".to_string(),
            "Insight".to_string(),
        ],
    )
    .with_cardinality(Cardinality::Exact(2));

    choice.select(0).unwrap();
    let err = choice.select(0).unwrap_err();

    match err {
        FormworkError::DuplicateSelection { prompt, index } => {
            assert_eq!(prompt, "Pick 2 skills");
            assert_eq!(index, 0);
        }
        other => panic!("Expected DuplicateSelection error, got {:?}", other),
    }
}

#[test]
fn test_3_3_pick_leaf_upto_cardinality_with_confirm() {
    let choice = MultiChoice::new(
        "Pick up to 3 skills",
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
    )
    .with_cardinality(Cardinality::UpTo(3));

    let mut bp = PickLeafTarget::builder()
        .max_skills(3usize)
        .skills(choice)
        .build();

    assert!(!bp.is_complete());

    // Select 1 option ("B")
    bp.skills.as_choice_mut().unwrap().select(1).unwrap();
    // UpTo(3) allows 1, but is not automatically Made until confirmed
    assert!(!bp.is_complete());

    // Confirm choice
    bp.skills.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().unwrap();
    assert_eq!(resolved.skills, vec!["B".to_string()]);
}

// ============================================================================
// 4. #[slot(cascade)] (Single Cascading Child Formwork)
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ChildSlotNode {
    pub id: String,
    #[slot]
    pub value: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ParentSingleCascade {
    pub name: String,
    #[slot(cascade)]
    pub child: ChildSlotNode,
}

#[test]
fn test_4_1_single_cascade_child_incompleteness_cascades_to_parent() {
    let child_bp = ChildSlotNode::builder()
        .id("C1")
        .value(SingleChoice::new(
            "Select val",
            vec!["V1".to_string(), "V2".to_string()],
        ))
        .build();

    let parent = ParentSingleCascade::builder()
        .name("Root")
        .child(child_bp)
        .build();

    // Child is incomplete, so parent must be incomplete
    assert!(!parent.is_complete());

    match parent.resolve() {
        Err(FormworkError::WetChoice { prompt }) => {
            assert_eq!(prompt, "Select val");
        }
        other => panic!("Expected WetChoice, got {:?}", other),
    }
}

#[test]
fn test_4_2_single_cascade_automated_resolution_via_visitor() {
    let child_bp = ChildSlotNode::builder()
        .id("C1")
        .value(SingleChoice::new("Select val", vec!["V1".to_string()]))
        .build();

    let mut parent = ParentSingleCascade::builder()
        .name("Root")
        .child(child_bp)
        .build();

    assert!(!parent.is_complete());

    parent.walk_choices(&mut AutoSelectFirst);
    assert!(parent.is_complete());

    let resolved = parent.resolve().unwrap();
    assert_eq!(
        resolved,
        ParentSingleCascade {
            name: "Root".to_string(),
            child: ChildSlotNode {
                id: "C1".to_string(),
                value: "V1".to_string(),
            },
        }
    );
}

#[test]
fn test_4_3_single_cascade_choice_of_child_formworks() {
    let cand_a = ChildSlotNode::builder()
        .id("CandidateA")
        .value("FixedA".to_string())
        .build();

    let cand_b = ChildSlotNode::builder()
        .id("CandidateB")
        .value(SingleChoice::new(
            "Inner choice B",
            vec!["B1".to_string(), "B2".to_string()],
        ))
        .build();

    let choice = SingleChoice::new("Select child candidate", vec![cand_a, cand_b]);

    let mut parent = ParentSingleCascade::builder()
        .name("Root")
        .child(choice)
        .build();

    // 1. Initial pending choice
    assert!(!parent.is_complete());
    assert!(parent.clone().resolve().is_err());

    // 2. Select Candidate B (index 1) and confirm
    parent.child.as_choice_mut().unwrap().select(1).unwrap();
    parent.child.as_choice_mut().unwrap().confirm().unwrap();

    // Pick choice is made, but Candidate B still has pending inner choice -> parent is incomplete
    assert!(!parent.is_complete());
    assert!(parent.clone().resolve().is_err());

    // 3. Satisfy Candidate B's inner choice
    let mut selected_cand = parent.child.as_choice_mut().unwrap().selected_values_mut();
    let inner = selected_cand[0].value.as_choice_mut().unwrap();
    inner.select(0).unwrap();
    inner.confirm().unwrap();

    assert!(parent.is_complete());

    let resolved = parent.resolve().unwrap();
    assert_eq!(
        resolved,
        ParentSingleCascade {
            name: "Root".to_string(),
            child: ChildSlotNode {
                id: "CandidateB".to_string(),
                value: "B1".to_string(),
            },
        }
    );
}

#[test]
fn test_4_4_single_cascade_pre_resolved_child() {
    let resolved_child: ChildSlotNode = ChildSlotNode {
        id: "PreDryId".to_string(),
        value: "PreDryVal".to_string(),
    };

    let parent = ParentSingleCascade::builder()
        .name("Root")
        .child(resolved_child)
        .build();

    assert!(parent.is_complete());
    let resolved = parent.resolve().unwrap();
    assert_eq!(
        resolved,
        ParentSingleCascade {
            name: "Root".to_string(),
            child: ChildSlotNode {
                id: "PreDryId".to_string(),
                value: "PreDryVal".to_string(),
            },
        }
    );
}

// ============================================================================
// 5. #[slot(collection, cascade)] (Collection Cascading Child Formworks)
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ItemSubsystem {
    pub code: String,
    #[slot]
    pub mode: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct GroupCollectionCascade {
    pub title: String,
    #[slot(collection, cascade)]
    pub systems: Vec<ItemSubsystem>,
}

#[test]
fn test_5_1_collection_cascade_mixed_ready_and_pending_children() {
    let child1: ItemSubsystemFormwork = ItemSubsystem::builder().code("SYS1").mode("Auto".to_string()).build();

    let child2: ItemSubsystemFormwork = ItemSubsystem::builder()
        .code("SYS2")
        .mode(SingleChoice::new(
            "SYS2 Mode",
            vec!["Manual".to_string(), "Eco".to_string()],
        ))
        .build();

    let mut group = GroupCollectionCascade::builder()
        .title("Alpha")
        .with_systems(child1)
        .with_systems(child2)
        .build();

    // Incomplete because child2 has open choice
    assert!(!group.is_complete());
    assert!(group.clone().resolve().is_err());

    // Satisfy Child 2's mode choice
    if let Slot::FixedWet(ref mut c2) = group.systems[1] {
        let choice = c2.mode.as_choice_mut().unwrap();
        choice.select(0).unwrap();
        choice.confirm().unwrap();
    }
    assert!(group.is_complete());

    let resolved = group.resolve().unwrap();
    assert_eq!(
        resolved,
        GroupCollectionCascade {
            title: "Alpha".to_string(),
            systems: vec![
                ItemSubsystem {
                    code: "SYS1".to_string(),
                    mode: "Auto".to_string(),
                },
                ItemSubsystem {
                    code: "SYS2".to_string(),
                    mode: "Manual".to_string(),
                },
            ],
        }
    );
}

#[test]
fn test_5_2_collection_cascade_empty() {
    let group = GroupCollectionCascade::builder().title("Empty").build();
    assert!(group.is_complete());
    let resolved = group.resolve().unwrap();
    assert_eq!(resolved.title, "Empty");
    assert!(resolved.systems.is_empty());
}

// ============================================================================
// 6. #[slot(pick, cascade)] (Pick Cascading Child Formworks)
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SpecialistCompanion {
    pub name: String,
    #[slot]
    pub ability: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct AdventuringParty {
    pub party_name: String,
    #[slot(pick, cascade)]
    pub companions: Vec<SpecialistCompanion>,
}

#[test]
fn test_6_1_pick_cascade_pool_of_candidates_with_internal_choices() {
    let cand1: SpecialistCompanionFormwork = SpecialistCompanion::builder()
        .name("Warrior")
        .ability("Swordplay".to_string())
        .build();

    let cand2: SpecialistCompanionFormwork = SpecialistCompanion::builder()
        .name("Wizard")
        .ability(SingleChoice::new(
            "Wizard element",
            vec!["Fire".to_string(), "Ice".to_string()],
        ))
        .build();

    let cand3: SpecialistCompanionFormwork = SpecialistCompanion::builder()
        .name("Rogue")
        .ability(SingleChoice::new(
            "Rogue trick",
            vec!["Lockpick".to_string(), "Stealth".to_string()],
        ))
        .build();

    let choice = MultiChoice::new("Pick 2 companions", vec![cand1, cand2, cand3])
        .with_cardinality(Cardinality::Exact(2));

    let mut party = AdventuringParty::builder()
        .party_name("Guild")
        .companions(choice)
        .build();

    // 1. Initial unpicked state
    assert!(!party.is_complete());
    assert!(party.clone().resolve().is_err());

    // 2. Select Candidate 1 (Warrior, index 0) and Candidate 2 (Wizard, index 1)
    party.companions.as_choice_mut().unwrap().select(0).unwrap();
    party.companions.as_choice_mut().unwrap().select(1).unwrap();
    party.companions.as_choice_mut().unwrap().confirm().unwrap();

    // Pick choice is Made, BUT selected Candidate 2 has an open inner ability choice!
    // Parent must remain incomplete (deep recursive verification).
    assert!(!party.is_complete());
    assert!(party.clone().resolve().is_err());

    // 3. Satisfy Candidate 2's inner choice ("Fire", index 0)
    let mut selected_companions = party
        .companions
        .as_choice_mut()
        .unwrap()
        .selected_values_mut();
    // Index 1 in selected options is Candidate 2 (Wizard)
    let ability_choice = selected_companions[1].ability.as_choice_mut().unwrap();
    ability_choice.select(0).unwrap();
    ability_choice.confirm().unwrap();

    // Now all selected candidates are fully complete!
    assert!(party.is_complete());

    // 4. Resolve produces watertight AdventuringParty
    // Unselected Candidate 3's pending choices do NOT block resolution.
    let resolved = party.resolve().unwrap();
    assert_eq!(resolved.party_name, "Guild");
    assert_eq!(resolved.companions.len(), 2);
    assert_eq!(
        resolved.companions[0],
        SpecialistCompanion {
            name: "Warrior".to_string(),
            ability: "Swordplay".to_string(),
        }
    );
    assert_eq!(
        resolved.companions[1],
        SpecialistCompanion {
            name: "Wizard".to_string(),
            ability: "Fire".to_string(),
        }
    );
}

// ============================================================================
// 7. Standard Derives & Serde Across All 6 Slot Kinds (Without Trait Imports)
// ============================================================================

#[formwork(builder)]
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct DeriveChild {
    pub id: String,
    #[slot]
    pub item: String,
}

#[formwork(builder)]
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct AllSlotKindsDeriveTarget {
    pub label: String,
    #[slot]
    pub single_leaf: String,
    #[slot(collection)]
    pub collection_leaf: Vec<String>,
    #[slot(pick)]
    pub pick_leaf: Vec<String>,
    #[slot(cascade)]
    pub single_cascade: DeriveChild,
    #[slot(collection, cascade)]
    pub collection_cascade: Vec<DeriveChild>,
    #[slot(pick, cascade)]
    pub pick_cascade: Vec<DeriveChild>,
}

#[formwork(
    builder,
    derives(
        Clone,
        Debug,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        serde::Serialize,
        serde::Deserialize
    )
)]
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ExplicitDerivesBackwardsCompat {
    #[slot]
    pub name: String,
}

#[test]
fn test_7_1_all_derives_on_all_six_slot_kinds_wet_and_dry() {
    use std::collections::{BTreeSet, HashSet};

    let child_a: DeriveChildFormwork = DeriveChild::builder().id("A").item("Sword".to_string()).build();
    let child_b = DeriveChild::builder()
        .id("B")
        .item(SingleChoice::new(
            "Pick shield",
            vec!["Buckler".to_string(), "Kite".to_string()],
        ))
        .build();
    let child_c: DeriveChildFormwork =
        DeriveChild::builder().id("C").item("Helm".to_string()).build();

    let mut bp: AllSlotKindsDeriveTargetFormwork = AllSlotKindsDeriveTarget::builder()
        .label("Hero")
        .single_leaf(SingleChoice::new(
            "Class",
            vec!["Paladin".to_string(), "Ranger".to_string()],
        ))
        .with_collection_leaf("Torch".to_string())
        .pick_leaf(
            MultiChoice::new(
                "Skills",
                vec!["Athletics".to_string(), "Stealth".to_string()],
            )
            .with_cardinality(Cardinality::Exact(1)),
        )
        .single_cascade(child_a)
        .with_collection_cascade(child_b)
        .pick_cascade(
            MultiChoice::new("Companions", vec![child_c]).with_cardinality(Cardinality::Exact(1)),
        )
        .build();

    // Verify Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize on wet formwork
    let bp_cloned = bp.clone();
    assert_eq!(bp, bp_cloned);
    assert!(!format!("{bp:?}").is_empty());

    let mut bp_hash_set = HashSet::new();
    bp_hash_set.insert(bp.clone());
    assert!(bp_hash_set.contains(&bp_cloned));

    let mut bp_btree_set = BTreeSet::new();
    bp_btree_set.insert(bp.clone());
    assert!(bp_btree_set.contains(&bp_cloned));

    let bp_json = serde_json::to_string(&bp).unwrap();
    let bp_roundtrip: AllSlotKindsDeriveTargetFormwork = serde_json::from_str(&bp_json).unwrap();
    assert_eq!(bp, bp_roundtrip);

    // Satisfy all choices and resolve
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());
    let resolved: AllSlotKindsDeriveTarget = bp.resolve().unwrap();

    // Verify Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize on dry struct
    let resolved_cloned = resolved.clone();
    assert_eq!(resolved, resolved_cloned);
    assert!(!format!("{resolved:?}").is_empty());

    let mut resolved_hash_set = HashSet::new();
    resolved_hash_set.insert(resolved.clone());
    assert!(resolved_hash_set.contains(&resolved_cloned));

    let mut resolved_btree_set = BTreeSet::new();
    resolved_btree_set.insert(resolved.clone());
    assert!(resolved_btree_set.contains(&resolved_cloned));

    let resolved_json = serde_json::to_string(&resolved).unwrap();
    let resolved_roundtrip: AllSlotKindsDeriveTarget =
        serde_json::from_str(&resolved_json).unwrap();
    assert_eq!(resolved, resolved_roundtrip);

    // Verify backwards compatibility with explicit #[formwork(derives(...))]
    let compat_bp: ExplicitDerivesBackwardsCompatFormwork =
        ExplicitDerivesBackwardsCompat::builder()
            .name("Test".to_string())
            .build();
    let compat_resolved = compat_bp.clone().resolve().unwrap();
    assert_eq!(compat_bp, compat_bp.clone());
    assert_eq!(compat_resolved.name, "Test");
}

// ============================================================================
// 8. Adversarial Edge Cases (No Builder, Float Slots, Qualified Derives, Serde Attrs, 2-Segment Types)
// ============================================================================

/// Struct without `builder` enabled across collection and single slots.
#[formwork]
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct NoBuilderFormwork {
    pub tag: String,
    #[slot]
    pub item: Option<String>,
    #[slot(collection)]
    pub list: Vec<String>,
}

/// Struct with `f64` slots (which implement `PartialEq` and `PartialOrd`, but NOT `Eq`, `Ord`, or `Hash`)
/// and a 2-segment non-slot path (`serde_json::Value`) to verify `__qualify_mode` does not corrupt non-`M` paths.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FloatAndTwoSegmentTarget {
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub static_items: Vec<String>,
    #[serde(rename = "weight_kg")]
    #[slot]
    pub weight: f64,
    #[serde(default)]
    #[slot(collection)]
    pub measurements: Vec<f64>,
    #[slot(pick)]
    pub calibration_points: Vec<f64>,
}

/// Struct using fully-qualified `std::*` paths in `#[derive(...)]` and `#[derive(Default)]`.
#[formwork(builder)]
#[derive(
    std::clone::Clone,
    std::fmt::Debug,
    std::default::Default,
    std::cmp::PartialEq,
    std::cmp::Eq,
    std::cmp::PartialOrd,
    std::cmp::Ord,
    std::hash::Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct QualifiedStdDerivesTarget {
    #[serde(default)]
    pub count: usize,
    #[serde(default)]
    #[slot(collection)]
    pub entries: Vec<String>,
}

#[test]
fn test_8_1_no_builder_derives_and_default() {
    let default_resolved: NoBuilderFormwork = NoBuilderFormwork::default();
    assert_eq!(default_resolved.tag, "");
    assert_eq!(default_resolved.item, None);
    assert!(default_resolved.list.is_empty());

    let bp = NoBuilderFormworkFormwork {
        tag: "manual".to_string(),
        item: Slot::FixedWet(Some("value".to_string())),
        list: vec![Slot::FixedWet("entry".to_string())],
    };
    assert!(bp.is_complete());
    let resolved = bp.clone().resolve().unwrap();
    assert_eq!(resolved.tag, "manual");
    assert_eq!(resolved.item, Some("value".to_string()));
    assert_eq!(resolved.list, vec!["entry".to_string()]);
}

#[test]
fn test_8_2_float_selective_derives_and_two_segment_non_slot_paths() {
    let bp = FloatAndTwoSegmentTarget::builder()
        .metadata(serde_json::json!({"unit": "metric"}))
        .with_static_items("sensor-1")
        .weight(42.5_f64)
        .with_measurements(1.25_f64)
        .calibration_points(vec![0.0_f64, 100.0_f64])
        .build();

    assert!(bp.is_complete());
    let json = serde_json::to_string(&bp).unwrap();
    assert!(json.contains("weight_kg"));

    let roundtrip: FloatAndTwoSegmentTargetFormwork = serde_json::from_str(&json).unwrap();
    assert_eq!(bp, roundtrip);

    let resolved = bp.resolve().unwrap();
    assert_eq!(resolved.weight, 42.5_f64);
    assert_eq!(resolved.measurements, vec![1.25_f64]);
    assert_eq!(resolved.calibration_points, vec![0.0_f64, 100.0_f64]);
    assert_eq!(resolved.static_items, vec!["sensor-1".to_string()]);
    assert_eq!(resolved.metadata, serde_json::json!({"unit": "metric"}));
}

#[test]
fn test_8_3_qualified_std_derives_and_default_on_formwork_and_dry() {
    use std::collections::{BTreeSet, HashSet};

    let def_bp = QualifiedStdDerivesTargetFormwork::default();
    assert!(def_bp.is_complete());
    assert!(def_bp.entries.is_empty());

    let def_resolved: QualifiedStdDerivesTarget = QualifiedStdDerivesTarget::default();
    assert_eq!(def_resolved.count, 0);
    assert!(def_resolved.entries.is_empty());

    let built: QualifiedStdDerivesTargetFormwork = QualifiedStdDerivesTarget::builder()
        .count(3usize)
        .with_entries("one".to_string())
        .build();
    assert!(built.is_complete());
    let mut set = HashSet::new();
    set.insert(built.clone());
    assert!(set.contains(&built));

    let mut btree = BTreeSet::new();
    btree.insert(built.clone());
    assert!(btree.contains(&built));
}

/// Struct with multi-line doc comments across all 4 field generator branches
/// (regular scalar, regular Vec, single/pick slot, and collection slot) to verify
/// `extract_doc_comments` and `format_doc_with_details`, as well as direct resolved-mode building.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentedAndDryBuilderTarget {
    /// Primary identifier for the configuration.
    /// Must be non-empty.
    pub id: String,
    /// Static labels attached to the configuration.
    /// Preserves insertion order.
    pub static_labels: Vec<String>,
    /// Primary role selection.
    /// Supports string slice ergonomics.
    #[slot]
    pub role: String,
    /// Ordered list of equipment items.
    /// Supports piecemeal `with_equipment` and batch `equipment`.
    #[slot(collection)]
    pub equipment: Vec<String>,
    /// Selected bonus perks.
    /// Supports `Vec<&str>` and `&[&str]` slice conversions.
    #[slot(pick)]
    pub perks: Vec<String>,
}

#[test]
fn test_8_4_documented_fields_and_dry_mode_builder_ergonomics() {
    let perk_slice = ["PerkA", "PerkB"];

    // Build directly in wet mode using [__V; __N] on pick slot and iterator mutator on collection slot
    let bp: DocumentedAndDryBuilderTargetFormwork =
        DocumentedAndDryBuilderTarget::builder()
            .id("cfg-1")
            .with_static_labels("v1")
            .role("Tank".to_string())
            .with_equipment("Shield".to_string())
            .equipment(["Armor".to_string(), "Boots".to_string()])
            .perks(perk_slice)
            .build();
    assert!(bp.is_complete());
    let resolved_from_bp = bp.resolve().unwrap();

    // Build directly in Dry mode (DocumentedAndDryBuilderTarget<DocumentedAndDryBuilderTargetDry>)
    let direct_resolved: DocumentedAndDryBuilderTarget =
        DocumentedAndDryBuilderTarget::builder()
            .id("cfg-1")
            .with_static_labels("v1")
            .role("Tank")
            .with_equipment("Shield")
            .equipment(["Armor", "Boots"])
            .perks(vec!["PerkA", "PerkB"])
            .build();

    assert_eq!(resolved_from_bp, direct_resolved);

    // Also test [__V; __N] in Dry mode
    let direct_resolved_slice: DocumentedAndDryBuilderTarget =
        DocumentedAndDryBuilderTarget::builder()
            .id("cfg-1")
            .with_static_labels("v1")
            .role("Tank")
            .with_equipment("Shield")
            .equipment(["Armor", "Boots"])
            .perks(perk_slice)
            .build();
    assert_eq!(direct_resolved, direct_resolved_slice);
}
