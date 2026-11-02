#![deny(missing_docs)]
//! Integration test verifying that `#[formwork]` generated code satisfies `#![deny(missing_docs)]`
//! and properly exposes documented Mode traits, associated types, and marker structs.

use formwork::choice::{Cardinality, MultiChoice, SingleChoice};
use formwork::visitor::AutoSelectFirst;
use formwork::{Formwork, formwork};

/// Public documented module containing public formwork definitions subject to `#![deny(missing_docs)]`.
pub mod documented_entities {
    use super::*;

    /// A nested child entity to test cascading slots under missing_docs enforcement.
    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq)]
    pub struct DocumentedChild {
        /// Identifier for the child.
        pub child_id: String,
        /// Scalar slot in the child.
        #[slot]
        pub detail: String,
    }

    /// A comprehensive formwork entity featuring all 8 slot kinds plus fixed invariants:
    /// - Fixed invariant field
    /// - Scalar leaf slot
    /// - Option leaf slot
    /// - Collection leaf slot
    /// - Pick leaf slot
    /// - Cascading single slot
    /// - Cascading option slot
    /// - Cascading collection slot
    /// - Cascading pick slot
    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq)]
    pub struct AllSlotKindsEntity {
        /// Fixed invariant name.
        pub name: String,

        /// Scalar leaf slot.
        #[slot]
        pub scalar_leaf: String,

        /// Option leaf slot.
        #[slot(option)]
        pub option_leaf: Option<String>,

        /// Collection leaf slot.
        #[slot(collection)]
        pub collection_leaf: Vec<String>,

        /// Pick leaf slot.
        #[slot(pick)]
        pub pick_leaf: Vec<String>,

        /// Cascading single child formwork.
        #[slot(cascade)]
        pub single_cascade: DocumentedChild,

        /// Cascading optional child formwork.
        #[slot(option, cascade)]
        pub option_cascade: Option<DocumentedChild>,

        /// Cascading collection of child formworks.
        #[slot(collection, cascade)]
        pub collection_cascade: Vec<DocumentedChild>,

        /// Cascading pick slot selecting child formworks.
        #[slot(pick, cascade)]
        pub pick_cascade: Vec<DocumentedChild>,
    }

    /// A formwork entity without the builder option to test bare HKD mode generation under `#![deny(missing_docs)]`.
    #[formwork(traverse = "dfs")]
    #[derive(Clone, Debug, PartialEq)]
    pub struct NoBuilderEntity {
        /// Fixed metadata.
        pub meta: String,
        /// Scalar choice slot.
        #[slot]
        pub choice_slot: String,
        /// Optional choice slot.
        #[slot(option)]
        pub opt_slot: Option<String>,
    }

    /// A formwork entity testing derive macro forwarding to mode markers under `#![deny(missing_docs)]`.
    #[formwork(builder, derives(PartialEq, Eq, Hash))]
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct DerivesForwardingEntity {
        /// Unique key.
        pub key: String,
        /// Slot field.
        #[slot]
        pub tag: String,
    }
}

#[test]
fn test_missing_docs_entity_instantiation_and_resolution() {
    use documented_entities::*;

    let child1: DocumentedChildFormwork = DocumentedChild::builder()
        .child_id("c1".to_string())
        .detail(SingleChoice::new("Detail", vec!["D1".to_string()]))
        .build();

    let child2: DocumentedChild = DocumentedChild::builder()
        .child_id("c2".to_string())
        .detail("FixedDetail".to_string())
        .build();

    let mut wet: AllSlotKindsEntityFormwork = AllSlotKindsEntity::builder()
        .name("Entity-1".to_string())
        .scalar_leaf(SingleChoice::new("Scalar", vec!["S1".to_string()]))
        .option_leaf(Some(SingleChoice::new("Option", vec!["O1".to_string()])))
        .with_collection_leaf(SingleChoice::new("Collection", vec!["Col1".to_string()]))
        .pick_leaf(
            MultiChoice::new("Pick", vec!["P1".to_string(), "P2".to_string()])
                .with_cardinality(Cardinality::Exact(1)),
        )
        .single_cascade(child1)
        .option_cascade(Some(child2.clone()))
        .with_collection_cascade(child2.clone())
        .pick_cascade(
            MultiChoice::new("Pick Child", vec![child2])
                .with_cardinality(Cardinality::Exact(1)),
        )
        .build();

    assert!(!wet.is_complete());
    wet.walk_choices(&mut AutoSelectFirst);
    assert!(wet.is_complete());

    let dry: AllSlotKindsEntity = wet.resolve().expect("resolution succeeds");
    assert_eq!(dry.name, "Entity-1");
    assert_eq!(dry.scalar_leaf, "S1");
    assert_eq!(dry.option_leaf, Some("O1".to_string()));
    assert_eq!(dry.collection_leaf, vec!["Col1".to_string()]);
    assert_eq!(dry.pick_leaf, vec!["P1".to_string()]);
    assert_eq!(dry.single_cascade.detail, "D1");
    assert_eq!(dry.option_cascade.unwrap().detail, "FixedDetail");
    assert_eq!(dry.collection_cascade.len(), 1);
    assert_eq!(dry.pick_cascade.len(), 1);
}

#[test]
fn test_mode_trait_and_associated_types_accessibility() {
    use documented_entities::*;

    // 1. Verify Mode traits exist and can be bound on generic functions
    fn assert_implements_mode<T: AllSlotKindsEntityMode>() {}
    assert_implements_mode::<AllSlotKindsEntityWet>();
    assert_implements_mode::<AllSlotKindsEntityDry>();

    fn assert_child_mode<T: DocumentedChildMode>() {}
    assert_child_mode::<DocumentedChildWet>();
    assert_child_mode::<DocumentedChildDry>();

    fn assert_no_builder_mode<T: NoBuilderEntityMode>() {}
    assert_no_builder_mode::<NoBuilderEntityWet>();
    assert_no_builder_mode::<NoBuilderEntityDry>();

    fn assert_derives_mode<T: DerivesForwardingEntityMode>() {}
    assert_derives_mode::<DerivesForwardingEntityWet>();
    assert_derives_mode::<DerivesForwardingEntityDry>();

    // 2. Verify all associated types exist for both Wet and Dry modes
    // Scalar Leaf:
    type WetScalar = <AllSlotKindsEntityWet as AllSlotKindsEntityMode>::ScalarLeaf;
    type DryScalar = <AllSlotKindsEntityDry as AllSlotKindsEntityMode>::ScalarLeaf;
    fn check_scalar_wet(_: WetScalar) {}
    fn check_scalar_dry(_: DryScalar) {}
    check_scalar_wet(formwork::SingleSlot::FixedWet("wet".to_string()));
    check_scalar_dry("dry".to_string());

    // Option Leaf:
    type WetOption = <AllSlotKindsEntityWet as AllSlotKindsEntityMode>::OptionLeaf;
    type DryOption = <AllSlotKindsEntityDry as AllSlotKindsEntityMode>::OptionLeaf;
    fn check_option_wet(_: WetOption) {}
    fn check_option_dry(_: DryOption) {}
    check_option_wet(formwork::SingleSlot::FixedWet("opt".to_string()));
    check_option_dry("opt".to_string());

    // Collection Leaf:
    type WetCollection = <AllSlotKindsEntityWet as AllSlotKindsEntityMode>::CollectionLeaf;
    type DryCollection = <AllSlotKindsEntityDry as AllSlotKindsEntityMode>::CollectionLeaf;
    fn check_collection_wet(_: WetCollection) {}
    fn check_collection_dry(_: DryCollection) {}
    check_collection_wet(vec![formwork::SingleSlot::FixedWet("c".to_string())]);
    check_collection_dry(vec!["c".to_string()]);

    // Pick Leaf:
    type WetPick = <AllSlotKindsEntityWet as AllSlotKindsEntityMode>::PickLeaf;
    type DryPick = <AllSlotKindsEntityDry as AllSlotKindsEntityMode>::PickLeaf;
    fn check_pick_wet(_: WetPick) {}
    fn check_pick_dry(_: DryPick) {}
    check_pick_wet(formwork::MultiSlot::FixedWet(vec!["p".to_string()]));
    check_pick_dry(vec!["p".to_string()]);

    // Cascading slots:
    type WetSingleCascade = <AllSlotKindsEntityWet as AllSlotKindsEntityMode>::SingleCascade;
    type DrySingleCascade = <AllSlotKindsEntityDry as AllSlotKindsEntityMode>::SingleCascade;
    fn check_cascade_single_wet(_: WetSingleCascade) {}
    fn check_cascade_single_dry(_: DrySingleCascade) {}
    check_cascade_single_wet(formwork::SingleSlot::FixedWet(
        DocumentedChildFormwork {
            child_id: "c".to_string(),
            detail: formwork::SingleSlot::FixedWet("d".to_string()),
        },
    ));
    check_cascade_single_dry(DocumentedChild {
        child_id: "c".to_string(),
        detail: "d".to_string(),
    });
}

#[test]
fn test_no_builder_entity_under_missing_docs() {
    use documented_entities::*;

    let mut wet = NoBuilderEntityFormwork {
        meta: "meta-info".to_string(),
        choice_slot: SingleChoice::new("Pick one", vec!["A".to_string(), "B".to_string()]).into(),
        opt_slot: Some(SingleChoice::new("Pick opt", vec!["X".to_string()]).into()),
    };

    assert!(!wet.is_complete());
    wet.walk_choices(&mut AutoSelectFirst);
    assert!(wet.is_complete());

    let dry: NoBuilderEntity = wet.resolve().expect("resolves without builder");
    assert_eq!(dry.meta, "meta-info");
    assert_eq!(dry.choice_slot, "A");
    assert_eq!(dry.opt_slot, Some("X".to_string()));
}

#[test]
fn test_derives_forwarding_on_marker_structs() {
    use documented_entities::*;
    use std::collections::HashSet;

    // Verify Hash and Eq derives were forwarded to marker structs
    let mut wet_set = HashSet::new();
    wet_set.insert(DerivesForwardingEntityWet);
    assert_eq!(wet_set.len(), 1);

    let mut dry_set = HashSet::new();
    dry_set.insert(DerivesForwardingEntityDry);
    assert_eq!(dry_set.len(), 1);
}
