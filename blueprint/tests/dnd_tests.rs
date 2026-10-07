use blueprint::choice::Choice;
use blueprint::dnd_model::*;
use blueprint::visitor::{
    AutoSelectFirst, ChoiceVisitor,
    VisitorFlow, UnmakeAll,
};
use blueprint::Blueprint;

struct ToolSelector {
    chosen_index: usize,
    recorded_prompt: Option<String>,
    recorded_options: Vec<String>,
}

impl ChoiceVisitor<Effect> for ToolSelector {
    fn visit_choice(&mut self, choice: &mut Choice<Effect>) -> VisitorFlow {
        self.recorded_prompt = Some(choice.prompt().to_string());
        self.recorded_options = choice.options().iter().map(|o| format!("{:?}", o)).collect();
        choice.select(self.chosen_index).unwrap();
        VisitorFlow::Break
    }
}

impl ChoiceVisitor<Trait<TraitDraft>> for ToolSelector {
    fn visit_choice(&mut self, _choice: &mut Choice<Trait<TraitDraft>>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<String> for ToolSelector {
    fn visit_choice(
        &mut self,
        _choice: &mut Choice<String>,
    ) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

struct PromptCollector(Vec<String>);

impl ChoiceVisitor<Effect> for PromptCollector {
    fn visit_choice(&mut self, choice: &mut Choice<Effect>) -> VisitorFlow {
        self.0.push(choice.prompt().to_string());
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<Trait<TraitDraft>> for PromptCollector {
    fn visit_choice(&mut self, choice: &mut Choice<Trait<TraitDraft>>) -> VisitorFlow {
        self.0.push(choice.prompt().to_string());
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<String> for PromptCollector {
    fn visit_choice(
        &mut self,
        choice: &mut Choice<String>,
    ) -> VisitorFlow {
        self.0.push(choice.prompt().to_string());
        VisitorFlow::Continue
    }
}

#[test]
fn test_dwarf_blueprint_wizard_flow_and_resolution() {
    let mut dwarf = dwarf_blueprint();

    // 1. Initial state: not complete because Tool Proficiency is pending
    assert!(!dwarf.is_complete());

    // Verify exactly 1 pending choice exists
    let mut collector = PromptCollector(vec![]);
    dwarf.walk_choices(&mut collector);
    assert_eq!(collector.0, vec!["Choose one tool proficiency:"]);

    // 2. Inspect available options & select option 1: "Brewer's supplies"
    let mut selector = ToolSelector {
        chosen_index: 1,
        recorded_prompt: None,
        recorded_options: vec![],
    };
    dwarf.walk_choices(&mut selector);

    assert_eq!(selector.recorded_prompt.as_deref(), Some("Choose one tool proficiency:"));
    assert_eq!(
        selector.recorded_options,
        vec![
            r#"Proficiency { item: "Smith's tools" }"#,
            r#"Proficiency { item: "Brewer's supplies" }"#,
            r#"Proficiency { item: "Mason's tools" }"#
        ]
    );

    // 3. Now draft should be complete
    assert!(dwarf.is_complete());

    // 4. Resolve into watertight final Race
    let race = dwarf.resolve().expect("dwarf resolves successfully");
    assert_eq!(race.id, "srd:race:dwarf");
    assert_eq!(race.name, "Dwarf");
    assert_eq!(race.traits.len(), 5);

    let trait_titles: Vec<&str> = race.traits.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(
        trait_titles,
        vec![
            "Ability Score Increase",
            "Darkvision",
            "Dwarven Resilience",
            "Dwarven Combat Training",
            "Tool Proficiency",
        ]
    );

    // Verify chosen tool proficiency effect
    let tool_trait = &race.traits[4];
    assert_eq!(tool_trait.title, "Tool Proficiency");
    assert_eq!(
        tool_trait.effects,
        vec![Effect::Proficiency {
            item: "Brewer's supplies".to_string()
        }]
    );
}

#[test]
fn test_dwarf_unmake_choice_and_reselect() {
    let mut dwarf = dwarf_blueprint();

    // Select Mason's tools (index 2)
    let mut selector = ToolSelector {
        chosen_index: 2,
        recorded_prompt: None,
        recorded_options: vec![],
    };
    dwarf.walk_choices(&mut selector);
    assert!(dwarf.is_complete());

    // Unmake the choice via walk_choices(&mut UnmakeAll) == VisitorFlow::Continue
    let unmade = dwarf.walk_choices(&mut UnmakeAll) == VisitorFlow::Continue;
    assert!(unmade);
    assert!(!dwarf.is_complete());

    // Check that choice is pending again, but history remembers [2]!
    struct HistoryInspector {
        selected_index: Option<usize>,
        is_made: bool,
    }
    impl ChoiceVisitor<Effect> for HistoryInspector {
        fn visit_choice(&mut self, choice: &mut Choice<Effect>) -> VisitorFlow {
            self.selected_index = choice.selected_index();
            self.is_made = choice.is_made();
            // Change selection to Smith's tools (index 0)
            choice.select(0).unwrap();
            VisitorFlow::Break
        }
    }
    impl ChoiceVisitor<Trait<TraitDraft>> for HistoryInspector {
        fn visit_choice(&mut self, _c: &mut Choice<Trait<TraitDraft>>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }
    impl ChoiceVisitor<String> for HistoryInspector {
        fn visit_choice(&mut self, _choice: &mut Choice<String>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    let mut inspector = HistoryInspector { selected_index: None, is_made: true };
    dwarf.walk_choices(&mut inspector);

    assert!(!inspector.is_made);
    assert_eq!(inspector.selected_index, Some(2));

    assert!(dwarf.is_complete());
    let race = dwarf.resolve().unwrap();
    let tool_trait = &race.traits[4];
    assert_eq!(
        tool_trait.effects,
        vec![Effect::Proficiency {
            item: "Smith's tools".to_string()
        }]
    );
}

#[test]
fn test_dwarf_serde_roundtrip() {
    let dwarf = dwarf_blueprint();

    // Draft JSON round-trip
    let json_draft = serde_json::to_string_pretty(&dwarf).unwrap();
    assert!(json_draft.contains("Choose one tool proficiency:"));

    let deserialized_draft: RaceBlueprint = serde_json::from_str(&json_draft).unwrap();
    assert_eq!(deserialized_draft, dwarf);

    // Resolve and Final JSON round-trip
    let mut resolved_dwarf = deserialized_draft;
    resolved_dwarf.walk_choices(&mut AutoSelectFirst);
    let race = resolved_dwarf.resolve().unwrap();

    let json_final = serde_json::to_string_pretty(&race).unwrap();
    // Final mode cannot contain "prompt" or "choose"!
    assert!(!json_final.contains("Choose one tool proficiency:"));
    assert!(!json_final.contains("\"choose\""));

    let deserialized_race: Race<RaceFinal> = serde_json::from_str(&json_final).unwrap();
    assert_eq!(deserialized_race, race);
}

#[test]
fn test_dwarf_lifting_and_watertightness() {
    let mut dwarf = dwarf_blueprint();
    let mut selector = ToolSelector {
        chosen_index: 1,
        recorded_prompt: None,
        recorded_options: vec![],
    };
    dwarf.walk_choices(&mut selector);
    let race: Race<RaceFinal> = dwarf.resolve().unwrap();

    // Lifting Final to Draft
    let lifted: RaceBlueprint = race.clone().into();
    assert!(lifted.is_complete());
    assert_eq!(lifted.resolve().unwrap(), race);
}

#[test]
fn test_typed_builder_dnd() {
    let t = Trait::<TraitFinal>::builder()
        .title("Fleet of Foot".to_string())
        .description("Your base walking speed increases to 35 feet.".to_string())
        .with_effect(Effect::Proficiency { item: "Speed".into() })
        .build();

    let race = Race::<RaceFinal>::builder()
        .id("srd:race:wood-elf".to_string())
        .name("Wood Elf".to_string())
        .description("Fast and stealthy.".to_string())
        .with_trait(t)
        .bonus_skills(vec!["Perception".into()])
        .build();

    assert_eq!(race.name, "Wood Elf");
    assert_eq!(race.traits[0].title, "Fleet of Foot");
}

#[test]
fn test_typed_builder_dnd_draft_with_choices() {
    use blueprint::choice::{Choice};

    let trait_choice = Choice::new(
        "Choose an elf trait:",
        vec![
            Trait::<TraitDraft>::builder()
                .title("Keen Senses")
                .description("Proficiency in Perception.")
                .with_effect(Effect::proficiency("Perception"))
                .build(),
        ],
    );

    let race_blueprint = RaceBlueprint::builder()
        .id("custom-elf")
        .name("Custom Elf")
        .description("A custom elf.")
        .with_trait(
            Trait::<TraitDraft>::builder()
                .title("Darkvision")
                .description("See in the dark.")
                .with_effect(Effect::darkvision(60))
                .build(),
        )
        .with_trait(trait_choice)
        .build();

    assert_eq!(race_blueprint.traits.len(), 2);
    assert!(race_blueprint.traits[0].is_fixed());
    assert!(race_blueprint.traits[1].is_pending());
}
