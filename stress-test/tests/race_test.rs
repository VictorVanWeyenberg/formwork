//! End-to-end integration test demonstrating character race building with `formwork`.
//!
//! Verifies builder ergonomics, mixing dry traits and wet choice traits,
//! and serialization of complex D&D 5e racial trait hierarchies.

use formwork::choice::SingleChoice;
use stress_test::ability::{Ability, AbilityScoreIncrease};
use stress_test::contentref::{RaceRef, SourceRef, ToolRef};
use stress_test::effect::{Effect, EffectId, EffectFormwork, SheetOperation};
use stress_test::proficiency::Proficiency;
use stress_test::race::RaceFormwork;
use stress_test::traits::{Trait, TraitFormwork};

fn source_ref() -> SourceRef {
    SourceRef::new("srd").unwrap()
}

/// Verifies that [`RaceFormwork::builder`] can assemble a racial formwork containing both
/// fixed dry traits and wet choice traits via cascading `.with_traits(...)`.
#[test]
fn main() {
    let race = RaceFormwork::builder()
        .id(RaceRef::new(source_ref(), "dwarf").unwrap())
        .name("Dwarf")
        .description("Dwarf test")
        .with_traits(ability_score_increase())
        .with_traits(tool_proficiency())
        .build();
    let json = serde_json::to_string_pretty(&race).unwrap();
    println!("{}", json);
    let _race: RaceFormwork = serde_json::from_str(&json).unwrap();
}

/// Constructs a fully dry [`Trait`] (Dwarf Ability Score Increase: +2 Constitution)
/// using the generated [`Trait::builder`] and [`Effect::builder`].
fn ability_score_increase() -> Trait {
    let effect = Effect::builder()
        .id(EffectId::default())
        .operation(SheetOperation::AbilityScoreIncrease(
            AbilityScoreIncrease::Racial {
                increase: 2,
                ability: Ability::Constitution,
            },
        ))
        .build();
    Trait::builder()
        .title("Ability score increase".to_string())
        .description("Ability score increase".to_string())
        .with_effects(effect)
        .build()
}

/// Constructs a wet [`TraitFormwork`] (Dwarf Tool Proficiency) containing a
/// [`SingleChoice`] for picking one tool proficiency from three options.
fn tool_proficiency() -> TraitFormwork {
    let effect = EffectFormwork::builder()
        .id(EffectId::default())
        .operation(SingleChoice::new(
            "Pick one tool to be proficient with",
            vec![
                SheetOperation::Proficiency(Proficiency::Tool(
                    ToolRef::new(source_ref(), "Brewer's Supplies").unwrap(),
                )),
                SheetOperation::Proficiency(Proficiency::Tool(
                    ToolRef::new(source_ref(), "Mason's Tools").unwrap(),
                )),
                SheetOperation::Proficiency(Proficiency::Tool(
                    ToolRef::new(source_ref(), "Something Else").unwrap(),
                )),
            ],
        ))
        .build();
    TraitFormwork::builder()
        .title("Tool Proficiency".to_string())
        .description("Tool Proficiency".to_string())
        .with_effects(effect)
        .build()
}
