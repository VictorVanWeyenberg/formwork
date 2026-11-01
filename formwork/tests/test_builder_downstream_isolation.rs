//! Adversarial integration test verifying that `#[formwork(builder)]` operates with zero downstream friction.
//!
//! Specifically, this test ensures:
//! 1. A downstream crate does NOT need `typed-builder` in its `Cargo.toml`.
//! 2. A downstream crate does NOT need `use typed_builder::...` or `use formwork::typed_builder::...`.
//! 3. `::formwork::typed_builder::TypedBuilder` and `crate_module_path = ::formwork::typed_builder`
//!    resolve transparently, even in deeply nested module hierarchies.
//! 4. Both non-mutator structs and collection-mutator structs construct and resolve cleanly.

use formwork::choice::{Cardinality, MultiChoice, SingleChoice};
use formwork::visitor::AutoSelectFirst;
use formwork::{Formwork, FormworkError, formwork};

// ---------------------------------------------------------------------------
// 1. Struct with No Mutators (scalar leaf, option leaf, scalar fixed fields)
// ---------------------------------------------------------------------------

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SimpleServerConfig {
    pub server_name: String,
    pub port: u16,
    #[slot]
    pub environment: String,
    #[slot(option)]
    pub secondary_domain: Option<String>,
}

#[test]
fn test_simple_server_config_builders_wet_and_dry() {
    // A. Wet builder with choices
    let mut wet = SimpleServerConfigFormwork::builder()
        .server_name("Edge-01".to_string())
        .port(8080u16)
        .environment(SingleChoice::new(
            "Select Environment",
            vec!["Production".to_string(), "Staging".to_string()],
        ))
        .secondary_domain(Some(SingleChoice::new(
            "Select Domain",
            vec!["alt.domain.org".to_string()],
        )))
        .build();

    assert!(!wet.is_complete());
    wet.walk_choices(&mut AutoSelectFirst);
    assert!(wet.is_complete());

    let dry = wet.resolve().expect("must resolve to SimpleServerConfig");
    assert_eq!(dry.server_name, "Edge-01");
    assert_eq!(dry.port, 8080u16);
    assert_eq!(dry.environment, "Production");
    assert_eq!(dry.secondary_domain, Some("alt.domain.org".to_string()));

    // B. Direct Dry builder without choices
    let direct_dry: SimpleServerConfig = SimpleServerConfig::builder()
        .server_name("Direct-02".to_string())
        .port(443u16)
        .environment("Development".to_string())
        .secondary_domain(None::<String>)
        .build();

    assert_eq!(direct_dry.server_name, "Direct-02");
    assert_eq!(direct_dry.port, 443u16);
    assert_eq!(direct_dry.environment, "Development");
    assert_eq!(direct_dry.secondary_domain, None);
}

// ---------------------------------------------------------------------------
// 2. Struct with Collection Mutators (collection leaf, pick leaf, fixed Vec)
// ---------------------------------------------------------------------------

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ClusterManifest {
    pub cluster_id: String,
    pub static_endpoints: Vec<String>,
    #[slot(collection)]
    pub worker_nodes: Vec<String>,
    #[slot(pick)]
    pub enabled_plugins: Vec<String>,
}

#[test]
fn test_cluster_manifest_collection_mutators_wet_and_dry() {
    // A. Wet builder exercising collection mutator .with_worker_nodes(...)
    let mut wet = ClusterManifestFormwork::builder()
        .cluster_id("Cluster-Omega".to_string())
        .with_static_endpoints("10.0.0.1".to_string())
        .with_static_endpoints("10.0.0.2".to_string())
        .with_worker_nodes("node-primary".to_string())
        .with_worker_nodes(SingleChoice::new(
            "Select Secondary Node",
            vec!["node-secondary-a".to_string(), "node-secondary-b".to_string()],
        ))
        .enabled_plugins(
            MultiChoice::new(
                "Pick 2 Plugins",
                vec!["Telemetry".to_string(), "Auth".to_string(), "Ingress".to_string()],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )
        .build();

    assert!(!wet.is_complete());
    assert!(matches!(wet.clone().resolve(), Err(FormworkError::WetChoice { .. })));

    wet.walk_choices(&mut AutoSelectFirst);
    assert!(wet.is_complete());

    let dry = wet.resolve().expect("must resolve to ClusterManifest");
    assert_eq!(dry.cluster_id, "Cluster-Omega");
    assert_eq!(dry.static_endpoints, vec!["10.0.0.1", "10.0.0.2"]);
    assert_eq!(dry.worker_nodes, vec!["node-primary", "node-secondary-a"]);
    assert_eq!(dry.enabled_plugins, vec!["Telemetry", "Auth"]);

    // B. Direct Dry builder exercising mutators
    let direct_dry: ClusterManifest = ClusterManifest::builder()
        .cluster_id("Cluster-Dry".to_string())
        .with_static_endpoints("192.168.1.1".to_string())
        .with_worker_nodes("fixed-node-1".to_string())
        .with_worker_nodes("fixed-node-2".to_string())
        .enabled_plugins(vec!["Telemetry".to_string(), "Metrics".to_string()])
        .build();

    assert_eq!(direct_dry.cluster_id, "Cluster-Dry");
    assert_eq!(direct_dry.worker_nodes.len(), 2);
    assert_eq!(direct_dry.enabled_plugins.len(), 2);
}

// ---------------------------------------------------------------------------
// 3. Deeply Nested Submodules (Verifying Path Anchoring `::formwork::...`)
// ---------------------------------------------------------------------------

pub mod outer_namespace {
    pub mod inner_layer {
        pub mod core_subsystem {
            use formwork::choice::SingleChoice;
            use formwork::visitor::AutoSelectFirst;
            use formwork::{Formwork, formwork};

            #[formwork(builder)]
            #[derive(Clone, Debug, PartialEq)]
            pub struct NestedTelemetryProbe {
                pub probe_id: u64,
                #[slot]
                pub frequency_band: String,
                #[slot(collection)]
                pub sensors: Vec<String>,
            }

            pub fn exercise_nested_builder() -> NestedTelemetryProbe {
                let mut wet = NestedTelemetryProbe::builder()
                    .probe_id(42u64)
                    .frequency_band(SingleChoice::new(
                        "Set Band",
                        vec!["Sub-GHz".to_string(), "2.4-GHz".to_string()],
                    ))
                    .with_sensors("Thermal".to_string())
                    .with_sensors("Pressure".to_string())
                    .build();

                wet.walk_choices(&mut AutoSelectFirst);
                wet.resolve().expect("must resolve nested entity")
            }
        }
    }
}

#[test]
fn test_deeply_nested_submodule_builder_resolution() {
    let dry = outer_namespace::inner_layer::core_subsystem::exercise_nested_builder();
    assert_eq!(dry.probe_id, 42);
    assert_eq!(dry.frequency_band, "Sub-GHz");
    assert_eq!(dry.sensors, vec!["Thermal", "Pressure"]);
}

// ---------------------------------------------------------------------------
// 4. Cascading Hierarchical Composition
// ---------------------------------------------------------------------------

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ChildSubunit {
    pub subunit_tag: String,
    #[slot]
    pub operational_mode: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ParentCompoundUnit {
    pub unit_name: String,
    #[slot(cascade)]
    pub primary_subunit: ChildSubunit,
    #[slot(collection, cascade)]
    pub backup_subunits: Vec<ChildSubunit>,
}

#[test]
fn test_cascading_builders_and_piecemeal_composition() {
    let child1 = ChildSubunit::builder()
        .subunit_tag("SUB-01".to_string())
        .operational_mode(SingleChoice::new("Mode", vec!["Active".to_string()]))
        .build();

    let child2 = ChildSubunit::builder()
        .subunit_tag("SUB-02".to_string())
        .operational_mode(SingleChoice::new("Mode", vec!["Standby".to_string()]))
        .build();

    let mut parent = ParentCompoundUnit::builder()
        .unit_name("Main-Frame".to_string())
        .primary_subunit(child1)
        .with_backup_subunits(child2)
        .build();

    assert!(!parent.is_complete());
    parent.walk_choices(&mut AutoSelectFirst);
    assert!(parent.is_complete());

    let dry = parent.resolve().expect("cascading resolution succeeds");
    assert_eq!(dry.unit_name, "Main-Frame");
    assert_eq!(dry.primary_subunit.subunit_tag, "SUB-01");
    assert_eq!(dry.primary_subunit.operational_mode, "Active");
    assert_eq!(dry.backup_subunits.len(), 1);
    assert_eq!(dry.backup_subunits[0].subunit_tag, "SUB-02");
    assert_eq!(dry.backup_subunits[0].operational_mode, "Standby");
}

// ---------------------------------------------------------------------------
// 5. Shadowing Defense (Conflicting local `typed_builder` module name)
// ---------------------------------------------------------------------------

pub mod shadow_defense {
    // Deliberately declare a conflicting local module named `typed_builder`
    // If macro expansion used unanchored `typed_builder::...`, this would collide!
    pub mod typed_builder {
        pub struct Dummy;
    }

    use formwork::choice::SingleChoice;
    use formwork::visitor::AutoSelectFirst;
    use formwork::{Formwork, formwork};

    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq)]
    pub struct ShadowResistantEntity {
        pub name: String,
        #[slot]
        pub slot_val: String,
        #[slot(collection)]
        pub items: Vec<String>,
    }

    pub fn test_shadowing() -> ShadowResistantEntity {
        let mut wet = ShadowResistantEntity::builder()
            .name("Safe".to_string())
            .slot_val(SingleChoice::new("Choice", vec!["V".to_string()]))
            .with_items("item".to_string())
            .build();
        wet.walk_choices(&mut AutoSelectFirst);
        wet.resolve().unwrap()
    }
}

#[test]
fn test_shadow_defense() {
    let dry = shadow_defense::test_shadowing();
    assert_eq!(dry.name, "Safe");
    assert_eq!(dry.slot_val, "V");
    assert_eq!(dry.items, vec!["item"]);
}
