//! Integration tests: use the public API only, the way `fifth_srd` and `fifth_engine` would.

use hkd_spike::blueprint::*;
use hkd_spike::effect::*;
use hkd_spike::race::{Race, RaceBlueprint};
use hkd_spike::samples::{self, asi, Both, BothBlueprint};
use hkd_spike::srd;
use hkd_spike::traits::{Trait, TraitBlueprint};
use serde_json::{json, Value};

fn json<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap()
}

fn tool(name: &str) -> Effect {
    srd::tool_proficiency_effect(name)
}

// ---------------------------------------------------------------------------------------------
// S0 item 1 — derives on `struct X<M: Mode> { .. Vec<M::Kind> .. }` (compiles ⇒ verified)
// ---------------------------------------------------------------------------------------------

#[test]
fn a_final_trait_serializes_exactly_like_a_plain_struct() {
    assert_eq!(
        json(&srd::darkvision_trait()),
        json!({
            "title": "Darkvision",
            "description": "You can see in dim light within 60 feet of you as if it were bright light.",
            "effects": [{ "vision": "Darkvision" }]
        })
    );
}

#[test]
fn fixed_slots_are_transparent_in_json() {
    let final_trait = srd::darkvision_trait();
    let draft: TraitBlueprint = final_trait.clone().into();
    assert_eq!(json(&draft), json(&final_trait));

    let text = serde_json::to_string(&draft).unwrap();
    let back: TraitBlueprint = serde_json::from_str(&text).unwrap();
    assert!(back == draft);
}

#[test]
fn a_draft_race_round_trips() {
    let dwarf = srd::dwarf();
    let text = serde_json::to_string(&dwarf).unwrap();
    let back: RaceBlueprint = serde_json::from_str(&text).unwrap();
    assert!(back == dwarf);
    println!("dwarf (draft) compact: {} bytes, pretty: {} bytes", text.len(), serde_json::to_string_pretty(&dwarf).unwrap().len());
    println!("{}", serde_json::to_string_pretty(&dwarf).unwrap());
}

// ---------------------------------------------------------------------------------------------
// S0 item 3 — `From` coherence + inference through `impl Into<M::Kind>` (compiles ⇒ verified)
// ---------------------------------------------------------------------------------------------

#[test]
fn every_conversion_the_builders_rely_on_exists() {
    let effect = tool("srd/tool/smith_tools");
    let _: SlotOne<Effect> = effect.clone().into();
    let _: SlotOne<Effect> = Choice::one("p", [ChoiceOption::new("a", effect)]).into();
    let _: TraitBlueprint = srd::darkvision_trait().into();
    // let _: SlotOne<TraitBlueprint> = srd::darkvision_trait().into();
    // let _: SlotOne<TraitBlueprint> = srd::tool_proficiency().into();
    // let _: SlotOne<TraitBlueprint> = samples::fighting_style().into();
}

// ---------------------------------------------------------------------------------------------
// S0 item 4 — `#[serde(flatten)]` over an externally tagged enum (ticket T6)
// ---------------------------------------------------------------------------------------------

#[test]
fn effect_json_is_externally_tagged_and_flat() {
    let skill = |expertise| Proficiency::Skill { skill: Skill::Perception, expertise };
    let cases: Vec<(Effect, Value)> = vec![
        (tool("srd/tool/smith_tools"), json!({"proficiency": {"tool": "srd/tool/smith_tools"}})),
        (Effect::new(SheetOperation::Proficiency(skill(false))), json!({"proficiency": {"skill": {"skill": "Perception"}}})),
        (Effect::new(SheetOperation::Proficiency(skill(true))), json!({"proficiency": {"skill": {"skill": "Perception", "expertise": true}}})),
        (Effect::new(SheetOperation::Proficiency(Proficiency::SavingThrow(Ability::Constitution))), json!({"proficiency": {"savingThrow": "Constitution"}})),
        (Effect::new(SheetOperation::Vision(Vision::Darkvision)), json!({"vision": "Darkvision"})),
        (Effect::new(SheetOperation::RaceWalkingSpeed(Speed(25))), json!({"raceWalkingSpeed": 25})),
        (Effect::new(SheetOperation::RaceSize(Size::Medium)), json!({"raceSize": "Medium"})),
        (Effect::new(SheetOperation::IgnoreRule(Rule::HeavyArmorSpeedPenalty)), json!({"ignoreRule": "HeavyArmorSpeedPenalty"})),
        (Effect::new(SheetOperation::DamageModulation(DamageModulation::Resistance(DamageType::Poison))), json!({"damageModulation": {"resistance": "Poison"}})),
        (asi(2, Ability::Constitution), json!({"abilityScoreIncrease": {"Racial": {"increase": 2, "ability": "Constitution"}}})),
        (Effect::new(SheetOperation::ModifyMaxHitPoint(ValueRef::Sheet(SheetRef::Level))), json!({"modifyMaxHitPoint": "Level"})),
        (Effect::new(SheetOperation::ModifyMaxHitPoint(ValueRef::Const(1))), json!({"modifyMaxHitPoint": 1})),
        (Effect::with_id("rage", SheetOperation::Vision(Vision::Darkvision)), json!({"id": "rage", "vision": "Darkvision"})),
    ];
    for (effect, expected) in cases {
        assert_eq!(json(&effect), expected);
        let text = serde_json::to_string(&expected).unwrap();
        let back: Effect = serde_json::from_str(&text).unwrap();
        assert_eq!(back, effect);
    }
}

#[test]
fn a_flattened_effect_survives_the_untagged_slot_and_the_options_map() {
    let fixed: SlotOne<Effect> = serde_json::from_str(r#"{"vision":"Darkvision"}"#).unwrap();
    assert_eq!(fixed.fixed(), Some(&Effect::new(SheetOperation::Vision(Vision::Darkvision))));

    let choose: SlotOne<Effect> = serde_json::from_str(
        r#"{"choose":1,"prompt":"Pick.","options":{"Darkvision":{"vision":"Darkvision"},"Poison":{"damageModulation":{"resistance":"Poison"}}}}"#,
    )
    .unwrap();
    let choice = choose.choice().expect("a choice");
    assert_eq!(choice.labels(), ["Darkvision", "Poison"]);
    assert_eq!(choice.options()[1].values(), &Effect::new(SheetOperation::DamageModulation(DamageModulation::Resistance(DamageType::Poison))));
}

// ---------------------------------------------------------------------------------------------
// S0 item 5 — label-keyed options map (ticket T5)
// ---------------------------------------------------------------------------------------------

#[test]
fn options_keep_document_order() {
    let choice: Choice<Effect> = serde_json::from_str(
        r#"{"choose":1,"prompt":"p","options":{"c":{"vision":"Darkvision"},"a":{"vision":"Darkvision"},"b":{"vision":"Darkvision"}}}"#,
    )
    .unwrap();
    assert_eq!(choice.labels(), ["c", "a", "b"]);
    let text = serde_json::to_string(&choice).unwrap();
    let position = |label: &str| text.find(&format!("\"{label}\":")).unwrap();
    assert!(position("c") < position("a") && position("a") < position("b"));
}

#[test]
fn duplicate_labels_are_rejected_on_load() {
    let error = serde_json::from_str::<Choice<Effect>>(
        r#"{"choose":1,"prompt":"p","options":{"a":{"vision":"Darkvision"},"a":{"vision":"Darkvision"}}}"#,
    )
    .err()
    .expect("duplicate label must fail");
    assert!(error.to_string().contains("duplicate option label `a`"), "{error}");
}

#[test]
fn single_and_many_values_round_trip() {
    let resistance = |t| Effect::new(SheetOperation::DamageModulation(DamageModulation::Resistance(t)));
    let choice = Choice::one(
        "Draconic ancestry",
        [
            ChoiceOption::new("Black", resistance(DamageType::Acid)),
            ChoiceOption::new("Red", resistance(DamageType::Fire)),
        ],
    );
    let text = serde_json::to_string(&choice).unwrap();
    assert_eq!(
        text,
        r#"{"choose":1,"prompt":"Draconic ancestry","options":{"Black":{"damageModulation":{"resistance":"Acid"}},"Red":[{"damageModulation":{"resistance":"Fire"}},{"vision":"Darkvision"}],"None":[]}}"#
    );
    let back: Choice<Effect> = serde_json::from_str(&text).unwrap();
    assert_eq!(back, choice);
}

#[test]
fn cardinality_json_and_semantics() {
    assert_eq!(json(&Cardinality::ONE), json!(1));
    assert_eq!(json(&Cardinality::exactly(2)), json!(2));
    assert_eq!(json(&Cardinality::up_to(2)), json!({"upTo": 2}));
    assert_eq!(json(&Cardinality::at_least(1)), json!({"atLeast": 1}));
    for c in [Cardinality::ONE, Cardinality::exactly(2), Cardinality::up_to(2), Cardinality::at_least(1)] {
        let back: Cardinality = serde_json::from_value(json(&c)).unwrap();
        assert_eq!(back, c);
    }
    assert!(!Cardinality::exactly(2).accepts(1) && Cardinality::exactly(2).accepts(2) && !Cardinality::exactly(2).accepts(3));
    assert!(Cardinality::up_to(2).accepts(0) && Cardinality::up_to(2).accepts(2) && !Cardinality::up_to(2).accepts(3));
    assert!(!Cardinality::at_least(2).accepts(1) && Cardinality::at_least(2).accepts(2) && Cardinality::at_least(2).accepts(3));
}

#[test]
fn selection_state_is_absent_in_a_book_and_present_on_a_character() {
    let mut dwarf = srd::dwarf();
    let book = serde_json::to_string(&dwarf).unwrap();
    assert!(!book.contains("\"selected\""));

    dwarf.next_open_choice_mut().unwrap().select(1);
    let character = serde_json::to_string(&dwarf).unwrap();
    assert!(character.contains(r#""selected":["Brewer's supplies"]"#), "{character}");
    let back: RaceBlueprint = serde_json::from_str(&character).unwrap();
    assert!(back == dwarf);
}

// ---------------------------------------------------------------------------------------------
// Behaviour — resolve, cardinality, nesting, lift (tickets T2/T3)
// ---------------------------------------------------------------------------------------------

#[test]
fn an_unresolved_draft_fails_naming_the_open_prompt() {
    let error = srd::dwarf().resolve().err().expect("tool proficiency is still open");
    assert_eq!(
        error,
        ChoiceError::BadCardinality {
            prompt: "Choose one tool to be proficient with.".into(),
            expected: Cardinality::ONE,
            got: 0,
        }
    );
}

#[test]
fn the_wizard_walk_resolves_the_dwarf() {
    let mut draft = srd::dwarf();
    assert!(!draft.is_complete());
    assert_eq!(draft.visible_choices().len(), 1);

    let mut prompts = vec![];
    while let Some(choice) = draft.next_open_choice_mut() {
        prompts.push(choice.prompt().to_string());
        choice.select(0);
    }
    assert_eq!(prompts, ["Choose one tool to be proficient with."]);
    assert!(draft.is_complete());

    let race: Race = draft.resolve().ok().expect("all choices made");
    let titles: Vec<&str> = race.traits().iter().map(Trait::title).collect();
    assert_eq!(
        titles,
        ["Ability Score Increase", "Size", "Speed", "Darkvision", "Dwarven Resilience", "Dwarven Combat Training", "Tool Proficiency"]
    );
    assert_eq!(race.traits()[6].effects(), [tool("srd/tool/smith_tools")]);
    assert!(!serde_json::to_string(&race).unwrap().contains("choose"));
}

#[test]
fn over_selection_is_rejected() {
    let mut draft = srd::dwarf();
    {
        let choice = draft.next_open_choice_mut().unwrap();
        choice.select(0);
        choice.select(1);
        assert!(!choice.is_ok());
        assert!(choice.is_selected(0) && choice.is_selected(1) && !choice.is_selected(2));
        choice.unselect(0);
        assert!(choice.is_ok());
        choice.select(0);
    }
    let error = draft.resolve().err().expect("two picks for a choose-one");
    assert!(matches!(error, ChoiceError::BadCardinality { got: 2, .. }));
}

#[test]
fn nested_choices_are_disclosed_progressively() {
    let mut fighter = samples::fighter();
    assert_eq!(fighter.visible_choices().len(), 2, "inner ASI choices hidden until a branch is picked");

    let mut prompts = vec![];
    while let Some(choice) = fighter.next_open_choice_mut() {
        prompts.push(choice.prompt().to_string());
        choice.select(0);
    }
    assert_eq!(
        prompts,
        ["Choose a fighting style.", "Increase one score by 2, or two scores by 1?", "Choose the ability to increase by 2."]
    );
    assert_eq!(fighter.visible_choices().len(), 3);

    let race = fighter.resolve().ok().expect("complete");
    let titles: Vec<&str> = race.traits().iter().map(Trait::title).collect();
    assert_eq!(titles, ["Archery", "Ability Score Improvement"]);
    assert_eq!(race.traits()[1].effects(), [asi(2, Ability::Strength)]);
}

#[test]
fn the_other_branch_reveals_a_two_pick_choice() {
    let mut fighter = samples::fighter();
    fighter.next_open_choice_mut().unwrap().select(1); // Defense
    fighter.next_open_choice_mut().unwrap().select(1); // "+1 to two"
    {
        let inner = fighter.next_open_choice_mut().unwrap();
        assert_eq!(inner.prompt(), "Choose two abilities to increase by 1.");
        assert_eq!(*inner.cardinality(), Cardinality::exactly(2));
        inner.select(1); // Dexterity
        assert!(!inner.is_ok());
        inner.select(2); // Constitution
        assert!(inner.is_ok());
    }
    let race = fighter.resolve().ok().expect("complete");
    assert_eq!(race.traits()[0].title(), "Defense");
    assert_eq!(race.traits()[1].effects(), [asi(1, Ability::Dexterity), asi(1, Ability::Constitution)]);
}

#[test]
fn lift_then_resolve_is_the_identity() {
    let mut draft = srd::dwarf();
    draft.next_open_choice_mut().unwrap().select(2);
    let race: Race = draft.resolve().ok().unwrap();
    let lifted: RaceBlueprint = race.clone().into();
    assert!(lifted.is_complete());
    assert!(lifted.resolve().ok().unwrap() == race);
}

// ---------------------------------------------------------------------------------------------
// S0 item 6 — `#[derive(Blueprint)]` on a struct with two slot fields (ticket T7)
// ---------------------------------------------------------------------------------------------

#[test]
fn the_derive_visits_slot_fields_in_declaration_order() {
    let mut both: BothBlueprint = Both {
        label: "both".into(),
        effects: vec![Choice::one("effect first", [ChoiceOption::new("a", tool("a"))]).into()],
        traits: vec![samples::fighting_style().into(), srd::tool_proficiency().into()],
    };
    let prompts: Vec<String> = both.visible_choices().iter().map(|c| c.prompt().to_string()).collect();
    assert_eq!(prompts, ["effect first", "Choose a fighting style.", "Choose one tool to be proficient with."]);

    while let Some(choice) = both.next_open_choice_mut() {
        choice.select(0);
    }
    let resolved: Both = both.resolve().ok().unwrap();
    assert_eq!(resolved.effects, [tool("a")]);
    assert_eq!(resolved.traits.iter().map(Trait::title).collect::<Vec<_>>(), ["Archery", "Tool Proficiency"]);

    let lifted: BothBlueprint = resolved.clone().into();
    assert!(lifted.resolve().ok().unwrap() == resolved);
}
