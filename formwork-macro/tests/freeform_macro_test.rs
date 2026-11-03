use formwork::choice::Freeform;
use formwork::error::FormworkError;
use formwork::visitor::{ChoiceVisitor, VisitorFlow};
use formwork::{Formwork, StepResult, formwork};

// ============================================================================
// Model Definitions
// ============================================================================

/// Model combining scalar freeform, optional freeform, and collection freeform slots
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ServerConfig {
    pub id: u64,

    #[slot(freeform)]
    pub hostname: String,

    #[slot(freeform, option)]
    pub admin_email: Option<String>,

    #[slot(freeform, collection)]
    pub tags: Vec<String>,
}

/// Model with BFS traversal
#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct BfsFreeformModel {
    #[slot(freeform)]
    pub first: String,

    #[slot(freeform)]
    pub second: String,

    #[slot(freeform)]
    pub third: String,
}

/// Model with DFS traversal
#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DfsFreeformModel {
    #[slot(freeform)]
    pub first: String,

    #[slot(freeform)]
    pub second: String,

    #[slot(freeform)]
    pub third: String,
}

// ============================================================================
// 1. Scalar, Option, and Collection Freeform Construction & Resolution
// ============================================================================

#[test]
fn test_freeform_macro_scalar_raw_values() {
    let wet = ServerConfig::builder()
        .id(1u64)
        .hostname("srv-prod-01".to_string())
        .build();

    assert!(wet.is_complete());
    let dry = wet.resolve().expect("Should resolve cleanly");
    assert_eq!(
        dry,
        ServerConfig {
            id: 1,
            hostname: "srv-prod-01".to_string(),
            admin_email: None,
            tags: vec![],
        }
    );
}

#[test]
fn test_freeform_macro_scalar_choice_pending_fails_resolve() {
    let wet = ServerConfig::builder()
        .id(2u64)
        .hostname(Freeform::new("Enter primary hostname"))
        .build();

    assert!(!wet.is_complete());
    let err = wet.resolve().unwrap_err();
    assert!(matches!(
        err,
        FormworkError::WetChoice { ref prompt } if prompt == "Enter primary hostname"
    ));
}

#[test]
fn test_freeform_macro_scalar_choice_staged_vs_confirmed() {
    let mut choice = Freeform::new("Enter hostname");
    choice.stage("staged-host".to_string());

    let wet_staged = ServerConfig::builder().id(3u64).hostname(choice).build();
    assert!(!wet_staged.is_complete());
    assert!(wet_staged.resolve().is_err());

    let mut made_choice = Freeform::new("Enter hostname");
    made_choice.set("confirmed-host".to_string());

    let wet_made = ServerConfig::builder()
        .id(3u64)
        .hostname(made_choice)
        .build();
    assert!(wet_made.is_complete());
    let dry = wet_made.resolve().expect("Confirmed choice resolves");
    assert_eq!(dry.hostname, "confirmed-host");
}

#[test]
fn test_freeform_macro_with_default_seeding() {
    let default_choice =
        Freeform::new("Enter hostname").with_default("default-host.internal".to_string());

    let mut wet = ServerConfig::builder()
        .id(4u64)
        .hostname(default_choice)
        .build();

    // Not complete until confirmed
    assert!(!wet.is_complete());
    assert!(wet.clone().resolve().is_err());

    // ConfirmAll confirms seeded default
    wet.confirm_all();
    assert!(wet.is_complete());
    let dry = wet.resolve().expect("Confirmed default resolves");
    assert_eq!(dry.hostname, "default-host.internal");
}

#[test]
fn test_freeform_macro_optional_slot_lifecycle() {
    // 1. None omitted (default)
    let wet_none = ServerConfig::builder()
        .id(10u64)
        .hostname("host".to_string())
        .build();
    assert!(wet_none.is_complete());
    let dry_none = wet_none.resolve().unwrap();
    assert_eq!(dry_none.admin_email, None);

    // 2. Raw value
    let wet_some_raw = ServerConfig::builder()
        .id(11u64)
        .hostname("host".to_string())
        .admin_email("admin@example.com".to_string())
        .build();
    assert!(wet_some_raw.is_complete());
    let dry_some_raw = wet_some_raw.resolve().unwrap();
    assert_eq!(
        dry_some_raw.admin_email,
        Some("admin@example.com".to_string())
    );

    // 3. Pending choice in Option slot
    let wet_opt_pending = ServerConfig::builder()
        .id(12u64)
        .hostname("host".to_string())
        .admin_email(Freeform::new("Enter admin email"))
        .build();
    assert!(!wet_opt_pending.is_complete());
    assert!(matches!(
        wet_opt_pending.resolve(),
        Err(FormworkError::WetChoice { ref prompt }) if prompt == "Enter admin email"
    ));

    // 4. Confirmed choice in Option slot
    let mut opt_choice = Freeform::new("Enter admin email");
    opt_choice.set("root@example.com".to_string());
    let wet_opt_made = ServerConfig::builder()
        .id(13u64)
        .hostname("host".to_string())
        .admin_email(opt_choice)
        .build();
    assert!(wet_opt_made.is_complete());
    let dry_opt_made = wet_opt_made.resolve().unwrap();
    assert_eq!(dry_opt_made.admin_email, Some("root@example.com".to_string()));
}

#[test]
fn test_freeform_macro_collection_slot_lifecycle() {
    // 1. Empty collection by default
    let wet_empty = ServerConfig::builder()
        .id(20u64)
        .hostname("host".to_string())
        .build();
    assert!(wet_empty.is_complete());
    assert_eq!(wet_empty.resolve().unwrap().tags, Vec::<String>::new());

    // 2. Raw values added via mutators
    let wet_raw = ServerConfig::builder()
        .id(21u64)
        .hostname("host".to_string())
        .with_tags("database".to_string())
        .with_tags("production".to_string())
        .build();
    assert!(wet_raw.is_complete());
    let dry_raw = wet_raw.resolve().unwrap();
    assert_eq!(dry_raw.tags, vec!["database", "production"]);

    // 3. Collection with pending choice
    let wet_pending = ServerConfig::builder()
        .id(22u64)
        .hostname("host".to_string())
        .with_tags("database".to_string())
        .with_tags(Freeform::new("Enter tag 2"))
        .build();
    assert!(!wet_pending.is_complete());
    assert!(matches!(
        wet_pending.resolve(),
        Err(FormworkError::WetChoice { ref prompt }) if prompt == "Enter tag 2"
    ));

    // 4. Collection with confirmed choice
    let mut tag_choice = Freeform::new("Enter tag 2");
    tag_choice.set("secure".to_string());
    let wet_confirmed = ServerConfig::builder()
        .id(23u64)
        .hostname("host".to_string())
        .with_tags("database".to_string())
        .with_tags(tag_choice)
        .build();
    assert!(wet_confirmed.is_complete());
    let dry_confirmed = wet_confirmed.resolve().unwrap();
    assert_eq!(dry_confirmed.tags, vec!["database", "secure"]);
}

#[test]
fn test_freeform_macro_cured_domain_from_conversion() {
    let dry = ServerConfig {
        id: 99,
        hostname: "converted.internal".to_string(),
        admin_email: Some("converted@example.com".to_string()),
        tags: vec!["alpha".to_string(), "beta".to_string()],
    };

    // From<Dry> for Formwork
    let wet: ServerConfigFormwork = dry.clone().into();
    assert!(wet.is_complete());
    let resolved_dry = wet.resolve().expect("Should resolve back into dry struct");
    assert_eq!(resolved_dry, dry);
}

// ============================================================================
// 2. Dynamic Visitor Traversal & Inspection
// ============================================================================

struct FreeformFillVisitor {
    visited_prompts: Vec<String>,
    fill_prefix: String,
    single_visits: usize,
    multi_visits: usize,
}

impl FreeformFillVisitor {
    fn new(fill_prefix: &str) -> Self {
        Self {
            visited_prompts: Vec::new(),
            fill_prefix: fill_prefix.to_string(),
            single_visits: 0,
            multi_visits: 0,
        }
    }
}

impl ChoiceVisitor<String> for FreeformFillVisitor {
    fn visit_single(&mut self, _choice: &mut formwork::choice::SingleChoice<String>) -> VisitorFlow {
        self.single_visits += 1;
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut formwork::choice::MultiChoice<String>) -> VisitorFlow {
        self.multi_visits += 1;
        VisitorFlow::Continue
    }

    fn visit_freeform(&mut self, choice: &mut Freeform<String>) -> VisitorFlow {
        self.visited_prompts.push(choice.prompt().to_string());
        choice.set(format!("{}-{}", self.fill_prefix, choice.prompt()));
        VisitorFlow::Continue
    }
}

#[test]
fn test_freeform_dynamic_visitor_traversal_and_resolution() {
    let mut wet = ServerConfig::builder()
        .id(100u64)
        .hostname(Freeform::new("HostnamePrompt"))
        .admin_email(Freeform::new("EmailPrompt"))
        .with_tags(Freeform::new("TagPrompt1"))
        .with_tags(Freeform::new("TagPrompt2"))
        .build();

    assert!(!wet.is_complete());

    let mut visitor = FreeformFillVisitor::new("auto");
    wet.walk_choices(&mut visitor);

    // Verify visit_freeform was called for all 4 slots, while single and multi were untouched
    assert_eq!(visitor.single_visits, 0);
    assert_eq!(visitor.multi_visits, 0);
    assert_eq!(
        visitor.visited_prompts,
        vec!["HostnamePrompt", "EmailPrompt", "TagPrompt1", "TagPrompt2"]
    );

    // Formwork is now complete and can be resolved
    assert!(wet.is_complete());
    let dry = wet.resolve().expect("All freeform slots filled and confirmed");
    assert_eq!(dry.hostname, "auto-HostnamePrompt");
    assert_eq!(dry.admin_email, Some("auto-EmailPrompt".to_string()));
    assert_eq!(dry.tags, vec!["auto-TagPrompt1", "auto-TagPrompt2"]);
}

// ============================================================================
// 3. Stepper & Traversal Flow Control (Continue, Back, Break) in BFS and DFS
// ============================================================================

type FreeformAction = Box<dyn FnMut(&mut Freeform<String>) -> VisitorFlow>;

struct FlowControlVisitor {
    action: FreeformAction,
}

impl FlowControlVisitor {
    fn new(f: impl FnMut(&mut Freeform<String>) -> VisitorFlow + 'static) -> Self {
        Self {
            action: Box::new(f),
        }
    }
}

impl ChoiceVisitor<String> for FlowControlVisitor {
    fn visit_single(&mut self, _choice: &mut formwork::choice::SingleChoice<String>) -> VisitorFlow {
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut formwork::choice::MultiChoice<String>) -> VisitorFlow {
        VisitorFlow::Continue
    }

    fn visit_freeform(&mut self, choice: &mut Freeform<String>) -> VisitorFlow {
        (self.action)(choice)
    }
}

#[test]
fn test_freeform_flow_break_in_dfs() {
    let mut model = DfsFreeformModel::builder()
        .first(Freeform::new("P1"))
        .second(Freeform::new("P2"))
        .third(Freeform::new("P3"))
        .build();

    let visited = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let visited_clone = visited.clone();
    let mut visitor = FlowControlVisitor::new(move |c| {
        visited_clone.borrow_mut().push(c.prompt().to_string());
        if c.prompt() == "P2" {
            VisitorFlow::Break
        } else {
            c.set("val".to_string());
            VisitorFlow::Continue
        }
    });

    model.walk_choices(&mut visitor);

    // Should break at P2, never visiting P3
    assert_eq!(*visited.borrow(), vec!["P1", "P2"]);
    assert!(!model.is_complete());
}

#[test]
fn test_freeform_flow_break_in_bfs() {
    let mut model = BfsFreeformModel::builder()
        .first(Freeform::new("B1"))
        .second(Freeform::new("B2"))
        .third(Freeform::new("B3"))
        .build();

    let visited = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let visited_clone = visited.clone();
    let mut visitor = FlowControlVisitor::new(move |c| {
        visited_clone.borrow_mut().push(c.prompt().to_string());
        if c.prompt() == "B1" {
            VisitorFlow::Break
        } else {
            c.set("val".to_string());
            VisitorFlow::Continue
        }
    });

    model.walk_choices(&mut visitor);

    // Should break at B1, never visiting B2 or B3
    assert_eq!(*visited.borrow(), vec!["B1"]);
    assert!(!model.is_complete());
}

#[test]
fn test_freeform_session_stepper_and_back_rewind() {
    let mut model = DfsFreeformModel::builder()
        .first(Freeform::new("Step 1"))
        .second(Freeform::new("Step 2"))
        .third(Freeform::new("Step 3"))
        .build();

    let mut session = model.session();
    assert_eq!(session.current_step(), 0);

    // Step 0: Fill Step 1
    let mut v0 = FlowControlVisitor::new(|c| {
        c.set("Answer 1".to_string());
        VisitorFlow::Continue
    });
    let res0 = session.step(&mut v0);
    assert_eq!(res0, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Step 1: Request Back!
    let mut v_back = FlowControlVisitor::new(|_c| VisitorFlow::Back);
    let res1 = session.step(&mut v_back);
    assert_eq!(res1, StepResult::Choice(VisitorFlow::Back));
    // Step index rewinds to 0!
    assert_eq!(session.current_step(), 0);

    // Rewound step 0 choice was automatically unmade back to UnmadeWithHistory
    let mut v_inspect = FlowControlVisitor::new(|c| {
        assert_eq!(c.prompt(), "Step 1");
        assert!(!c.is_made());
        assert_eq!(c.value(), Some(&"Answer 1".to_string()));
        c.set("Answer 1 Revised".to_string());
        VisitorFlow::Continue
    });
    let res_revisit = session.step(&mut v_inspect);
    assert_eq!(res_revisit, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);
}

// ============================================================================
// 4. UnmakeAll and ConfirmAll Round-Trips
// ============================================================================

#[test]
fn test_freeform_unmake_all_and_confirm_all_roundtrip() {
    let mut model = DfsFreeformModel::builder()
        .first("Val 1".to_string())
        .second(Freeform::new("P2"))
        .third(Freeform::new("P3"))
        .build();

    // Set remaining choices
    let mut fill = FreeformFillVisitor::new("cured");
    model.walk_choices(&mut fill);
    assert!(model.is_complete());

    // UnmakeAll
    model.unmake_all();
    // Fixed slots remain complete, but Choose slots become unmade
    assert!(!model.is_complete());

    // ConfirmAll restores all unmade choices
    model.confirm_all();
    assert!(model.is_complete());

    let dry = model.resolve().expect("Restored model resolves cleanly");
    assert_eq!(dry.first, "Val 1");
    assert_eq!(dry.second, "cured-P2");
    assert_eq!(dry.third, "cured-P3");
}

// ============================================================================
// 5. Heterogeneous Types & Serde JSON Roundtrips
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HeteroFreeformModel {
    #[slot(freeform)]
    pub name: String,

    #[slot(freeform)]
    pub count: u32,

    #[slot(freeform, option)]
    pub enabled: Option<bool>,

    #[slot(freeform, collection)]
    pub weights: Vec<f64>,
}

#[test]
fn test_freeform_heterogeneous_builder_and_resolution() {
    let wet = HeteroFreeformModel::builder()
        .name("HeteroTest".to_string())
        .count(42u32)
        .enabled(true)
        .with_weights(1.5)
        .with_weights(2.5)
        .build();

    assert!(wet.is_complete());
    let dry = wet.resolve().expect("Heterogeneous model resolves cleanly");
    assert_eq!(
        dry,
        HeteroFreeformModel {
            name: "HeteroTest".to_string(),
            count: 42,
            enabled: Some(true),
            weights: vec![1.5, 2.5],
        }
    );
}

#[test]
fn test_freeform_heterogeneous_unmake_confirm_roundtrip() {
    let mut wet = HeteroFreeformModel::builder()
        .name(Freeform::new("Enter name"))
        .count(Freeform::new("Enter count"))
        .enabled(Freeform::new("Enter enabled"))
        .with_weights(Freeform::new("Weight 1"))
        .build();

    assert!(!wet.is_complete());

    // Stage and confirm individually
    if let formwork::Slot::Choose(ref mut c) = wet.name {
        c.set("ResolvedName".to_string());
    }
    if let formwork::Slot::Choose(ref mut c) = wet.count {
        c.set(999u32);
    }
    if let Some(formwork::Slot::Choose(ref mut c)) = wet.enabled {
        c.set(false);
    }
    if let formwork::Slot::Choose(c) = &mut wet.weights[0] {
        c.set(3.25);
    }

    assert!(wet.is_complete());

    // UnmakeAll
    wet.unmake_all();
    assert!(!wet.is_complete());

    // ConfirmAll
    wet.confirm_all();
    assert!(wet.is_complete());

    let dry = wet.resolve().unwrap();
    assert_eq!(dry.name, "ResolvedName");
    assert_eq!(dry.count, 999);
    assert_eq!(dry.enabled, Some(false));
    assert_eq!(dry.weights, vec![3.25]);
}

#[test]
fn test_freeform_serde_json_roundtrip_wet_model() {
    let wet = HeteroFreeformModel::builder()
        .name(Freeform::new("Name").with_default("Seeded".to_string()))
        .count(Freeform::new("Count"))
        .build();

    let json = serde_json::to_string(&wet).expect("Serialization succeeds");
    let mut deserialized: HeteroFreeformModelFormwork =
        serde_json::from_str(&json).expect("Deserialization succeeds");

    assert!(!deserialized.is_complete());
    // Name had a default seeded: ConfirmAll confirms it
    deserialized.confirm_all();
    // Count was still pending without default, so incomplete
    assert!(!deserialized.is_complete());

    // Set count
    if let formwork::Slot::Choose(ref mut c) = deserialized.count {
        c.set(777);
    }
    assert!(deserialized.is_complete());

    let dry = deserialized.resolve().unwrap();
    assert_eq!(dry.name, "Seeded");
    assert_eq!(dry.count, 777);
    assert_eq!(dry.enabled, None);
    assert_eq!(dry.weights, Vec::<f64>::new());
}
