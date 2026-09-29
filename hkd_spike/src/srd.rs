//! The Dwarf, written the way a book author (`fifth_srd`) would write it. Helpers keep returning
//! plain `Trait`; only the one trait with a choice is a `Trait<Draft>`.

use crate::blueprint::{Choice, ChoiceOption};
use crate::effect::*;
use crate::race::{Race, RaceBlueprint};
use crate::traits::{Trait, TraitBlueprint, TraitDraft};

pub fn ability_score_increase_trait(increase: isize, ability: Ability) -> Trait {
    Trait::builder()
        .title("Ability Score Increase")
        .description(format!("Your {ability:?} score increases by {increase}."))
        .with_effect(Effect::new(SheetOperation::AbilityScoreIncrease(AbilityScoreIncrease::Racial {
            increase,
            ability,
        })))
        .build()
}

pub fn size_trait(size: Size) -> Trait {
    Trait::builder()
        .title("Size")
        .description(format!("Your size is {size:?}."))
        .with_effect(Effect::new(SheetOperation::RaceSize(size)))
        .build()
}

pub fn darkvision_trait() -> Trait {
    Trait::builder()
        .title("Darkvision")
        .description("You can see in dim light within 60 feet of you as if it were bright light.")
        .with_effect(Effect::new(SheetOperation::Vision(Vision::Darkvision)))
        .build()
}

fn speed() -> Trait {
    Trait::builder()
        .title("Speed")
        .description("Your base walking speed is 25 feet. Your speed is not reduced by wearing heavy armor.")
        .with_effect(Effect::new(SheetOperation::RaceWalkingSpeed(Speed(25))))
        .with_effect(Effect::new(SheetOperation::IgnoreRule(Rule::HeavyArmorSpeedPenalty)))
        .build()
}

fn dwarven_resilience() -> Trait {
    Trait::builder()
        .title("Dwarven Resilience")
        .description("You have resistance against poison damage.")
        .with_effect(Effect::new(SheetOperation::DamageModulation(DamageModulation::Resistance(
            DamageType::Poison,
        ))))
        .build()
}

fn dwarven_combat_training() -> Trait {
    let mut trait_ = Trait::builder()
        .title("Dwarven Combat Training")
        .description("You have proficiency with the battleaxe, handaxe, light hammer, and warhammer.")
        .build();
    for weapon in ["battleaxe", "handaxe", "light_hammer", "warhammer"] {
        trait_.add_effect(Effect::new(SheetOperation::Proficiency(Proficiency::Weapon(format!(
            "srd/weapon/{weapon}"
        )))));
    }
    trait_
}

pub const ARTISAN_TOOLS: [(&str, &str); 3] = [
    ("Smith's tools", "srd/tool/smith_tools"),
    ("Brewer's supplies", "srd/tool/brewer_supplies"),
    ("Mason's tools", "srd/tool/mason_tools"),
];

pub fn tool_proficiency_effect(tool: &str) -> Effect {
    Effect::new(SheetOperation::Proficiency(Proficiency::Tool(tool.to_string())))
}

/// The only Dwarf trait that needs a choice: a `Choice<Effect>` sits where the effect will be.
/// `Trait::builder()` infers `M = Draft` from the return type (no turbofish needed).
pub fn tool_proficiency() -> TraitBlueprint {
    Trait::builder()
        .title("Tool Proficiency")
        .description("You gain proficiency with the artisan's tools of your choice: smith's tools, brewer's supplies, or mason's tools.")
        .with_effect(Choice::one(
            "Choose one tool to be proficient with.",
            ARTISAN_TOOLS.map(|(label, tool)| ChoiceOption::new(label, tool_proficiency_effect(tool))),
        ))
        .build()
}

pub fn dwarf() -> RaceBlueprint {
    RaceBlueprint::builder()
        .id("srd/race/dwarf")
        .name("Dwarf")
        .description("Digs holes and likes to drink.")
        .with_trait(ability_score_increase_trait(2, Ability::Constitution)) // Trait, lifted
        .with_trait(size_trait(Size::Medium))
        .with_trait(speed())
        .with_trait(darkvision_trait())
        .with_trait(dwarven_resilience())
        .with_trait(dwarven_combat_training())
        .with_trait(tool_proficiency()) // Trait<Draft>
        .build()
}
