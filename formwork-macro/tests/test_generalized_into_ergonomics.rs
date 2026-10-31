//! Adversarial integration test suite for Generalized `Into<T>` slot ergonomics,
//! primitive widening, custom domain types, pick arrays, choice string literals,
//! and cascading slot preservation.

use formwork::{
    choice::{Cardinality, MultiChoice, SingleChoice},
    formwork,
    visitor::AutoSelectFirst,
    Formwork, Slot,
};


// ============================================================================
// 1. Primitive Widening in Resolved and Choice Modes
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimitiveWideningTarget {
    #[slot]
    pub u32_val: u32,
    #[slot]
    pub u64_val: u64,
    #[slot]
    pub i64_val: i64,
    #[slot]
    pub f64_val: u64, // using u64 to test integer widening
}

#[test]
fn test_primitive_widening_resolved_mode() {
    // Widening: u16 -> u32, u32 -> u64, i16 -> i64, u8 -> u64
    let target: PrimitiveWideningTarget = PrimitiveWideningTarget::builder()
        .u32_val(100u16)
        .u64_val(200u32)
        .i64_val(-50i16)
        .f64_val(255u8)
        .build();

    assert_eq!(target.u32_val, 100u32);
    assert_eq!(target.u64_val, 200u64);
    assert_eq!(target.i64_val, -50i64);
    assert_eq!(target.f64_val, 255u64);
}

#[test]
fn test_primitive_widening_in_choice_wet_mode() {
    // SingleChoice::new allows passing arrays of widened primitives!
    let choice_u32 = SingleChoice::new("Pick u32", [10u16, 20u16, 30u16]);
    let choice_u64 = SingleChoice::new("Pick u64", [1000u32, 2000u32]);

    let mut bp = PrimitiveWideningTargetFormwork::builder()
        .u32_val(choice_u32)
        .u64_val(choice_u64)
        .i64_val(12345i64)
        .f64_val(99999u64)
        .build();

    assert!(!bp.is_complete());
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve widened choices");
    assert_eq!(resolved.u32_val, 10u32);
    assert_eq!(resolved.u64_val, 1000u64);
    assert_eq!(resolved.i64_val, 12345i64);
    assert_eq!(resolved.f64_val, 99999u64);
}

// ============================================================================
// 2. Custom Domain Newtypes and Enums with From / Into
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmorTier {
    Cloth,
    Leather,
    Mail,
    Plate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArmorSpec {
    pub name: String,
    pub defense: u32,
}

impl From<ArmorTier> for ArmorSpec {
    fn from(tier: ArmorTier) -> Self {
        match tier {
            ArmorTier::Cloth => ArmorSpec {
                name: "Cloth Robes".into(),
                defense: 1,
            },
            ArmorTier::Leather => ArmorSpec {
                name: "Leather Jerkin".into(),
                defense: 5,
            },
            ArmorTier::Mail => ArmorSpec {
                name: "Ring Mail".into(),
                defense: 12,
            },
            ArmorTier::Plate => ArmorSpec {
                name: "Full Plate".into(),
                defense: 25,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatBonus(pub i32);

impl From<i16> for StatBonus {
    fn from(v: i16) -> Self {
        StatBonus(v as i32)
    }
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainTarget {
    #[slot]
    pub armor: ArmorSpec,
    #[slot]
    pub bonus: StatBonus,
}

#[test]
fn test_domain_types_from_into_in_resolved_mode() {
    let domain: DomainTarget = DomainTarget::builder()
        .armor(ArmorTier::Plate) // ArmorTier -> ArmorSpec via Into
        .bonus(7i16)            // i16 -> StatBonus via Into
        .build();

    assert_eq!(
        domain.armor,
        ArmorSpec {
            name: "Full Plate".into(),
            defense: 25,
        }
    );
    assert_eq!(domain.bonus, StatBonus(7));
}

#[test]
fn test_domain_types_in_choice_wet_mode() {
    // SingleChoice with domain enum variants constructing ArmorSpec candidates
    let armor_choice: SingleChoice<ArmorSpec> = SingleChoice::new(
        "Choose armor tier",
        [ArmorTier::Cloth, ArmorTier::Mail],
    );
    let bonus_choice: SingleChoice<StatBonus> = SingleChoice::new("Bonus", [2i16, 4i16]);

    let mut bp = DomainTargetFormwork::builder()
        .armor(armor_choice)
        .bonus(bonus_choice)
        .build();

    assert!(!bp.is_complete());
    bp.armor.as_choice_mut().unwrap().select(1).unwrap();
    bp.armor.as_choice_mut().unwrap().confirm().unwrap();
    bp.bonus.as_choice_mut().unwrap().select(0).unwrap();
    bp.bonus.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("domain target resolves");
    assert_eq!(
        resolved.armor,
        ArmorSpec {
            name: "Ring Mail".into(),
            defense: 12,
        }
    );
    assert_eq!(resolved.bonus, StatBonus(2));
}

// ============================================================================
// 3. Arrays and Widened Items into Pick Slots
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickArraysTarget {
    #[slot(pick)]
    pub u32_scores: Vec<u32>,
    #[slot(pick)]
    pub string_tags: Vec<String>,
}

#[test]
fn test_pick_slots_with_widened_arrays_resolved_mode() {
    // Array of u16 directly into Pick slot for Vec<u32>
    // Array of string literals directly into Pick slot for Vec<String>
    let resolved: PickArraysTarget = PickArraysTarget::builder()
        .u32_scores([10u16, 20u16, 30u16]) // [u16; 3] -> Vec<u32>
        .string_tags(["alpha", "beta", "gamma"]) // [&str; 3] -> Vec<String>
        .build();

    assert_eq!(resolved.u32_scores, vec![10u32, 20u32, 30u32]);
    assert_eq!(resolved.string_tags, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn test_pick_slots_with_widened_arrays_wet_mode() {
    // In wet mode, raw [T; N] with Into conversion wraps into MultiSlot::FixedWet
    let bp = PickArraysTargetFormwork::builder()
        .u32_scores([5u16, 15u16]) // [u16; 2]
        .string_tags(["one", "two"]) // [&str; 2]
        .build();

    assert!(bp.is_complete());
    let resolved = bp.resolve().expect("must resolve fixed pick slots");
    assert_eq!(resolved.u32_scores, vec![5u32, 15u32]);
    assert_eq!(resolved.string_tags, vec!["one", "two"]);
}

#[test]
fn test_pick_slots_with_widened_vec_dry_and_wet() {
    // Vec<u16> into Pick slot
    let bp = PickArraysTargetFormwork::builder()
        .u32_scores(vec![100u16, 200u16])
        .string_tags(vec!["hello", "world"])
        .build();
    assert!(bp.is_complete());
    let resolved = bp.resolve().unwrap();
    assert_eq!(resolved.u32_scores, vec![100u32, 200u32]);
    assert_eq!(resolved.string_tags, vec!["hello", "world"]);

    let direct: PickArraysTarget = PickArraysTarget::builder()
        .u32_scores(vec![300u16])
        .string_tags(vec!["direct"])
        .build();
    assert_eq!(direct.u32_scores, vec![300u32]);
    assert_eq!(direct.string_tags, vec!["direct"]);
}

// ============================================================================
// 4. String Literals in Choices (SingleChoice and MultiChoice)
// ============================================================================

#[test]
fn test_string_literals_in_single_and_multi_choices() {
    // SingleChoice::new with [&str; 2] array yielding SingleChoice<String>
    let single: SingleChoice<String> = SingleChoice::new("Weapons", ["Sword", "Bow"]);
    assert_eq!(single.prompt(), "Weapons");
    assert_eq!(single.options(), &["Sword".to_string(), "Bow".to_string()]);

    // MultiChoice::new with [&str; 3] array yielding MultiChoice<String>
    let multi: MultiChoice<String> = MultiChoice::new("Spells", ["Fireball", "Heal", "Shield"]);
    assert_eq!(multi.prompt(), "Spells");
    assert_eq!(
        multi.options(),
        &["Fireball".to_string(), "Heal".to_string(), "Shield".to_string()]
    );

    // MultiChoice with vec![&str] yielding MultiChoice<String>
    let multi_vec: MultiChoice<String> = MultiChoice::new("Skills", vec!["Athletics", "Stealth"]);
    assert_eq!(
        multi_vec.options(),
        &["Athletics".to_string(), "Stealth".to_string()]
    );
}

// ============================================================================
// 5. Collection Slots with with_<field> and <field> using Into<T>
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionErgonomicsTarget {
    #[slot(collection)]
    pub names: Vec<String>,
    #[slot(collection)]
    pub ids: Vec<u32>,
}

#[test]
fn test_collection_slots_with_into_items_resolved_mode() {
    let resolved: CollectionErgonomicsTarget = CollectionErgonomicsTarget::builder()
        .with_names("Alice") // &str -> String
        .with_names(String::from("Bob")) // String -> String
        .names(["Charlie", "David"]) // [&str; 2]
        .with_ids(10u16) // u16 -> u32
        .with_ids(20u8) // u8 -> u32
        .ids([30u16, 40u16]) // [u16; 2] -> IntoIterator<Item: Into<u32>>
        .build();

    assert_eq!(
        resolved.names,
        vec!["Alice", "Bob", "Charlie", "David"]
    );
    assert_eq!(resolved.ids, vec![10, 20, 30, 40]);
}

#[test]
fn test_collection_slots_with_choices_wet_mode() {
    let c1: SingleChoice<String> = SingleChoice::new("Name 1", ["Hero", "Sidekick"]);
    let c2: SingleChoice<String> = SingleChoice::new("Name 2", ["Villain", "Henchman"]);

    let mut bp = CollectionErgonomicsTargetFormwork::builder()
        .with_names(c1)
        .with_names("FixedName".to_string())
        .names([c2])
        .with_ids(100u32)
        .build();

    assert!(!bp.is_complete());
    bp.walk_choices(&mut AutoSelectFirst);
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("collection resolves");
    assert_eq!(resolved.names, vec!["Hero", "FixedName", "Villain"]);
    assert_eq!(resolved.ids, vec![100]);
}

// ============================================================================
// 6. Cascading Pick Slots: Preservation of Pre-Resolved Children
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CascadeChildItem {
    #[slot]
    pub code: u32,
    #[slot]
    pub desc: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CascadeParentHolder {
    #[slot(pick, cascade)]
    pub children: Vec<CascadeChildItem>,
}

#[test]
fn test_cascading_pick_slot_preserves_preresolved_children() {
    // Build pre-resolved children in Resolved mode
    let child1: CascadeChildItem = CascadeChildItem::builder()
        .code(101u16) // primitive widening
        .desc("Item One") // &str -> String via Into
        .build();
    let child2: CascadeChildItem = CascadeChildItem::builder()
        .code(202u16)
        .desc("Item Two")
        .build();

    // 1. Pass array of dry children into wet parent
    let bp_array = CascadeParentHolderFormwork::builder()
        .children([child1.clone(), child2.clone()])
        .build();

    // CRITICAL INVARIANT: bp_array must be complete because child items are pre-resolved (Slot::FixedDry)
    assert!(
        bp_array.is_complete(),
        "Passing dry children into cascading pick slot must NOT degrade to wet"
    );

    match &bp_array.children {
        Slot::FixedDry(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], child1);
            assert_eq!(items[1], child2);
        }
        other => panic!("Expected Slot::FixedDry, got {:?}", other),
    }

    let resolved_from_array = bp_array.resolve().expect("resolves cleanly");
    assert_eq!(resolved_from_array.children, vec![child1.clone(), child2.clone()]);

    // 2. Pass Vec of dry children into Wet parent
    let bp_vec = CascadeParentHolderFormwork::builder()
        .children(vec![child1.clone(), child2.clone()])
        .build();
    assert!(bp_vec.is_complete());
    assert_eq!(bp_vec.resolve().unwrap().children, vec![child1.clone(), child2.clone()]);

    // 3. Direct Dry mode builder
    let direct_resolved: CascadeParentHolder = CascadeParentHolder::builder()
        .children([child1.clone(), child2.clone()])
        .build();
    assert_eq!(direct_resolved.children, vec![child1, child2]);
}

#[test]
fn test_cascading_pick_slot_with_wet_children_lifecycle() {
    // Build wet children containing pending choices
    let child_wet_1 = CascadeChildItemFormwork::builder()
        .code(SingleChoice::new("Code", [10u32, 20u32]))
        .desc("Desc1".to_string())
        .build();

    let child_wet_2 = CascadeChildItemFormwork::builder()
        .code(SingleChoice::new("Code2", [30u32, 40u32]))
        .desc("Desc2".to_string())
        .build();

    let mut parent_bp = CascadeParentHolderFormwork::builder()
        .children([child_wet_1, child_wet_2])
        .build();

    // Must be incomplete because children are wet
    assert!(!parent_bp.is_complete());

    match &parent_bp.children {
        Slot::FixedWet(items) => {
            assert_eq!(items.len(), 2);
        }
        other => panic!("Expected Slot::FixedWet, got {:?}", other),
    }

    // Traversal auto-selects choices inside wet children
    parent_bp.walk_choices(&mut AutoSelectFirst);
    assert!(parent_bp.is_complete());

    let resolved = parent_bp.resolve().expect("parent resolves after children choices");
    assert_eq!(resolved.children[0].code, 10);
    assert_eq!(resolved.children[0].desc, "Desc1");
    assert_eq!(resolved.children[1].code, 30);
    assert_eq!(resolved.children[1].desc, "Desc2");
}

// ============================================================================
// 7. MultiChoice with Arrays and Iterators directly in Pick Slots
// ============================================================================

#[test]
fn test_multi_choice_in_pick_slot() {
    let multi = MultiChoice::new(
        "Pick skills",
        ["Swordsmanship", "Archery", "Magic", "Sneak"],
    )
    .with_cardinality(Cardinality::Exact(2));


    let mut bp = PickArraysTargetFormwork::builder()
        .u32_scores([100u32])
        .string_tags(multi)
        .build();


    assert!(!bp.is_complete());

    // Select options 0 and 2
    bp.string_tags.as_choice_mut().unwrap().select(0).unwrap();
    bp.string_tags.as_choice_mut().unwrap().select(2).unwrap();
    bp.string_tags.as_choice_mut().unwrap().confirm().unwrap();
    assert!(bp.is_complete());

    let resolved = bp.resolve().expect("must resolve selected multi choice");
    assert_eq!(
        resolved.string_tags,
        vec!["Swordsmanship".to_string(), "Magic".to_string()]
    );
}
