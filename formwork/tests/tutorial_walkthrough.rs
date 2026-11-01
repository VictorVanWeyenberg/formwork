//! Comprehensive integration test suite verifying all 10 tutorial chapters end-to-end.

use formwork::choice::{Cardinality, MultiChoice, SingleChoice};
use formwork::tutorial::*;
use formwork::{Formwork, FormworkError};

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
