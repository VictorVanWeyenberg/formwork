//! Integration test suite for Test Case 18: SRD 5.1 Hill Dwarf Character Creation.
//!
//! Validates:
//! 1. Construction of an unresolved `RaceSchematic` with fixed traits and choice traits.
//! 2. Incompleteness verification and resolution failure reporting `SchematicError::UnresolvedChoice`.
//! 3. Manual selection of Option 1 (Brewer's Supplies) satisfying all open choices.
//! 4. Resolution into a watertight `Race` domain model.
//! 5. Verification of race name and traits via clean public getters (`name()`, `traits()`, `effects()`, `operation()`).
//! 6. Serde roundtrip JSON serialization and deserialization with structural equality.
//! 7. Automatic resolution via `AutoSelectFirst` choosing Option 0 (Smith's Tools).

use schematic::Schematic;
use schematic::choice::{MultiChoice, SingleChoice};
use schematic::error::SchematicError;
use schematic::visitor::{AutoSelectFirst, ChoiceVisitor, VisitorFlow};
use stress_test::contentref::{RaceRef, Referable, SubraceRef, ToolRef};
use stress_test::damage::{DamageModulation, DamageType};
use stress_test::effect::{Effect, EffectId, EffectSchematic, SheetOperation};
use stress_test::proficiency::Proficiency;
use stress_test::race::{Race, RaceSchematic, Subrace};
use stress_test::roll::{
    RollDescription, SavingThrowAgainst, SavingThrowKind, VantageOnRoll, VantageStatus,
};
use stress_test::traits::{Trait, TraitSchematic};
use stress_test::valueref::{SheetRef, ValueRef};

/// Visitor to select a specific choice option by index across all encountered `SingleChoice`s.
struct SelectOption(usize);

impl<T> ChoiceVisitor<T> for SelectOption {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        if !choice.is_made() && self.0 < choice.options().len() {
            choice
                .select(self.0)
                .expect("failed to select choice option");
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

/// Constructs the fixed SRD 5.1 trait "Dwarven Resilience".
///
/// Grants advantage on saving throws against poison and resistance against poison damage.
fn dwarven_resilience() -> Trait {
    Trait::builder()
        .title("Dwarven Resilience".to_string())
        .description(
            "You have advantage on saving throws against poison, and you have resistance against poison damage."
                .to_string(),
        )
        .with_effects(Effect::new(SheetOperation::DamageModulation(
            DamageModulation::Resistance(DamageType::Poison),
        )))
        .with_effects(Effect::new(SheetOperation::VantageOnRoll(
            VantageOnRoll::new(
                VantageStatus::Advantage,
                RollDescription::SavingThrow(SavingThrowKind::Against(
                    SavingThrowAgainst::DamageType(DamageType::Poison),
                )),
            ),
        )))
        .build()
}

/// Constructs the choice SRD 5.1 trait "Dwarven Tool Proficiency".
///
/// Offers a `SingleChoice` between:
/// - Option 0: Smith's Tools (`srd/tool/smithstools`)
/// - Option 1: Brewer's Supplies (`srd/tool/brewersupplies`)
/// - Option 2: Mason's Tools (`srd/tool/masonstools`)
fn dwarven_tool_proficiency() -> TraitSchematic {
    let smiths_tools: ToolRef = "srd/tool/smithstools".parse().unwrap();
    let brewers_supplies: ToolRef = "srd/tool/brewersupplies".parse().unwrap();
    let masons_tools: ToolRef = "srd/tool/masonstools".parse().unwrap();

    let effect = EffectSchematic::builder()
        .id(EffectId::default())
        .operation(SingleChoice::new(
            "Dwarven Tool Proficiency",
            vec![
                SheetOperation::Proficiency(Proficiency::Tool(smiths_tools)),
                SheetOperation::Proficiency(Proficiency::Tool(brewers_supplies)),
                SheetOperation::Proficiency(Proficiency::Tool(masons_tools)),
            ],
        ))
        .build();

    TraitSchematic::builder()
        .title("Dwarven Tool Proficiency".to_string())
        .description(
            "You have proficiency with the artisan's tools of your choice: smith's tools, brewer's supplies, or mason's tools."
                .to_string(),
        )
        .with_effects(effect)
        .build()
}

/// Constructs the SRD 5.1 Hill Dwarf subrace trait "Dwarven Toughness".
///
/// Grants a Max HP increase by 1 per character level (`ValueRef::Sheet(SheetRef::Level)`).
fn dwarven_toughness() -> Trait {
    Trait::builder()
        .title("Dwarven Toughness".to_string())
        .description(
            "Your hit point maximum increases by 1, and it increases by 1 every time you gain a level."
                .to_string(),
        )
        .with_effects(Effect::new(SheetOperation::ModifyMaxHitPoint(
            ValueRef::Sheet(SheetRef::Level),
        )))
        .build()
}

/// Constructs the Hill Dwarf `Subrace` domain model.
fn hill_dwarf_subrace() -> Subrace {
    Subrace::new(
        "srd/subrace/hilldwarf".parse::<SubraceRef>().unwrap(),
        "Hill Dwarf".to_string(),
        "As a hill dwarf, you have keen senses, deep intuition, and remarkable resilience."
            .to_string(),
        "srd/race/dwarf".parse::<RaceRef>().unwrap(),
        vec![dwarven_toughness()],
    )
}

/// Constructs an unresolved `RaceSchematic` for a Hill Dwarf character.
fn build_dwarf_schematic() -> RaceSchematic {
    RaceSchematic::builder()
        .id("srd/race/dwarf".parse::<RaceRef>().unwrap())
        .name("Dwarf")
        .description(
            "Bold and hardy, dwarves are known as skilled warriors, miners, and workers of stone and metal."
                .to_string(),
        )
        .with_traits(dwarven_resilience())
        .with_traits(dwarven_tool_proficiency())
        .with_traits(dwarven_toughness())
        .build()
}

#[test]
fn test_hill_dwarf_character_creation_suite() {
    // 1. Construct dwarf_bp: RaceSchematic -> assert !is_complete(), resolve() fails with UnresolvedChoice
    let mut dwarf_bp: RaceSchematic = build_dwarf_schematic();
    assert!(!dwarf_bp.is_complete());

    let resolve_err = dwarf_bp.clone().resolve().unwrap_err();
    assert_eq!(
        resolve_err,
        SchematicError::UnresolvedChoice {
            prompt: "Dwarven Tool Proficiency".to_string()
        }
    );

    // 2. Select Option 1 ("Brewer's Supplies") -> assert dwarf_bp.is_complete()
    dwarf_bp.walk_choices(&mut SelectOption(1));
    assert!(dwarf_bp.is_complete());

    // 3. Resolve into watertight domain model
    let resolved_race: Race = dwarf_bp.resolve().unwrap();

    // 4. Assert resolved_race.name() == "Dwarf"
    assert_eq!(resolved_race.name(), "Dwarf");

    // 5. Assert traits include Brewer's Supplies proficiency and Dwarven Resilience using clean getters
    let traits = resolved_race.traits();
    assert_eq!(traits.len(), 3);

    // Assert Dwarven Resilience is present with poison resistance and advantage on poison saves
    let resilience_trait = traits
        .iter()
        .find(|t| t.title() == "Dwarven Resilience")
        .expect("Dwarven Resilience trait must be present");
    let resilience_effects = resilience_trait.effects();
    assert!(
        resilience_effects.iter().any(|e| matches!(
            e.operation(),
            SheetOperation::DamageModulation(DamageModulation::Resistance(DamageType::Poison))
        )),
        "Dwarven Resilience must grant poison damage resistance"
    );

    // Assert Brewer's Supplies tool proficiency is present
    let tool_trait = traits
        .iter()
        .find(|t| t.title() == "Dwarven Tool Proficiency")
        .expect("Dwarven Tool Proficiency trait must be present");
    let brewers_supplies_ref = "srd/tool/brewersupplies".parse::<ToolRef>().unwrap();
    let has_brewers_supplies = tool_trait.effects().iter().any(|e| match e.operation() {
        SheetOperation::Proficiency(Proficiency::Tool(t)) => t == &brewers_supplies_ref,
        _ => false,
    });
    assert!(
        has_brewers_supplies,
        "Dwarven Tool Proficiency must grant Brewer's Supplies proficiency"
    );

    // Assert Dwarven Toughness is present with HP modification by Level
    let toughness_trait = traits
        .iter()
        .find(|t| t.title() == "Dwarven Toughness")
        .expect("Dwarven Toughness trait must be present");
    assert!(
        toughness_trait.effects().iter().any(|e| matches!(
            e.operation(),
            SheetOperation::ModifyMaxHitPoint(ValueRef::Sheet(SheetRef::Level))
        )),
        "Dwarven Toughness must increase max hit points by level"
    );

    // 6. Serde roundtrip: serialize resolved Race to JSON and deserialize back; assert structural equality
    let json = serde_json::to_string_pretty(&resolved_race).unwrap();
    let deserialized_race: Race = serde_json::from_str(&json).unwrap();
    assert_eq!(resolved_race, deserialized_race);

    // 7. Test walk_choices(&mut AutoSelectFirst) selects Option 0 (Smith's Tools) and resolves cleanly
    let mut auto_dwarf_bp = build_dwarf_schematic();
    assert!(!auto_dwarf_bp.is_complete());

    auto_dwarf_bp.walk_choices(&mut AutoSelectFirst);
    assert!(auto_dwarf_bp.is_complete());

    let auto_resolved_race: Race = auto_dwarf_bp.resolve().unwrap();
    assert_eq!(auto_resolved_race.name(), "Dwarf");

    let smiths_tools_ref = "srd/tool/smithstools".parse::<ToolRef>().unwrap();
    let auto_tool_trait = auto_resolved_race
        .traits()
        .iter()
        .find(|t| t.title() == "Dwarven Tool Proficiency")
        .expect("Dwarven Tool Proficiency trait must be present");
    let has_smiths_tools = auto_tool_trait
        .effects()
        .iter()
        .any(|e| match e.operation() {
            SheetOperation::Proficiency(Proficiency::Tool(t)) => t == &smiths_tools_ref,
            _ => false,
        });
    assert!(
        has_smiths_tools,
        "AutoSelectFirst must select Option 0 (Smith's Tools)"
    );

    // Verify Hill Dwarf Subrace reference and metadata
    let subrace = hill_dwarf_subrace();
    assert_eq!(subrace.reference().to_string(), "srd/subrace/hilldwarf");
    assert_eq!(subrace.for_race().to_string(), "srd/race/dwarf");
    assert_eq!(subrace.name(), "Hill Dwarf");
    assert_eq!(subrace.traits().len(), 1);
    assert_eq!(subrace.traits()[0].title(), "Dwarven Toughness");

    let subrace_json = serde_json::to_string_pretty(&subrace).unwrap();
    let deserialized_subrace: Subrace = serde_json::from_str(&subrace_json).unwrap();
    assert_eq!(subrace, deserialized_subrace);
}
