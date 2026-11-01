//! # The Chronicle of the Realm-Crafter: A Guided Formwork Tutorial
//!
//! Welcome, Architect, to **Aethelgard**—a mythical realm fractured into floating sky-islands
//! by the primordial Voidstorm. The only beings capable of braving the abyssal expanse and
//! weaving the fractured skylands back together are the **Realm-Crafters**.
//!
//! In Aethelgard, the **Formwork** metaphor is living celestial magic:
//!
//! - **The Wet Weave (Fluid Formwork)**: When shaping blueprints, reality is fluid and undecided.
//!   Slots hold open choices, candidate runes, and unconfirmed attunements. Decisions can be
//!   inspected, reconsidered, stepped one by one, or rewound with history intact.
//! - **Curing (The Watertight Dry Domain Model)**: Once every runic slot is satisfied and validated,
//!   the Crafter invokes resolution. The fluid weave crystallizes into permanent, impenetrable
//!   celestial matter—rigid, durable, watertight against the howling void, with zero choice wrappers
//!   or runtime uncertainties.
//!
//! ## Curriculum Overview
//!
//! This tutorial spans 10 progressive chapters covering the entire `formwork` feature set:
//!
//! ### Act I: The Artisan's Awakening (Leaf Slots)
//! 1. [`ch01_single_leaf`]: Single scalar choices with `#[slot]` (Keystone of Dawn).
//! 2. [`ch02_option_leaf`]: Optional choices with `#[slot(option)]` (Celestial Patron Blessing).
//! 3. [`ch03_collection_leaf`]: Ordered lists of items and choices with `#[slot(collection)]` (Relic Armory).
//! 4. [`ch04_pick_leaf`]: Candidate pool selection with cardinality constraints via `#[slot(pick)]` (Council Attunements).
//!
//! ### Act II: Spires & Citadels (Cascading Slots)
//! 5. [`ch05_single_cascade`]: Nested child formworks with `#[slot(cascade)]` (Astral Core Sanctum).
//! 6. [`ch06_option_cascade`]: Optional child formworks with `#[slot(option, cascade)]` (Outrigger Wing).
//! 7. [`ch07_collection_cascade`]: Ordered lists of child formworks with `#[slot(collection, cascade)]` (Defense Bastion Array).
//! 8. [`ch08_pick_cascade`]: Multi-choice candidate drafting of child formworks with `#[slot(pick, cascade)]` (Skyguard Retinue).
//!
//! ### Act III: The Weave of Sight (Traversal & Capstone)
//! 9. [`ch09_traversal_tactics`]: Traversal orders (`traverse = "bfs"` vs `"dfs"`), interactive stepping via `FormworkSession`, and rewinding with `VisitorFlow::Back`.
//! 10. [`ch10_realm_crafter_trial`]: The Capstone Trial—assembling the complete Sovereign Spire-Skiff and conquering the Aetherial Rift Storm.
//!
//! ## Running the Entire Tutorial Journey
//!
//! You can execute all chapters programmatically using [`run_all_tutorials`].

pub mod ch01_single_leaf;
pub mod ch02_option_leaf;
pub mod ch03_collection_leaf;
pub mod ch04_pick_leaf;
pub mod ch05_single_cascade;
pub mod ch06_option_cascade;
pub mod ch07_collection_cascade;
pub mod ch08_pick_cascade;
pub mod ch09_traversal_tactics;
pub mod ch10_realm_crafter_trial;

/// Runs all 10 tutorial chapters in sequence, verifying each chapter's assertions and domain outcomes.
///
/// Returns a list of chapter names and their execution status upon complete success.
///
/// # Example
///
/// ```rust
/// use formwork::tutorial::run_all_tutorials;
///
/// let results = run_all_tutorials();
/// assert_eq!(results.len(), 10);
/// ```
pub fn run_all_tutorials() -> Vec<(&'static str, bool)> {
    let mut outcomes = Vec::new();

    // Chapter 1: Single Leaf
    let ch01 = ch01_single_leaf::run();
    outcomes.push(("ch01_single_leaf", ch01.discipline == "Aether-Weaver"));

    // Chapter 2: Option Leaf
    let (boon_none, boon_some) = ch02_option_leaf::run();
    outcomes.push((
        "ch02_option_leaf",
        boon_none.celestial_blessing.is_none()
            && boon_some.celestial_blessing == Some("Dawn-Crest".to_string()),
    ));

    // Chapter 3: Collection Leaf
    let armory = ch03_collection_leaf::run();
    outcomes.push((
        "ch03_collection_leaf",
        armory.tools.len() == 2 && armory.tools.contains(&"Aether-Chisel".to_string()),
    ));

    // Chapter 4: Pick Leaf
    let attunement = ch04_pick_leaf::run();
    outcomes.push(("ch04_pick_leaf", attunement.attuned_runes.len() == 2));

    // Chapter 5: Single Cascade
    let sanctum = ch05_single_cascade::run();
    outcomes.push((
        "ch05_single_cascade",
        sanctum.core.catalyst_crystal == "Sun-Shard",
    ));

    // Chapter 6: Option Cascade
    let (vessel_none, vessel_some) = ch06_option_cascade::run();
    outcomes.push((
        "ch06_option_cascade",
        vessel_none.auxiliary_wing.is_none() && vessel_some.auxiliary_wing.is_some(),
    ));

    // Chapter 7: Collection Cascade
    let bastion_grid = ch07_collection_cascade::run();
    outcomes.push((
        "ch07_collection_cascade",
        bastion_grid.bastions.len() == 2,
    ));

    // Chapter 8: Pick Cascade
    let squad = ch08_pick_cascade::run();
    outcomes.push(("ch08_pick_cascade", squad.drafted_guards.len() == 2));

    // Chapter 9: Traversal Tactics
    let (bfs_order, dfs_order) = ch09_traversal_tactics::run();
    outcomes.push((
        "ch09_traversal_tactics",
        bfs_order != dfs_order,
    ));

    // Chapter 10: Capstone Trial
    let report = ch10_realm_crafter_trial::run();
    outcomes.push(("ch10_realm_crafter_trial", report.survived));

    outcomes
}
