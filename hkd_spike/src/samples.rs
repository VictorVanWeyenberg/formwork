//! Shapes the Dwarf does not exercise: a choice between *complete* traits (Fighting Style), a
//! *nested* choice (Ability Score Improvement: "+2 to one, or +1 to two"), and a struct with two
//! slot fields for the derive.

use crate::blueprint::{Blueprint, Cardinality, Choice, ChoiceOption, Draft, Mode};
use crate::effect::*;
use crate::race::Race;
use crate::traits::Trait;
use serde::{Deserialize, Serialize};

fn style(title: &str, description: &str) -> Trait {
    Trait::builder().title(title).description(description).build()
}

/// Each option is a finished `Trait`, lifted into the draft by `ChoiceOption::single`'s `Into`.
pub fn fighting_style() -> Choice<Trait<Draft>> {
    Choice::one(
        "Choose a fighting style.",
        [
            ChoiceOption::single("Archery", style("Archery", "+2 to attack rolls with ranged weapons.")),
            ChoiceOption::single("Defense", style("Defense", "+1 to AC while wearing armor.")),
            ChoiceOption::single("Dueling", style("Dueling", "+2 damage with a one-handed melee weapon.")),
        ],
    )
}

pub fn asi(increase: isize, ability: Ability) -> Effect {
    Effect::new(SheetOperation::AbilityScoreIncrease(AbilityScoreIncrease::Racial { increase, ability }))
}

fn asi_options(increase: isize) -> impl Iterator<Item = ChoiceOption<Effect>> {
    Ability::ALL
        .into_iter()
        .map(move |ability| ChoiceOption::single(format!("{ability:?}"), asi(increase, ability)))
}

/// A choice whose options themselves contain choices.
pub fn ability_score_improvement() -> Choice<Trait<Draft>> {
    let plus_two = Trait::<Draft>::builder()
        .title("Ability Score Improvement")
        .description("Increase one ability score by 2.")
        .with_effect(Choice::one("Choose the ability to increase by 2.", asi_options(2)))
        .build();
    let plus_one_twice = Trait::<Draft>::builder()
        .title("Ability Score Improvement")
        .description("Increase two ability scores by 1.")
        .with_effect(Choice::new(
            "Choose two abilities to increase by 1.",
            Cardinality::exactly(2),
            asi_options(1),
        ))
        .build();
    Choice::one(
        "Increase one score by 2, or two scores by 1?",
        [
            ChoiceOption::single("+2 to one", plus_two),
            ChoiceOption::single("+1 to two", plus_one_twice),
        ],
    )
}

/// Not a race, but the spike has no `Class`: the two class-level shapes hung on a `Race<Draft>`.
pub fn fighter() -> Race<Draft> {
    Race::builder()
        .id("spike/race/fighter")
        .name("Fighter")
        .description("Level 1 fighting style and level 4 ASI, as traits.")
        .with_trait(fighting_style())
        .with_trait(ability_score_improvement())
        .build()
}

/// Two slot fields of different kinds; the derive must visit them in declaration order.
/// Shorthand `M::Kind` spelling (no builder here), so both derive detection paths are covered,
/// and deliberately *without* `#[serde(bound = "")]`: with the shorthand, serde infers no bound on `M`.
#[derive(Clone, PartialEq, Serialize, Deserialize, Blueprint)]
pub struct Both<M: Mode> {
    pub label: String,
    pub effects: Vec<M::Effect>,
    pub traits: Vec<M::Trait>,
}
