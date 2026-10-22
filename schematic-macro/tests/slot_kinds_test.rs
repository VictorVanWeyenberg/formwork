//! Integration test suite for all 6 slot kinds and configurations in the `schematic` ecosystem:
//!
//! 1. `#[slot]` (Single Leaf)
//! 2. `#[slot(collection)]` (Collection Leaf)
//! 3. `#[slot(pick)]` (Pick Leaf with Cardinality)
//! 4. `#[slot(cascade)]` (Single Cascading Child Schematic)
//! 5. `#[slot(collection, cascade)]` (Collection Cascading Children)
//! 6. `#[slot(pick, cascade)]` (Pick Cascading Children with Cardinality)

use schematic::{
    Schematic, Slot,
    choice::{Cardinality, ChoiceState, MultiChoice, SingleChoice},
    error::SchematicError,
    schematic,
    visitor::AutoSelectFirst,
};

// ============================================================================
// 1. #[slot] (Single Leaf Slot)
// ============================================================================

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SingleLeafTarget {
    pub id: u32,
    #[slot]
    pub weapon: String,
}

#[test]
fn test_1_1_single_leaf_raw_values() {
    // Direct raw &str assignment
    let bp_str = SingleLeafTarget::builder()
        .id(1u32)
        .weapon("Warhammer")
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
        Err(SchematicError::UnresolvedChoice { prompt }) => {
            assert_eq!(prompt, "Choose weapon");
        }
        other => panic!("Expected UnresolvedChoice error, got {:?}", other),
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

#[schematic(builder)]
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
        .with_tags("alpha")
        .with_tags("beta")
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
        .with_tags("static")
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
        .with_tags("first")
        .with_tags(choice1)
        .with_tags(choice2)
        .with_tags("last")
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

#[schematic(builder)]
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
    let mut choice = MultiChoice::new(
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
        SchematicError::DuplicateSelection { prompt, index } => {
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
// 4. #[slot(cascade)] (Single Cascading Child Schematic)
// ============================================================================

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ChildSlotNode {
    pub id: String,
    #[slot]
    pub value: String,
}

#[schematic(builder)]
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
        Err(SchematicError::UnresolvedChoice { prompt }) => {
            assert_eq!(prompt, "Select val");
        }
        other => panic!("Expected UnresolvedChoice, got {:?}", other),
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
fn test_4_3_single_cascade_choice_of_child_blueprints() {
    let cand_a = ChildSlotNode::builder()
        .id("CandidateA")
        .value("FixedA")
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
        id: "PreResolvedId".to_string(),
        value: "PreResolvedVal".to_string(),
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
                id: "PreResolvedId".to_string(),
                value: "PreResolvedVal".to_string(),
            },
        }
    );
}

// ============================================================================
// 5. #[slot(collection, cascade)] (Collection Cascading Child Schematics)
// ============================================================================

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ItemSubsystem {
    pub code: String,
    #[slot]
    pub mode: String,
}

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct GroupCollectionCascade {
    pub title: String,
    #[slot(collection, cascade)]
    pub systems: Vec<ItemSubsystem>,
}

#[test]
fn test_5_1_collection_cascade_mixed_ready_and_pending_children() {
    let child1: ItemSubsystemSchematic = ItemSubsystem::builder().code("SYS1").mode("Auto").build();

    let child2: ItemSubsystemSchematic = ItemSubsystem::builder()
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
    if let Slot::FixedUnresolved(ref mut c2) = group.systems[1] {
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
// 6. #[slot(pick, cascade)] (Pick Cascading Child Schematics)
// ============================================================================

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SpecialistCompanion {
    pub name: String,
    #[slot]
    pub ability: String,
}

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct AdventuringParty {
    pub party_name: String,
    #[slot(pick, cascade)]
    pub companions: Vec<SpecialistCompanion>,
}

#[test]
fn test_6_1_pick_cascade_pool_of_candidates_with_internal_choices() {
    let cand1: SpecialistCompanionSchematic = SpecialistCompanion::builder()
        .name("Warrior")
        .ability("Swordplay")
        .build();

    let cand2: SpecialistCompanionSchematic = SpecialistCompanion::builder()
        .name("Wizard")
        .ability(SingleChoice::new(
            "Wizard element",
            vec!["Fire".to_string(), "Ice".to_string()],
        ))
        .build();

    let cand3: SpecialistCompanionSchematic = SpecialistCompanion::builder()
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

#[schematic(builder)]
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct DeriveChild {
    pub id: String,
    #[slot]
    pub item: String,
}

#[schematic(builder)]
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

#[schematic(
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
fn test_7_1_all_derives_on_all_six_slot_kinds_resolved_and_unresolved() {
    use std::collections::{BTreeSet, HashSet};

    let child_a: DeriveChildSchematic = DeriveChild::builder().id("A").item("Sword").build();
    let child_b = DeriveChild::builder()
        .id("B")
        .item(SingleChoice::new(
            "Pick shield",
            vec!["Buckler".to_string(), "Kite".to_string()],
        ))
        .build();
    let child_c = DeriveChild::builder().id("C").item("Helm").build();

    let mut bp: AllSlotKindsDeriveTargetSchematic = AllSlotKindsDeriveTarget::builder()
        .label("Hero")
        .single_leaf(SingleChoice::new(
            "Class",
            vec!["Paladin".to_string(), "Ranger".to_string()],
        ))
        .with_collection_leaf("Torch")
        .pick_leaf(
            MultiChoice::new("Skills", vec!["Athletics".to_string(), "Stealth".to_string()])
                .with_cardinality(Cardinality::Exact(1)),
        )
        .single_cascade(child_a)
        .with_collection_cascade(child_b)
        .pick_cascade(
            MultiChoice::new("Companions", vec![child_c]).with_cardinality(Cardinality::Exact(1)),
        )
        .build();

    // Verify Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize on unresolved blueprint
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
    let bp_roundtrip: AllSlotKindsDeriveTargetSchematic = serde_json::from_str(&bp_json).unwrap();
    assert_eq!(bp, bp_roundtrip);

    // Satisfy all choices and resolve
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());
    let resolved: AllSlotKindsDeriveTarget = bp.resolve().unwrap();

    // Verify Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize on resolved struct
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

    // Verify backwards compatibility with explicit #[schematic(derives(...))]
    let compat_bp = ExplicitDerivesBackwardsCompat::builder()
        .name("Test")
        .build();
    let compat_resolved = compat_bp.clone().resolve().unwrap();
    assert_eq!(compat_bp, compat_bp.clone());
    assert_eq!(compat_resolved.name, "Test");
}

