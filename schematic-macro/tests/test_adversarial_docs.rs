use schematic::Schematic;
use schematic::choice::{Cardinality, MultiChoice, SingleChoice};
use schematic::visitor::AutoSelectFirst;
use schematic_macro::schematic;

/// # Adversarial Nested Child Struct
///
/// This child struct tests cascading slots inside parent structs.
#[schematic(builder, derives(Clone, Debug, PartialEq))]
#[derive(Clone, Debug, PartialEq)]
pub struct ChildEntity {
    /// Unique identifier for child.
    pub id: String,

    /// Child property slot.
    #[slot]
    pub details: String,
}

/// # Adversarial Complex Schematic
///
/// This struct is deliberately adversarial:
/// - Multiple paragraphs of doc comments.
/// - Markdown headings, lists, code samples.
/// - Mixes **all** slot kinds and non-slot fields.
///
/// ```rust
/// // Code sample inside struct doc
/// let x = 42;
/// ```
#[schematic(builder, derives(Clone, Debug, PartialEq))]
#[derive(Clone, Debug, PartialEq)]
pub struct AdversarialEntity {
    /// Non-slot mandatory identifier.
    ///
    /// Must be known upfront.
    pub entity_id: String,

    /// Non-slot collection of tags.
    ///
    /// Uses `.with_tags(...)` mutator.
    pub tags: Vec<String>,

    /// A single slot field holding a single string or choice.
    #[slot]
    pub single_slot: String,

    /// A collection slot holding multiple strings or multi-choices.
    #[slot(collection)]
    pub collection_slot: Vec<String>,

    /// A pick slot holding items picked with cardinality.
    #[slot(pick)]
    pub pick_slot: Vec<String>,

    /// A cascading single child schematic.
    #[slot(cascade)]
    pub single_cascade: ChildEntity,

    /// A cascading collection of child schematics.
    #[slot(collection, cascade)]
    pub collection_cascade: Vec<ChildEntity>,
}

/// A struct without builder or field doc comments to test edge cases.
#[schematic(derives(Clone, Debug, PartialEq))]
#[derive(Clone, Debug, PartialEq)]
pub struct NoBuilderEntity {
    #[slot]
    pub val: String,
}

#[test]
fn test_adversarial_entity_builder_and_resolution() {
    let mut blueprint: AdversarialEntitySchematic = AdversarialEntity::builder()
        .entity_id("ent-001")
        .with_tags("alpha".to_string())
        .with_tags("beta".to_string())
        .single_slot(SingleChoice::new(
            "Select role",
            vec!["Admin".to_string(), "User".to_string()],
        ))
        .with_collection_slot(MultiChoice::new(
            "Select permissions",
            vec!["Read".to_string(), "Write".to_string()],
        ))
        .with_collection_slot("Execute".to_string())
        .with_pick_slot(
            MultiChoice::new(
                "Pick 2 features",
                vec!["F1".to_string(), "F2".to_string(), "F3".to_string()],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )
        .single_cascade(
            ChildEntitySchematic::builder()
                .id("child-1")
                .details(SingleChoice::new(
                    "Select details",
                    vec!["DetailA".to_string(), "DetailB".to_string()],
                ))
                .build(),
        )
        .with_collection_cascade(
            ChildEntitySchematic::builder()
                .id("child-2")
                .details("ResolvedDetail")
                .build(),
        )
        .build();

    // Incomplete while choices remain
    assert!(!blueprint.is_complete());

    // Traverse and auto-select choices across parent and child levels
    blueprint.walk_choices(&mut AutoSelectFirst);
    assert!(blueprint.is_complete());

    // Resolve into watertight domain model
    let resolved: AdversarialEntity = blueprint.resolve().expect("Resolution should succeed");
    assert_eq!(resolved.entity_id, "ent-001");
    assert_eq!(resolved.tags, vec!["alpha", "beta"]);
    assert_eq!(resolved.single_slot, "Admin");
    assert_eq!(resolved.collection_slot, vec!["Read", "Execute"]);
    assert_eq!(resolved.pick_slot, vec!["F1", "F2"]);
    assert_eq!(resolved.single_cascade.id, "child-1");
    assert_eq!(resolved.single_cascade.details, "DetailA");
    assert_eq!(resolved.collection_cascade.len(), 1);
    assert_eq!(resolved.collection_cascade[0].id, "child-2");
    assert_eq!(resolved.collection_cascade[0].details, "ResolvedDetail");
}

#[test]
fn test_no_builder_entity_manual_construction_and_resolution() {
    let mut blueprint: NoBuilderEntitySchematic = NoBuilderEntity {
        val: SingleChoice::new("Pick value", vec!["V1".to_string(), "V2".to_string()]).into(),
    };

    assert!(!blueprint.is_complete());
    blueprint.walk_choices(&mut AutoSelectFirst);
    assert!(blueprint.is_complete());

    let resolved: NoBuilderEntity = blueprint.resolve().unwrap();
    assert_eq!(resolved.val, "V1");
}
