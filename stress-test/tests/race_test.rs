use schematic::{choice::SingleChoice};
use stress_test::ability::{Ability, AbilityScoreIncrease};
use stress_test::contentref::{RaceRef, SourceRef, ToolRef};
use stress_test::effect::{EffectId, EffectSchematic, SheetOperation, Effect};
use stress_test::proficiency::Proficiency;
use stress_test::race::RaceSchematic;
use stress_test::traits::{TraitSchematic, Trait};

fn source_ref() -> SourceRef {
    SourceRef::new("srd").unwrap()
}

#[test]
fn main() {
    let race = RaceSchematic::builder()
        .id(RaceRef::new(source_ref(), "dwarf").unwrap())
        .name("Dwarf")
        .description("Dwarf test")
        .with_traits(ability_score_increase())
        .with_traits(tool_proficiency())
        .build();
    println!("{}", serde_json::to_string_pretty(&race).unwrap());
}

fn ability_score_increase() -> Trait {
    let effect = Effect::builder()
        .id(EffectId::default())
        .operation(SheetOperation::AbilityScoreIncrease(
            AbilityScoreIncrease::Racial {
                increase: 2,
                ability: Ability::Constitution,
            }
        ))
        .build();
    Trait::builder()
        .title("Ability score increase".to_string())
        .description("Ability score increase".to_string())
        .with_effects(effect)
        .build()
}

fn tool_proficiency() -> TraitSchematic {
    let effect = EffectSchematic::builder()
        .id(EffectId::default())
        .operation(SingleChoice::new(
            "Pick one tool to be proficient with",
            vec![
                SheetOperation::Proficiency(Proficiency::Tool(ToolRef::new(source_ref(), "Brewer's Supplies").unwrap())),
                SheetOperation::Proficiency(Proficiency::Tool(ToolRef::new(source_ref(), "Mason's Tools").unwrap())),
                SheetOperation::Proficiency(Proficiency::Tool(ToolRef::new(source_ref(), "Something Else").unwrap())),
            ]
        ))
        .build();
    TraitSchematic::builder()
        .title("Tool Proficiency".to_string())
        .description("Tool Proficiency".to_string())
        .with_effects(effect)
        .build()
}