//! Comprehensive integration test suite verifying all 10 tutorial chapters end-to-end.

use formwork::choice::{Cardinality, MultiChoice, SingleChoice};
use formwork::tutorial::*;
use formwork::visitor::{ChoiceVisitor, VisitorFlow};
use formwork::{Formwork, FormworkError, StepResult};
use std::fs;
use std::path::Path;

#[test]
fn test_run_all_tutorials_suite() {
    let results = run_all_tutorials();
    assert_eq!(results.len(), 10);
    for (name, success) in results {
        assert!(success, "Tutorial chapter '{}' failed execution!", name);
    }
}

#[test]
fn test_ch01_single_leaf_lifecycle_and_adversarial() {
    let dry = ch01_single_leaf::run();
    assert_eq!(dry.name, "Valen");
    assert_eq!(dry.discipline, "Aether-Weaver");

    // Adversarial: Premature resolution returns WetChoice
    let wet = ch01_single_leaf::CrafterIdentity::builder()
        .name("Test".to_string())
        .discipline(SingleChoice::new("Discipline", vec!["A".to_string()]))
        .build();
    assert!(!wet.is_complete());
    assert!(matches!(wet.resolve(), Err(FormworkError::WetChoice { .. })));
}

#[test]
fn test_ch02_option_leaf_omission_and_adversarial() {
    let (unaligned, blessed) = ch02_option_leaf::run();
    assert_eq!(unaligned.celestial_blessing, None);
    assert_eq!(
        blessed.celestial_blessing,
        Some("Dawn-Crest".to_string())
    );

    // Adversarial: Pending choice in optional slot prevents resolution
    let wet = ch02_option_leaf::CrafterAffiliation::builder()
        .crafter_name("Test".to_string())
        .celestial_blessing(Some(SingleChoice::new("Blessing", vec!["B".to_string()])))
        .build();
    assert!(!wet.is_complete());
    assert!(matches!(wet.resolve(), Err(FormworkError::WetChoice { .. })));
}

#[test]
fn test_ch03_collection_leaf_piecemeal_and_adversarial() {
    let armory = ch03_collection_leaf::run();
    assert_eq!(armory.tools.len(), 2);
    assert_eq!(armory.tools[0], "Aether-Chisel");
    assert_eq!(armory.tools[1], "Astral-Compass");

    // Adversarial: Single unconfirmed choice in collection prevents resolution
    let wet = ch03_collection_leaf::ArtisanToolbelt::builder()
        .owner("Test".to_string())
        .with_tools("Fixed".to_string())
        .with_tools(SingleChoice::new("Choice", vec!["Option".to_string()]))
        .build();
    assert!(!wet.is_complete());
    assert!(matches!(wet.resolve(), Err(FormworkError::WetChoice { .. })));
}

#[test]
fn test_ch04_pick_leaf_cardinality_and_adversarial() {
    let matrix = ch04_pick_leaf::run();
    assert_eq!(matrix.attuned_runes.len(), 2);
    assert_eq!(matrix.attuned_runes[0], "Solar");
    assert_eq!(matrix.attuned_runes[1], "Starlight");

    // Adversarial: Under-selecting options fails cardinality validation
    let mut under_choice: MultiChoice<String> = MultiChoice::new("Pick 2", vec!["A".to_string(), "B".to_string()])
        .with_cardinality(Cardinality::Exact(2));
    let _ = under_choice.select(0);
    assert!(under_choice.confirm().is_err());
}

#[test]
fn test_ch05_single_cascade_deep_resolution() {
    let sanctum = ch05_single_cascade::run();
    assert_eq!(sanctum.name, "Aethel-Vessel");
    assert_eq!(sanctum.core.catalyst_crystal, "Sun-Shard");

    // Adversarial: Unconfirmed child choice cascades to parent incompleteness
    let child = ch05_single_cascade::CoreSanctum::builder()
        .sanctum_id("C1".to_string())
        .catalyst_crystal(SingleChoice::new("Catalyst", vec!["C".to_string()]))
        .build();
    let parent = ch05_single_cascade::FloatingSanctum::builder()
        .name("P1".to_string())
        .core(child)
        .build();
    assert!(!parent.is_complete());
    assert!(matches!(parent.resolve(), Err(FormworkError::WetChoice { .. })));
}

#[test]
fn test_ch06_option_cascade_skipping_and_visiting() {
    let (light, heavy) = ch06_option_cascade::run();
    assert!(light.auxiliary_wing.is_none());
    assert_eq!(
        heavy.auxiliary_wing.unwrap().stabilizer_trim,
        "High-Lift"
    );
}

#[test]
fn test_ch07_collection_cascade_grid_ordering() {
    let citadel = ch07_collection_cascade::run();
    assert_eq!(citadel.bastions.len(), 2);
    assert_eq!(citadel.bastions[0].position, "Fore-Port");
    assert_eq!(citadel.bastions[0].beam_frequency, "Beam");
    assert_eq!(citadel.bastions[1].position, "Aft-Starboard");
    assert_eq!(citadel.bastions[1].beam_frequency, "Beam");
}

#[test]
fn test_ch08_pick_cascade_filtering_and_curation() {
    let roster = ch08_pick_cascade::run();
    assert_eq!(roster.drafted_guards.len(), 2);
    assert_eq!(roster.drafted_guards[0].callsign, "Astra-1");
    assert_eq!(roster.drafted_guards[1].callsign, "Astra-2");
}

#[test]
fn test_ch09_traversal_orders_and_session_rewind() {
    let (bfs_order, dfs_order) = ch09_traversal_tactics::run();

    // Verify BFS vs DFS differences
    assert_ne!(bfs_order, dfs_order);
    assert_eq!(
        bfs_order,
        vec!["Vessel Stance", "Vessel Horn", "Compartment Valve"]
    );
    assert_eq!(
        dfs_order,
        vec!["Vessel Stance", "Compartment Valve", "Vessel Horn"]
    );
}

#[test]
fn test_ch10_realm_crafter_capstone_trial() {
    let report = ch10_realm_crafter_trial::run();
    assert!(report.survived);
    assert_eq!(report.hull_remaining, 100);
    assert!(report.chronicle.iter().any(|line| line.contains("VICTORY")));
}

// ===========================================================================
// Comprehensive Adversarial & Boundary Validation Tests
// ===========================================================================

struct BreakImmediateVisitor;

impl<T> ChoiceVisitor<T> for BreakImmediateVisitor {
    fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
        VisitorFlow::Break
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
        VisitorFlow::Break
    }
}

struct FlowControllerVisitor {
    flow: VisitorFlow,
    auto_select: bool,
}

impl<T> ChoiceVisitor<T> for FlowControllerVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        if self.auto_select && !choice.is_made() && !choice.options().is_empty() {
            let _ = choice.select(0);
            let _ = choice.confirm();
        }
        self.flow
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        if self.auto_select && !choice.is_made() && !choice.options().is_empty() {
            let _ = choice.select(0);
            let _ = choice.confirm();
        }
        self.flow
    }
}

#[test]
fn test_adversarial_invalid_cardinality_in_pick_slots() {
    // 1. Exact(2) constraints
    let mut exact_choice = MultiChoice::<String>::new(
        "Pick 2",
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
    )
    .with_cardinality(Cardinality::Exact(2));

    // 0 selected: confirm fails
    assert!(matches!(
        exact_choice.confirm(),
        Err(FormworkError::InvalidCardinality {
            expected: Cardinality::Exact(2),
            actual: 0,
            ..
        })
    ));

    // 1 selected: confirm fails
    exact_choice.select(0).unwrap();
    assert!(matches!(
        exact_choice.confirm(),
        Err(FormworkError::InvalidCardinality {
            expected: Cardinality::Exact(2),
            actual: 1,
            ..
        })
    ));

    // Duplicate selection rejected
    assert!(matches!(
        exact_choice.select(0),
        Err(FormworkError::DuplicateSelection { .. })
    ));

    // Out of bounds selection rejected
    assert!(matches!(
        exact_choice.select(99),
        Err(FormworkError::IndexOutOfBounds { .. })
    ));

    // 2 selected: confirm succeeds
    exact_choice.select(1).unwrap();
    assert!(exact_choice.confirm().is_ok());

    // 2. AtLeast(2) constraints
    let mut at_least_choice = MultiChoice::<String>::new(
        "Pick At Least 2",
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
    )
    .with_cardinality(Cardinality::AtLeast(2));
    at_least_choice.select(0).unwrap();
    assert!(at_least_choice.confirm().is_err());
    at_least_choice.select(1).unwrap();
    assert!(at_least_choice.confirm().is_ok());
    at_least_choice.select(2).unwrap();
    assert!(at_least_choice.confirm().is_ok());

    // 3. UpTo(2) constraints
    let mut up_to_choice = MultiChoice::<String>::new(
        "Pick Up To 2",
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
    )
    .with_cardinality(Cardinality::UpTo(2));
    assert!(up_to_choice.confirm().is_ok()); // 0 selected is accepted

    let mut up_to_choice2 = MultiChoice::<String>::new(
        "Pick Up To 2",
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
    )
    .with_cardinality(Cardinality::UpTo(2));
    up_to_choice2.select(0).unwrap();
    up_to_choice2.select(1).unwrap();
    up_to_choice2.select(2).unwrap();
    assert!(matches!(
        up_to_choice2.confirm(),
        Err(FormworkError::InvalidCardinality {
            expected: Cardinality::UpTo(2),
            actual: 3,
            ..
        })
    ));

    // 4. Domain integration with Chapter 4 (RunicMatrix)
    let under_selected_matrix = ch04_pick_leaf::RunicMatrix::builder()
        .matrix_name("Under-Attuned".to_string())
        .attuned_runes(
            MultiChoice::new(
                "Pick 2",
                vec!["Solar".to_string(), "Starlight".to_string(), "Void".to_string()],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )
        .build();
    assert!(!under_selected_matrix.is_complete());
    assert!(matches!(
        under_selected_matrix.resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    // 5. Domain integration with Chapter 8 (SkyguardRoster)
    let g1 = ch08_pick_cascade::Skyguard::builder()
        .callsign("G1".to_string())
        .specialty(SingleChoice::new("S1", vec!["Lancer".to_string()]))
        .build();
    let g2 = ch08_pick_cascade::Skyguard::builder()
        .callsign("G2".to_string())
        .specialty(SingleChoice::new("S2", vec!["Sniper".to_string()]))
        .build();
    let under_roster = ch08_pick_cascade::SkyguardRoster::builder()
        .squad_id("Under-Drafted".to_string())
        .drafted_guards(
            MultiChoice::new("Draft 2", vec![g1, g2])
                .with_cardinality(Cardinality::Exact(2)),
        )
        .build();
    assert!(!under_roster.is_complete());
    assert!(matches!(
        under_roster.resolve(),
        Err(FormworkError::WetChoice { .. })
    ));
}

#[test]
fn test_adversarial_premature_resolution_all_8_slot_kinds() {
    // Slot 1: Single Leaf (ch01)
    let mut choice1 = SingleChoice::new("Discipline", vec!["Weaver".to_string()]);
    let wet1 = ch01_single_leaf::CrafterIdentity::builder()
        .name("Test".to_string())
        .discipline(choice1.clone())
        .build();
    assert!(!wet1.is_complete());
    assert!(matches!(wet1.resolve(), Err(FormworkError::WetChoice { .. })));

    choice1.select(0).unwrap(); // staged, unconfirmed
    let wet1_staged = ch01_single_leaf::CrafterIdentity::builder()
        .name("Test".to_string())
        .discipline(choice1)
        .build();
    assert!(!wet1_staged.is_complete());
    assert!(matches!(wet1_staged.resolve(), Err(FormworkError::WetChoice { .. })));

    // Slot 2: Option Leaf (ch02)
    let mut choice2 = SingleChoice::new("Blessing", vec!["B1".to_string()]);
    let wet2 = ch02_option_leaf::CrafterAffiliation::builder()
        .crafter_name("Test".to_string())
        .celestial_blessing(Some(choice2.clone()))
        .build();
    assert!(!wet2.is_complete());
    assert!(matches!(wet2.resolve(), Err(FormworkError::WetChoice { .. })));

    choice2.select(0).unwrap();
    let wet2_staged = ch02_option_leaf::CrafterAffiliation::builder()
        .crafter_name("Test".to_string())
        .celestial_blessing(Some(choice2))
        .build();
    assert!(!wet2_staged.is_complete());
    assert!(matches!(wet2_staged.resolve(), Err(FormworkError::WetChoice { .. })));

    // Slot 3: Collection Leaf (ch03)
    let mut choice3 = SingleChoice::new("Tool", vec!["Chisel".to_string()]);
    let wet3 = ch03_collection_leaf::ArtisanToolbelt::builder()
        .owner("Test".to_string())
        .with_tools("FixedHammer".to_string())
        .with_tools(choice3.clone())
        .build();
    assert!(!wet3.is_complete());
    assert!(matches!(wet3.resolve(), Err(FormworkError::WetChoice { .. })));

    choice3.select(0).unwrap();
    let wet3_staged = ch03_collection_leaf::ArtisanToolbelt::builder()
        .owner("Test".to_string())
        .with_tools("FixedHammer".to_string())
        .with_tools(choice3)
        .build();
    assert!(!wet3_staged.is_complete());
    assert!(matches!(wet3_staged.resolve(), Err(FormworkError::WetChoice { .. })));

    // Slot 4: Pick Leaf (ch04)
    let mut choice4 = MultiChoice::<String>::new("Runes", vec!["R1".to_string(), "R2".to_string()])
        .with_cardinality(Cardinality::Exact(1));
    let wet4 = ch04_pick_leaf::RunicMatrix::builder()
        .matrix_name("Test".to_string())
        .attuned_runes(choice4.clone())
        .build();
    assert!(!wet4.is_complete());
    assert!(matches!(wet4.resolve(), Err(FormworkError::WetChoice { .. })));

    choice4.select(0).unwrap(); // staged, unconfirmed
    let wet4_staged = ch04_pick_leaf::RunicMatrix::builder()
        .matrix_name("Test".to_string())
        .attuned_runes(choice4)
        .build();
    assert!(!wet4_staged.is_complete());
    assert!(matches!(wet4_staged.resolve(), Err(FormworkError::WetChoice { .. })));

    // Slot 5: Single Cascade (ch05)
    let mut child5_choice = SingleChoice::new("Crystal", vec!["Cr1".to_string()]);
    let child5 = ch05_single_cascade::CoreSanctum::builder()
        .sanctum_id("C5".to_string())
        .catalyst_crystal(child5_choice.clone())
        .build();
    let wet5 = ch05_single_cascade::FloatingSanctum::builder()
        .name("P5".to_string())
        .core(child5)
        .build();
    assert!(!wet5.is_complete());
    assert!(matches!(wet5.resolve(), Err(FormworkError::WetChoice { .. })));

    child5_choice.select(0).unwrap();
    let child5_staged = ch05_single_cascade::CoreSanctum::builder()
        .sanctum_id("C5".to_string())
        .catalyst_crystal(child5_choice)
        .build();
    let wet5_staged = ch05_single_cascade::FloatingSanctum::builder()
        .name("P5".to_string())
        .core(child5_staged)
        .build();
    assert!(!wet5_staged.is_complete());
    assert!(matches!(wet5_staged.resolve(), Err(FormworkError::WetChoice { .. })));

    // Slot 6: Option Cascade (ch06)
    let mut child6_choice = SingleChoice::new("Trim", vec!["T1".to_string()]);
    let child6 = ch06_option_cascade::OutriggerWing::builder()
        .wing_id("W6".to_string())
        .stabilizer_trim(child6_choice.clone())
        .build();
    let wet6 = ch06_option_cascade::SkyVessel::builder()
        .vessel_name("V6".to_string())
        .auxiliary_wing(child6)
        .build();
    assert!(!wet6.is_complete());
    assert!(matches!(wet6.resolve(), Err(FormworkError::WetChoice { .. })));

    child6_choice.select(0).unwrap();
    let child6_staged = ch06_option_cascade::OutriggerWing::builder()
        .wing_id("W6".to_string())
        .stabilizer_trim(child6_choice)
        .build();
    let wet6_staged = ch06_option_cascade::SkyVessel::builder()
        .vessel_name("V6".to_string())
        .auxiliary_wing(child6_staged)
        .build();
    assert!(!wet6_staged.is_complete());
    assert!(matches!(wet6_staged.resolve(), Err(FormworkError::WetChoice { .. })));

    // Slot 7: Collection Cascade (ch07)
    let mut b1_choice = SingleChoice::new("Freq", vec!["F1".to_string()]);
    b1_choice.select(0).unwrap();
    b1_choice.confirm().unwrap();
    let b1 = ch07_collection_cascade::DefenseBastion::builder()
        .position("P1".to_string())
        .beam_frequency(b1_choice)
        .build();
    let b2_unconfirmed = ch07_collection_cascade::DefenseBastion::builder()
        .position("P2".to_string())
        .beam_frequency(SingleChoice::new("Freq", vec!["F2".to_string()]))
        .build();
    let wet7 = ch07_collection_cascade::DefenseCitadel::builder()
        .citadel_name("C7".to_string())
        .with_bastions(b1)
        .with_bastions(b2_unconfirmed)
        .build();
    assert!(!wet7.is_complete());
    assert!(matches!(wet7.resolve(), Err(FormworkError::WetChoice { .. })));

    // Slot 8: Pick Cascade (ch08)
    let g1 = ch08_pick_cascade::Skyguard::builder()
        .callsign("G1".to_string())
        .specialty(SingleChoice::new("Specialty", vec!["S1".to_string()]))
        .build();
    let g2 = ch08_pick_cascade::Skyguard::builder()
        .callsign("G2".to_string())
        .specialty(SingleChoice::new("Specialty", vec!["S2".to_string()]))
        .build();

    // 8a: Candidate pool itself unconfirmed
    let wet8_pending_pool = ch08_pick_cascade::SkyguardRoster::builder()
        .squad_id("S8a".to_string())
        .drafted_guards(
            MultiChoice::new("Draft 1", vec![g1.clone(), g2.clone()])
                .with_cardinality(Cardinality::Exact(1)),
        )
        .build();
    assert!(!wet8_pending_pool.is_complete());
    assert!(matches!(wet8_pending_pool.resolve(), Err(FormworkError::WetChoice { .. })));

    // 8b: Candidate pool confirmed, but drafted child has open internal choice
    let mut drafted_pool = MultiChoice::new("Draft 1", vec![g1, g2])
        .with_cardinality(Cardinality::Exact(1));
    drafted_pool.select(0).unwrap();
    drafted_pool.confirm().unwrap();

    let wet8_pending_child = ch08_pick_cascade::SkyguardRoster::builder()
        .squad_id("S8b".to_string())
        .drafted_guards(drafted_pool)
        .build();
    assert!(!wet8_pending_child.is_complete());
    assert!(matches!(wet8_pending_child.resolve(), Err(FormworkError::WetChoice { .. })));
}

#[test]
fn test_adversarial_visitor_flow_breaks_and_rewinds() {
    // 1. VisitorFlow::Break halts walk immediately
    let sub = ch09_traversal_tactics::SubCompartment::builder()
        .label("Sub".to_string())
        .valve(SingleChoice::new("Valve", vec!["Open".to_string()]))
        .build();
    let mut vessel = ch09_traversal_tactics::VesselBfs::builder()
        .stance(SingleChoice::new("Stance", vec!["Cruise".to_string()]))
        .compartment(sub)
        .horn(SingleChoice::new("Horn", vec!["Honk".to_string()]))
        .build();

    vessel.walk_choices(&mut BreakImmediateVisitor);
    assert!(!vessel.is_complete());
    assert!(matches!(vessel.resolve(), Err(FormworkError::WetChoice { .. })));

    // 2. Interactive stepping and rewinding with VisitorFlow::Back
    let sub_session = ch09_traversal_tactics::SubCompartment::builder()
        .label("Sub-S".to_string())
        .valve(SingleChoice::new("Valve", vec!["Open".to_string()]))
        .build();
    let mut vessel_session = ch09_traversal_tactics::VesselBfs::builder()
        .stance(SingleChoice::new("Stance", vec!["Cruise".to_string()]))
        .compartment(sub_session)
        .horn(SingleChoice::new("Horn", vec!["Honk".to_string()]))
        .build();

    let mut session = vessel_session.session();
    assert_eq!(session.current_step(), 0);

    // Step 0: advance with Continue and select option
    let mut step_and_continue = FlowControllerVisitor {
        flow: VisitorFlow::Continue,
        auto_select: true,
    };
    let res0 = session.step(&mut step_and_continue);
    assert_eq!(res0, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Step 1: backtrack with Back!
    let mut step_and_back = FlowControllerVisitor {
        flow: VisitorFlow::Back,
        auto_select: false,
    };
    let res_back = session.step(&mut step_and_back);
    assert_eq!(res_back, StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // Step 0 again: advance to step 1
    let res0_re = session.step(&mut step_and_continue);
    assert_eq!(res0_re, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Step 1: advance to step 2
    let res1 = session.step(&mut step_and_continue);
    assert_eq!(res1, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 2);

    // Backtrack from step 2 to 1, and 1 to 0
    let res_back2 = session.step(&mut step_and_back);
    assert_eq!(res_back2, StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 1);

    let res_back1 = session.step(&mut step_and_back);
    assert_eq!(res_back1, StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // Calling Back at step 0 is safe (no underflow, returns StepResult::AtStart)
    let res_back0 = session.step(&mut step_and_back);
    assert_eq!(res_back0, StepResult::AtStart);
    assert_eq!(session.current_step(), 0);

    // 3. Cascading boundary rewind in DFS
    let sub_dfs = ch09_traversal_tactics::SubCompartment::builder()
        .label("Sub-DFS".to_string())
        .valve(SingleChoice::new("Valve", vec!["Open".to_string()]))
        .build();
    let mut vessel_dfs = ch09_traversal_tactics::VesselDfs::builder()
        .stance(SingleChoice::new("Stance", vec!["Cruise".to_string()]))
        .compartment(sub_dfs)
        .horn(SingleChoice::new("Horn", vec!["Honk".to_string()]))
        .build();

    let mut dfs_session = vessel_dfs.session();
    let _ = dfs_session.step(&mut step_and_continue);
    assert_eq!(dfs_session.current_step(), 1);

    // Step 1: inside child compartment! Returning Back rewinds out across boundary
    let res_dfs_back = dfs_session.step(&mut step_and_back);
    assert_eq!(res_dfs_back, StepResult::Choice(VisitorFlow::Back));
    assert_eq!(dfs_session.current_step(), 0);
}

#[test]
fn test_adversarial_no_dnd_5e_terms_in_tutorial_source() {
    let tutorial_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tutorial");
    assert!(tutorial_dir.exists(), "src/tutorial directory must exist");

    let forbidden_terms = [
        "barbarian",
        "longsword",
        "spell slot",
        "spell slots",
        "d&d",
        "dnd",
        "5e",
        "dungeon master",
        "passive perception",
        "hit dice",
        "paladin",
        "warlock",
        "sorcerer",
        "bard",
        "druid",
        "cleric",
        "cantrip",
        "saving throw",
        "armor class",
    ];

    for entry in fs::read_dir(&tutorial_dir).expect("read tutorial dir") {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read file {}: {}", path.display(), e));
            let content_lower = content.to_lowercase();
            for term in &forbidden_terms {
                assert!(
                    !content_lower.contains(term),
                    "Forbidden D&D/5e term '{}' found in '{}'!",
                    term,
                    path.display()
                );
            }
        }
    }
}

#[test]
fn test_adversarial_every_struct_uses_builder_attribute() {
    let tutorial_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tutorial");

    for entry in fs::read_dir(&tutorial_dir).expect("read tutorial dir") {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = fs::read_to_string(&path).expect("read file");
            let lines: Vec<&str> = content.lines().collect();
            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("pub struct ") && trimmed.ends_with('{') {
                    let struct_name = trimmed
                        .strip_prefix("pub struct ")
                        .unwrap()
                        .strip_suffix('{')
                        .unwrap()
                        .trim();
                    // Ignore visitor structs and report helper structs
                    if struct_name.ends_with("Selector")
                        || struct_name.ends_with("Attuner")
                        || struct_name.ends_with("Drafter")
                        || struct_name.ends_with("Recorder")
                        || struct_name.ends_with("Visitor")
                        || struct_name == "FlightReport"
                    {
                        continue;
                    }

                    // Check preceding lines for #[formwork(builder
                    let has_formwork_builder = (0..idx).rev().take(10).any(|i| {
                        lines[i].contains("#[formwork(") && lines[i].contains("builder")
                    });

                    assert!(
                        has_formwork_builder,
                        "Struct '{}' in file '{}' does NOT have #[formwork(builder)] attribute!",
                        struct_name,
                        path.display()
                    );
                }
            }
        }
    }

    // Direct invocation of all 20 domain struct builders:
    let _ = ch01_single_leaf::CrafterIdentityFormwork::builder();
    let _ = ch02_option_leaf::CrafterAffiliationFormwork::builder();
    let _ = ch03_collection_leaf::ArtisanToolbeltFormwork::builder();
    let _ = ch04_pick_leaf::RunicMatrixFormwork::builder();
    let _ = ch05_single_cascade::CoreSanctumFormwork::builder();
    let _ = ch05_single_cascade::FloatingSanctumFormwork::builder();
    let _ = ch06_option_cascade::OutriggerWingFormwork::builder();
    let _ = ch06_option_cascade::SkyVesselFormwork::builder();
    let _ = ch07_collection_cascade::DefenseBastionFormwork::builder();
    let _ = ch07_collection_cascade::DefenseCitadelFormwork::builder();
    let _ = ch08_pick_cascade::SkyguardFormwork::builder();
    let _ = ch08_pick_cascade::SkyguardRosterFormwork::builder();
    let _ = ch09_traversal_tactics::SubCompartmentFormwork::builder();
    let _ = ch09_traversal_tactics::VesselBfsFormwork::builder();
    let _ = ch09_traversal_tactics::VesselDfsFormwork::builder();
    let _ = ch10_realm_crafter_trial::AetherEngineFormwork::builder();
    let _ = ch10_realm_crafter_trial::FlightWingFormwork::builder();
    let _ = ch10_realm_crafter_trial::HullTurretFormwork::builder();
    let _ = ch10_realm_crafter_trial::BridgeOfficerFormwork::builder();
    let _ = ch10_realm_crafter_trial::SovereignSpireSkiffFormwork::builder();
}

#[test]
fn test_adversarial_pick_cascade_unselected_candidate_isolation() {
    let mut g1_choice = SingleChoice::new("Specialty", vec!["Lancer".to_string()]);
    g1_choice.select(0).unwrap();
    g1_choice.confirm().unwrap();
    let g1 = ch08_pick_cascade::Skyguard::builder()
        .callsign("G1".to_string())
        .specialty(g1_choice)
        .build();

    let mut g2_choice = SingleChoice::new("Specialty", vec!["Sniper".to_string()]);
    g2_choice.select(0).unwrap();
    g2_choice.confirm().unwrap();
    let g2 = ch08_pick_cascade::Skyguard::builder()
        .callsign("G2".to_string())
        .specialty(g2_choice)
        .build();

    // g3 has an open UNCONFIRMED choice!
    let g3_unconfirmed = ch08_pick_cascade::Skyguard::builder()
        .callsign("G3".to_string())
        .specialty(SingleChoice::new("Specialty", vec!["Medic".to_string()]))
        .build();

    // Draft only candidates 0 and 1
    let mut candidate_pool = MultiChoice::new("Draft 2", vec![g1, g2, g3_unconfirmed])
        .with_cardinality(Cardinality::Exact(2));
    candidate_pool.select(0).unwrap();
    candidate_pool.select(1).unwrap();
    candidate_pool.confirm().unwrap();

    let roster_wet = ch08_pick_cascade::SkyguardRoster::builder()
        .squad_id("Elite-Duo".to_string())
        .drafted_guards(candidate_pool)
        .build();

    // Even though unselected candidate g3 has an open choice, it was not drafted.
    // Therefore, the roster is complete and cures cleanly!
    assert!(roster_wet.is_complete());
    let dry = roster_wet
        .resolve()
        .expect("unselected candidate does not block resolution");
    assert_eq!(dry.drafted_guards.len(), 2);
    assert_eq!(dry.drafted_guards[0].callsign, "G1");
    assert_eq!(dry.drafted_guards[1].callsign, "G2");
}

#[test]
fn test_adversarial_ch10_capstone_catastrophic_failure() {
    // Build a fragile, sub-standard skiff designed to fail the rift storm
    let mut wet_engine = ch10_realm_crafter_trial::AetherEngine::builder()
        .engine_code("AE-DOOM".to_string())
        .catalyst_core(SingleChoice::new(
            "Core",
            vec!["Shadow-Essence".to_string()],
        ))
        .build();
    let mut visitor = ch10_realm_crafter_trial::ExpeditionVisitor;
    wet_engine.walk_choices(&mut visitor);
    let dry_engine = wet_engine.resolve().unwrap();

    let doomed_skiff = ch10_realm_crafter_trial::SovereignSpireSkiff {
        skiff_name: "The Doomed".to_string(),
        hull_integrity: 40, // Low starting hull
        tactical_stance: "Evasive".to_string(),
        engine: dry_engine,
        auxiliary_wing: None, // No wing -> -25 hull
        turrets: vec![],     // No turrets
        officers: vec![],    // No officers -> defense score 0 < 60 -> -40 hull
    };

    let report = ch10_realm_crafter_trial::navigate_rift_storm(&doomed_skiff);
    assert!(!report.survived);
    assert_eq!(report.hull_remaining, 0);
    assert!(report.chronicle.iter().any(|line| line.contains("CRITICAL FAILURE")));
}

