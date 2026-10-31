//! Integration tests for struct permutations (Test Cases 7–10):
//! - Test Case 7: Without `builder` (Manual Struct Instantiation)
//! - Test Case 8: Mixed Fields: Non-slot Scalars and Non-slot Collections
//! - Test Case 9: Derives & Serde JSON Roundtrips
//! - Test Case 10: Doc Comment Preservation

use formwork::{Formwork, Slot, choice::SingleChoice, visitor::AutoSelectFirst};
use formwork_macro::formwork;

// ============================================================================
// Test Case 7: Without `builder` (Manual Struct Instantiation)
// ============================================================================

#[formwork]
#[derive(Clone, Debug, PartialEq)]
pub struct ManualRecord {
    pub id: u64,
    #[slot]
    pub val: String,
    #[slot(collection)]
    pub items: Vec<String>,
}

#[test]
fn test_manual_struct_instantiation_without_builder() {
    // 1. Manually instantiate ManualRecordFormwork using formwork::Slot
    let mut formwork: ManualRecordFormwork = ManualRecord {
        id: 99,
        val: Slot::Choose(SingleChoice::new(
            "Pick val",
            vec!["V1".to_string(), "V2".to_string()],
        )),
        items: vec![
            Slot::FixedWet("I1".to_string()),
            Slot::Choose(SingleChoice::new(
                "Pick items",
                vec!["I2".to_string(), "I3".to_string()],
            )),
        ],
    };

    // 2. Test completeness before selection
    assert!(!formwork.is_complete());
    assert!(formwork.clone().resolve().is_err());

    // 3. Visitor walk with AutoSelectFirst
    formwork.walk_choices(&mut AutoSelectFirst);
    assert!(formwork.is_complete());

    // 4. Resolution
    let resolved: ManualRecord = formwork.resolve().expect("Resolution should succeed");
    assert_eq!(resolved.id, 99);
    assert_eq!(resolved.val, "V1");
    assert_eq!(resolved.items, vec!["I1".to_string(), "I2".to_string()]);

    // 5. Test ManualRecordFormwork::from(manual_resolved) conversion
    let converted = ManualRecordFormwork::from(resolved.clone());
    assert!(converted.is_complete());
    assert_eq!(converted.id, 99);
    assert_eq!(converted.val, Slot::FixedDry("V1".to_string()));
    assert_eq!(
        converted.items,
        vec![
            Slot::FixedDry("I1".to_string()),
            Slot::FixedDry("I2".to_string()),
        ]
    );
}

// ============================================================================
// Test Case 8: Mixed Fields: Non-slot Scalars and Non-slot Collections
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct MixedFieldsEntity {
    pub id: u64,
    pub label: String,
    pub tags: Vec<String>,
    #[slot]
    pub dynamic_option: String,
}

#[test]
fn test_mixed_fields_preservation_and_builder_mutators() {
    // Verify builder generates `with_<field>` for non-slot Vec<String> accepting `impl Into<String>`.
    // We pass both &str slices ("tag_1", "tag_2") and String instances ("tag_3".to_string())
    // to prove `impl Into<String>` is accepted.
    // Also verify non-slot scalar fields `id` and `label` (with Into<String>).
    let mut formwork: MixedFieldsEntityFormwork = MixedFieldsEntity::builder()
        .id(10u64)
        .label("Alpha")
        .with_tags("tag_1")
        .with_tags("tag_2")
        .with_tags(String::from("tag_3"))
        .dynamic_option(SingleChoice::new(
            "Pick dynamic option",
            vec!["OptionA".to_string(), "OptionB".to_string()],
        ))
        .build();

    assert!(!formwork.is_complete());
    formwork.walk_choices(&mut AutoSelectFirst);
    assert!(formwork.is_complete());

    // Verify non-slot scalar and collection fields are preserved verbatim across resolution
    let resolved: MixedFieldsEntity = formwork.resolve().expect("Resolution should succeed");
    assert_eq!(resolved.id, 10);
    assert_eq!(resolved.label, "Alpha");
    assert_eq!(
        resolved.tags,
        vec![
            "tag_1".to_string(),
            "tag_2".to_string(),
            "tag_3".to_string()
        ]
    );
    assert_eq!(resolved.dynamic_option, "OptionA");
}

// ============================================================================
// Test Case 9: Derives & Serde JSON Roundtrips
// ============================================================================

#[formwork(
    builder,
    derives(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SerdeEntity {
    pub id: u32,
    #[slot]
    pub choice: String,
    #[slot(collection)]
    pub tags: Vec<String>,
}

#[test]
fn test_serde_json_roundtrip_pending_and_resolved() {
    // 1. Construct formwork with pending choices
    let formwork: SerdeEntityFormwork = SerdeEntity::builder()
        .id(42u32)
        .choice(SingleChoice::new(
            "Choose weapon",
            vec!["Sword".to_string(), "Bow".to_string()],
        ))
        .with_tags("primary".to_string())
        .build();

    assert!(!formwork.is_complete());

    // 2. Serialize/deserialize formwork with pending choices; verify choice state preserved
    let json =
        serde_json::to_string(&formwork).expect("Serialization of formwork should succeed");
    let mut deserialized_bp: SerdeEntityFormwork =
        serde_json::from_str(&json).expect("Deserialization of formwork should succeed");

    assert_eq!(formwork, deserialized_bp);
    assert!(!deserialized_bp.is_complete());

    // 3. Satisfy choices on deserialized formwork
    deserialized_bp.walk_choices(&mut AutoSelectFirst);
    assert!(deserialized_bp.is_complete());

    // 4. Resolve formwork into watertight domain model
    let resolved: SerdeEntity = deserialized_bp
        .resolve()
        .expect("Resolution of deserialized formwork should succeed");
    assert_eq!(resolved.id, 42);
    assert_eq!(resolved.choice, "Sword");
    assert_eq!(resolved.tags, vec!["primary".to_string()]);

    // 5. Serialize/deserialize resolved struct; verify structural equality
    let resolved_json =
        serde_json::to_string(&resolved).expect("Serialization of resolved struct should succeed");
    let deserialized_resolved: SerdeEntity = serde_json::from_str(&resolved_json)
        .expect("Deserialization of resolved struct should succeed");

    assert_eq!(resolved, deserialized_resolved);
}

// ============================================================================
// Test Case 10: Doc Comment Preservation
// ============================================================================

mod documented_module {
    #![deny(missing_docs)]

    use formwork::Formwork;
    use formwork::choice::SingleChoice;
    use formwork_macro::formwork;

    /// Root entity doc comment documenting the primary record.
    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq)]
    pub struct DocumentedEntity {
        /// Primary numerical key identifier.
        pub id: u64,
        /// Primary choice slot for entity attribute.
        #[slot]
        pub slot_item: String,
    }

    #[test]
    fn test_documented_entity_compiles_under_deny_missing_docs() {
        let mut bp: DocumentedEntityFormwork = DocumentedEntity::builder()
            .id(100u64)
            .slot_item(SingleChoice::new(
                "Select item",
                vec!["ItemA".to_string(), "ItemB".to_string()],
            ))
            .build();

        assert!(!bp.is_complete());
        bp.walk_choices(&mut formwork::visitor::AutoSelectFirst);
        assert!(bp.is_complete());

        let resolved: DocumentedEntity = bp.resolve().expect("Resolution should succeed");
        assert_eq!(resolved.id, 100);
        assert_eq!(resolved.slot_item, "ItemA");
    }
}
