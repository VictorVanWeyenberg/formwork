use super::effect::Effect;
use super::race::{Race, RaceBlueprint, RaceDraft};
use super::trait_def::{Trait, TraitDraft};
use crate::choice::{Choice, ChoiceOption};
use crate::slot::{Slot};

/// Constructs the D&D 5e Dwarf race blueprint.
/// All racial traits are fixed except for Tool Proficiency, which requires a choice.
pub fn dwarf_blueprint() -> RaceBlueprint {
    let asi = Trait::<TraitDraft>::draft(
        "Ability Score Increase",
        "Your Constitution score increases by 2.",
        vec![Slot::fixed(Effect::ability_bonus("Constitution", 2))],
    );

    let darkvision = Trait::<TraitDraft>::draft(
        "Darkvision",
        "You can see in dim light within 60 feet of you as if it were bright light.",
        vec![Slot::fixed(Effect::darkvision(60))],
    );

    let dwarven_resilience = Trait::<TraitDraft>::draft(
        "Dwarven Resilience",
        "You have advantage on saving throws against poison, and you have resistance against poison damage.",
        vec![Slot::fixed(Effect::resistance("Poison"))],
    );

    let dwarven_combat_training = Trait::<TraitDraft>::draft(
        "Dwarven Combat Training",
        "You have proficiency with the battleaxe, handaxe, light hammer, and warhammer.",
        vec![
            Slot::fixed(Effect::proficiency("Battleaxe")),
            Slot::fixed(Effect::proficiency("Handaxe")),
            Slot::fixed(Effect::proficiency("Light hammer")),
            Slot::fixed(Effect::proficiency("Warhammer")),
        ],
    );

    // Tool Proficiency: choice of 1 from 3 tools (strictly scalar, no cardinality)
    let tool_choice = Choice::new(
        "Choose one tool proficiency:",
        vec![
            ChoiceOption::new("Smith's tools", Effect::proficiency("Smith's tools")),
            ChoiceOption::new("Brewer's supplies", Effect::proficiency("Brewer's supplies")),
            ChoiceOption::new("Mason's tools", Effect::proficiency("Mason's tools")),
        ],
    );

    let tool_proficiency = Trait::<TraitDraft>::draft(
        "Tool Proficiency",
        "You gain proficiency with the artisan's tools of your choice: smith's tools, brewer's supplies, or mason's tools.",
        vec![Slot::pending(tool_choice)],
    );

    Race::<RaceDraft>::draft(
        "srd:race:dwarf",
        "Dwarf",
        "Bold and hardy, dwarves are known as skilled warriors, miners, and workers of stone and metal.",
        vec![
            Slot::fixed(asi),
            Slot::fixed(darkvision),
            Slot::fixed(dwarven_resilience),
            Slot::fixed(dwarven_combat_training),
            Slot::fixed(tool_proficiency),
        ],
        vec![],
    )
}
