//! # Chapter 10: The Sovereign Spire-Skiff (The Grand Capstone Trial)
//!
//! The culmination of the Realm-Crafter's journey has arrived. To break through the impenetrable
//! **Aetherial Rift Storm** and reunite the fractured skylands of Aethelgard, the Crafter must weave
//! together every single subsystem into a single, magnificent **Sovereign Spire-Skiff**.
//!
//! ## Concept: The Grand Composite Formwork
//!
//! In this capstone trial, we unite all dimensions of Higher-Kinded Data formworks:
//!
//! - **Invariant Fields**: Fixed parameters such as `skiff_name` and initial `hull_integrity`.
//! - **Single Leaf Slot (`#[slot]`)**: Tactical helm stance (`tactical_stance`).
//! - **Single Cascade Slot (`#[slot(cascade)]`)**: Power chamber (`engine: AetherEngine`).
//! - **Option Cascade Slot (`#[slot(option, cascade)]`)**: Deployable flight wing (`auxiliary_wing: Option<FlightWing>`).
//! - **Collection Cascade Slot (`#[slot(collection, cascade)]`)**: Perimeter turret grid (`turrets: Vec<HullTurret>`).
//! - **Pick Cascade Slot (`#[slot(pick, cascade)]`)**: Bridge command staff drafted from a candidate pool (`officers: Vec<BridgeOfficer>`).
//!
//! Once all choices are visited and confirmed, `.resolve()` cures the entire composite tree into
//! a solid, watertight dry domain struct. This watertight model is then fed into an interactive
//! text-adventure flight simulator (`navigate_rift_storm`), where pure domain logic determines survival!
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::tutorial::ch10_realm_crafter_trial::{run, FlightReport};
//!
//! let report: FlightReport = run();
//! assert!(report.survived);
//! assert!(report.hull_remaining > 0);
//! ```

use crate::choice::{Cardinality, MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, formwork};

// ---------------------------------------------------------------------------
// 1. Domain Entities
// ---------------------------------------------------------------------------

/// The heart engine powering the spire-skiff.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct AetherEngine {
    /// Engine model designation.
    pub engine_code: String,
    /// Catalyst core chosen to feed the drive conduits (single leaf slot).
    #[slot]
    pub catalyst_core: String,
}

/// A deployable auxiliary flight wing.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct FlightWing {
    /// Wing strut serial.
    pub wing_code: String,
    /// Configured aerodynamic flap geometry (single leaf slot).
    #[slot]
    pub flap_profile: String,
}

/// A hull-mounted defense turret.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct HullTurret {
    /// Turret mount location label.
    pub turret_label: String,
    /// Directed beam energy type (single leaf slot).
    #[slot]
    pub beam_resonance: String,
}

/// A bridge officer directing skiff operations.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct BridgeOfficer {
    /// Officer callsign.
    pub callsign: String,
    /// Duty role on the bridge (single leaf slot).
    #[slot]
    pub station_role: String,
}

/// The grand composite vessel: The Sovereign Spire-Skiff.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SovereignSpireSkiff {
    /// Registered name of the vessel.
    pub skiff_name: String,
    /// Base structural integrity.
    pub hull_integrity: u32,
    /// Pilot helm stance (single leaf slot).
    #[slot]
    pub tactical_stance: String,
    /// Power engine (single cascade slot).
    #[slot(cascade)]
    pub engine: AetherEngine,
    /// Optional aerodynamic stabilizer (option cascade slot).
    #[slot(option, cascade)]
    pub auxiliary_wing: Option<FlightWing>,
    /// Defensive battery (collection cascade slot).
    #[slot(collection, cascade)]
    pub turrets: Vec<HullTurret>,
    /// Bridge command retinue drafted from candidate pool (pick cascade slot).
    #[slot(pick, cascade)]
    pub officers: Vec<BridgeOfficer>,
}

// ---------------------------------------------------------------------------
// 2. Automated Expedition Visitor
// ---------------------------------------------------------------------------

/// A visitor configured to automatically select optimal choices for the storm expedition.
pub struct ExpeditionVisitor;

impl<T> ChoiceVisitor<T> for ExpeditionVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        if !choice.is_made() && !choice.options().is_empty() {
            let _ = choice.select(0);
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        if !choice.is_made() {
            let target_count = match choice.cardinality {
                Cardinality::Exact(n) => n,
                Cardinality::AtLeast(n) => n,
                Cardinality::UpTo(n) => n.min(choice.options().len()),
            };
            for i in 0..target_count.min(choice.options().len()) {
                let _ = choice.select(i);
            }
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }
}

// ---------------------------------------------------------------------------
// 3. Storm Navigation Mini-Game Simulation
// ---------------------------------------------------------------------------

/// Detailed log and status outcome of the Rift Storm navigation.
#[derive(Clone, Debug, PartialEq)]
pub struct FlightReport {
    /// Whether the skiff successfully crossed the void.
    pub survived: bool,
    /// Remaining structural integrity of the hull.
    pub hull_remaining: u32,
    /// Chronicle log describing the voyage encounters.
    pub chronicle: Vec<String>,
}

/// Simulates braving the Aetherial Rift Storm using the watertight dry domain model.
pub fn navigate_rift_storm(skiff: &SovereignSpireSkiff) -> FlightReport {
    let mut hull = skiff.hull_integrity;
    let mut log = Vec::new();

    log.push(format!(
        "The Sovereign Spire-Skiff '{}' enters the swirling vortex of the Aetherial Rift Storm.",
        skiff.skiff_name
    ));

    // Phase 1: The Aether Surge (Evaluates Engine Catalyst and Tactical Stance)
    log.push(format!(
        "Phase 1: An aether surge tests conduits. Catalyst '{}' channels raw power under stance '{}'.",
        skiff.engine.catalyst_core, skiff.tactical_stance
    ));
    if skiff.engine.catalyst_core == "Sun-Shard" {
        log.push("Sun-Shard harmonizes with solar winds; zero turbulence sustained.".to_string());
    } else {
        hull = hull.saturating_sub(15);
        log.push("Engine strain causes minor feedback (-15 hull).".to_string());
    }

    // Phase 2: The Void Squall (Evaluates Auxiliary Wing)
    log.push("Phase 2: Violent shear winds batter the hull.".to_string());
    if let Some(wing) = &skiff.auxiliary_wing {
        log.push(format!(
            "Outrigger wing '{}' with flap profile '{}' dampens buffeting winds.",
            wing.wing_code, wing.flap_profile
        ));
    } else {
        hull = hull.saturating_sub(25);
        log.push("Without auxiliary stabilization, crosswinds buckle armor plates (-25 hull).".to_string());
    }

    // Phase 3: The Leviathan Ambush (Evaluates Turret Array and Bridge Officers)
    log.push(format!(
        "Phase 3: Void-serpents emerge! {} turrets open fire alongside {} bridge officers.",
        skiff.turrets.len(),
        skiff.officers.len()
    ));

    let turret_firepower = skiff.turrets.len() as u32 * 20;
    let officer_coordination = skiff.officers.len() as u32 * 15;
    let total_defense = turret_firepower + officer_coordination;

    if total_defense >= 60 {
        log.push(format!(
            "Overwhelming coordinated fire (defense score {}) repels the leviathans cleanly!",
            total_defense
        ));
    } else {
        hull = hull.saturating_sub(40);
        log.push("Defenses overwhelmed; beast claws rake the superstructure (-40 hull).".to_string());
    }

    let survived = hull > 0;
    if survived {
        log.push(format!(
            "VICTORY: The skiff emerges onto the calm skies of the Sun-Spire Archipelago with {} hull intact!",
            hull
        ));
    } else {
        log.push("CRITICAL FAILURE: The vessel is consumed by the storm abyss.".to_string());
    }

    FlightReport {
        survived,
        hull_remaining: hull,
        chronicle: log,
    }
}

/// Runs the Chapter 10 capstone trial: builds the composite skiff, walks choices, cures into watertight model,
/// and executes the Rift Storm navigation simulation.
pub fn run() -> FlightReport {
    // 1. Build wet engine
    let wet_engine = AetherEngine::builder()
        .engine_code("AE-9000".to_string())
        .catalyst_core(SingleChoice::new(
            "Engine Catalyst",
            vec!["Sun-Shard".to_string(), "Shadow-Essence".to_string()],
        ))
        .build();

    // 2. Build wet outrigger wing
    let wet_wing = FlightWing::builder()
        .wing_code("WING-AURORA".to_string())
        .flap_profile(SingleChoice::new(
            "Flap Profile",
            vec!["Delta-Aerofoil".to_string(), "Camber-Split".to_string()],
        ))
        .build();

    // 3. Build wet defense turrets
    let t1 = HullTurret::builder()
        .turret_label("Port-Primary".to_string())
        .beam_resonance(SingleChoice::new(
            "Port Beam",
            vec!["Solar-Lance".to_string(), "Void-Cutter".to_string()],
        ))
        .build();

    let t2 = HullTurret::builder()
        .turret_label("Starboard-Primary".to_string())
        .beam_resonance(SingleChoice::new(
            "Starboard Beam",
            vec!["Sonic-Disruptor".to_string(), "Plasma-Arc".to_string()],
        ))
        .build();

    // 4. Build candidate bridge officers for drafting
    let off1 = BridgeOfficer::builder()
        .callsign("Commander Vane".to_string())
        .station_role(SingleChoice::new(
            "Vane Role",
            vec!["Fleet-Tactician".to_string(), "Helmsman".to_string()],
        ))
        .build();

    let off2 = BridgeOfficer::builder()
        .callsign("Navigator Selene".to_string())
        .station_role(SingleChoice::new(
            "Selene Role",
            vec!["Aether-Cartographer".to_string(), "Sensor-Operator".to_string()],
        ))
        .build();

    let off3 = BridgeOfficer::builder()
        .callsign("Engineer Torian".to_string())
        .station_role(SingleChoice::new(
            "Torian Role",
            vec!["Chief-Machinist".to_string(), "Shield-Tuner".to_string()],
        ))
        .build();

    let officer_draft = MultiChoice::new(
        "Draft Bridge Officers",
        vec![off1, off2, off3],
    )
    .with_cardinality(Cardinality::Exact(2));

    // 5. Assemble the Grand Sovereign Spire-Skiff via fluent builder
    let mut wet_skiff = SovereignSpireSkiff::builder()
        .skiff_name("Aethelgard's Hope".to_string())
        .hull_integrity(100u32)
        .tactical_stance(SingleChoice::new(
            "Helm Stance",
            vec!["Full-Shields".to_string(), "Maximum-Velocity".to_string()],
        ))
        .engine(wet_engine)
        .auxiliary_wing(wet_wing)
        .with_turrets(t1)
        .with_turrets(t2)
        .officers(officer_draft)
        .build();

    assert!(!wet_skiff.is_complete());

    // 6. Draft bridge officers 0 and 1
    wet_skiff
        .officers
        .as_choice_mut()
        .unwrap()
        .select(0)
        .unwrap();
    wet_skiff
        .officers
        .as_choice_mut()
        .unwrap()
        .select(1)
        .unwrap();
    wet_skiff
        .officers
        .as_choice_mut()
        .unwrap()
        .confirm()
        .unwrap();

    // 7. Walk all choices using ExpeditionVisitor
    let mut visitor = ExpeditionVisitor;
    wet_skiff.walk_choices(&mut visitor);
    assert!(wet_skiff.is_complete());

    // 8. CURE into Watertight Dry Domain Struct!
    let dry_skiff: SovereignSpireSkiff = wet_skiff.resolve().expect("skiff cures cleanly");
    assert_eq!(dry_skiff.skiff_name, "Aethelgard's Hope");
    assert_eq!(dry_skiff.tactical_stance, "Full-Shields");
    assert_eq!(dry_skiff.engine.catalyst_core, "Sun-Shard");
    assert!(dry_skiff.auxiliary_wing.is_some());
    assert_eq!(dry_skiff.turrets.len(), 2);
    assert_eq!(dry_skiff.officers.len(), 2);

    // 9. Execute flight simulation mini-game
    let report = navigate_rift_storm(&dry_skiff);
    assert!(report.survived);
    assert_eq!(report.hull_remaining, 100); // Perfect flight!

    report
}
