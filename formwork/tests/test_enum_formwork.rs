#![deny(warnings)]

use formwork::choice::{Cardinality, Freeform, MultiChoice, SingleChoice};
use formwork::visitor::{AutoSelectFirst, ChoiceVisitor, VisitorFlow};
use formwork::{Formwork, FormworkError, MultiSlot, Slot, StepResult, formwork};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// Shared Domain Types
// ============================================================================

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Proficiency {
    pub category: String,
    #[slot]
    pub skill: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbilityBonus {
    pub amount: i32,
    #[slot]
    pub ability: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StaticTag {
    pub label: String,
    #[slot(option)]
    pub note: Option<String>,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniqueIdentifier {
    pub id: String,
}

#[formwork(derives(Eq, Hash))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    Proficiency(Proficiency),
    Bonus(AbilityBonus),
    Tag(StaticTag),
    Id(UniqueIdentifier),
}

// ============================================================================
// 1. Core HKD, Lifecycle, Derives Forwarding, Zero-Slot Variant, Module Paths
// ============================================================================

#[test]
fn test_core_hkd_lifecycle_and_derives() {
    // Default type parameter is Dry
    let dry_prof: Proficiency = Proficiency::builder()
        .category("Skill")
        .skill("Athletics")
        .build();
    let dry_effect: Effect = Effect::Proficiency(dry_prof.clone());
    let dry_effect_explicit: Effect<EffectDry> = dry_effect.clone();
    assert_eq!(dry_effect, dry_effect_explicit);

    // Derives forwarded to EffectWet and EffectDry (including Hash + Eq deduplication)
    let mut set = HashSet::new();
    set.insert(EffectDry);
    assert!(set.contains(&EffectDry));

    let mut wet_set = HashSet::new();
    wet_set.insert(EffectWet);
    assert!(wet_set.contains(&EffectWet));

    // Wet alias EffectFormwork = Effect<EffectWet>
    let wet_prof: ProficiencyFormwork = Proficiency::builder()
        .category("Skill")
        .skill(SingleChoice::new(
            "Choose a skill",
            vec!["Athletics".to_string(), "Stealth".to_string()],
        ))
        .build();
    let mut wet_effect: EffectFormwork = Effect::Proficiency(wet_prof);

    assert!(!wet_effect.is_complete());
    let err = wet_effect
        .clone()
        .resolve()
        .expect_err("incomplete wet enum must fail resolution");
    assert!(
        matches!(err, FormworkError::WetChoice { .. }),
        "expected WetChoice error, got {err:?}"
    );

    wet_effect.walk_choices(&mut AutoSelectFirst);
    assert!(wet_effect.is_complete());

    let resolved: Effect = wet_effect.resolve().expect("completed enum must resolve");
    assert_eq!(resolved, Effect::Proficiency(dry_prof));
}

#[test]
fn test_true_zero_slot_inner_formwork_variant() {
    let id_wet: UniqueIdentifierFormwork = UniqueIdentifierFormwork::builder()
        .id("hero-001")
        .build();
    let mut effect_wet: EffectFormwork = id_wet.into();

    // Immediately complete with zero choices
    assert!(effect_wet.is_complete());

    // walk_choices visits 0 choices
    let mut rec = RecordingVisitor::new(0);
    effect_wet.walk_choices(&mut rec);
    assert!(rec.prompts.is_empty());

    // session completes immediately on first step
    let mut sess = effect_wet.session();
    let mut step_v = StepVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(sess.step(&mut step_v), StepResult::Complete);
    drop(sess);

    // serde_json round-trip (wet & dry) must NOT leak "__formwork_mode"
    let wet_json = serde_json::to_string(&effect_wet).unwrap();
    assert!(
        !wet_json.contains("__formwork_mode"),
        "wet JSON must not contain __formwork_mode: {wet_json}"
    );
    assert_eq!(
        serde_json::from_str::<EffectFormwork>(&wet_json).unwrap(),
        effect_wet
    );

    let resolved: Effect = effect_wet.resolve().expect("zero-slot variant resolves cleanly");
    let expected_id: UniqueIdentifier = UniqueIdentifier::builder().id("hero-001").build();
    assert_eq!(resolved, Effect::Id(expected_id.clone()));

    let dry_json = serde_json::to_string(&resolved).unwrap();
    assert!(
        !dry_json.contains("__formwork_mode"),
        "dry JSON must not contain __formwork_mode: {dry_json}"
    );
    assert_eq!(serde_json::from_str::<Effect>(&dry_json).unwrap(), resolved);

    // Full builder ergonomics in parent struct
    let parent_wet: FullMatrixParentFormwork = FullMatrixParentFormwork::builder()
        .reg_dry(expected_id.clone())
        .with_reg_dry_vec(expected_id.clone())
        .reg_wet(expected_id.clone())
        .c_single(UniqueIdentifierFormwork::builder().id("c-wet").build())
        .c_option(Some(expected_id.clone()))
        .with_c_collection(expected_id.clone())
        .c_pick(vec![expected_id.clone()])
        .nc_single(expected_id.clone())
        .nc_option(expected_id.clone())
        .with_nc_collection(expected_id.clone())
        .nc_pick([expected_id.clone()])
        .nc_ff_single(expected_id.clone())
        .nc_ff_option(Some(expected_id.clone()))
        .with_nc_ff_collection(expected_id.clone())
        .build();
    assert!(parent_wet.c_single.as_wet().is_some());
    assert!(parent_wet.c_option.as_ref().unwrap().as_dry().is_some());
    assert!(parent_wet.is_complete());
    let resolved_parent = parent_wet.resolve().unwrap();
    assert_eq!(
        resolved_parent.c_single,
        Effect::Id(UniqueIdentifier::builder().id("c-wet").build())
    );
    assert_eq!(resolved_parent.reg_dry, Effect::Id(expected_id));
}

#[test]
fn test_fixed_only_inner_formwork_variant() {
    let tag_wet: StaticTagFormwork = StaticTagFormwork::builder()
        .label("Darkvision")
        .note(None::<String>)
        .build();
    let effect_wet: EffectFormwork = Effect::Tag(tag_wet);
    assert!(effect_wet.is_complete());

    let resolved: Effect = effect_wet.resolve().expect("fixed-only variant resolves immediately");
    assert_eq!(
        resolved,
        Effect::Tag(StaticTag {
            label: "Darkvision".to_string(),
            note: None,
        })
    );
}

pub mod nested_mod {
    use super::*;

    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ModInner {
        #[slot]
        pub value: String,
    }

    #[formwork]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum SuperQualifiedEnum {
        FromSuper(super::Proficiency),
        FromCrate(crate::AbilityBonus),
        FromSelf(ModInner),
    }
}

#[test]
fn test_module_qualified_inner_paths() {
    let mut e1: nested_mod::SuperQualifiedEnumFormwork = nested_mod::SuperQualifiedEnum::FromSuper(
        Proficiency::builder()
            .category("Tool")
            .skill(SingleChoice::new("Tool", vec!["Smith".to_string()]))
            .build(),
    );
    assert!(!e1.is_complete());
    e1.walk_choices(&mut AutoSelectFirst);
    let r1 = e1.resolve().expect("FromSuper resolves");
    assert_eq!(
        r1,
        nested_mod::SuperQualifiedEnum::FromSuper(Proficiency {
            category: "Tool".to_string(),
            skill: "Smith".to_string(),
        })
    );

    let bonus_dry: AbilityBonus = AbilityBonus::builder()
        .amount(2)
        .ability("STR")
        .build();
    let e2: nested_mod::SuperQualifiedEnumFormwork = bonus_dry.into();
    assert!(e2.is_complete());
    let r2 = e2.resolve().expect("FromCrate resolves");
    assert_eq!(
        r2,
        nested_mod::SuperQualifiedEnum::FromCrate(AbilityBonus {
            amount: 2,
            ability: "STR".to_string(),
        })
    );

    let mut e3: nested_mod::SuperQualifiedEnumFormwork = nested_mod::ModInner::builder()
        .value(SingleChoice::new("Val", vec!["Ok".to_string()]))
        .build()
        .into();
    e3.walk_choices(&mut AutoSelectFirst);
    let r3 = e3.resolve().expect("FromSelf resolves");
    assert_eq!(
        r3,
        nested_mod::SuperQualifiedEnum::FromSelf(nested_mod::ModInner {
            value: "Ok".to_string(),
        })
    );
}

// ============================================================================
// 2. Hygiene (no `use formwork::{Formwork, Walkable};` in scope)
// ============================================================================

mod hygiene_mod {
    #[formwork::formwork]
    #[derive(Clone, Debug, PartialEq)]
    pub struct HygienicInner {
        #[slot]
        pub item: String,
    }

    #[formwork::formwork]
    #[derive(Clone, Debug, PartialEq)]
    pub enum HygienicEnum {
        One(HygienicInner),
    }
}

#[test]
fn test_hygiene_without_trait_imports() {
    let inner = hygiene_mod::HygienicInnerFormwork {
        item: Slot::FixedWet("Clean".to_string()),
    };
    let e: hygiene_mod::HygienicEnumFormwork = inner.into();
    assert!(e.is_complete());
    let dry = e.resolve().expect("hygienic enum resolves");
    assert_eq!(
        dry,
        hygiene_mod::HygienicEnum::One(hygiene_mod::HygienicInner {
            item: "Clean".to_string(),
        })
    );
}

// ============================================================================
// 3. Visibility (private, pub(crate), pub(super) wrapping private inner types)
// ============================================================================

mod visibility_mod {
    use super::*;

    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq)]
    struct PrivateInnerA {
        #[slot]
        x: i32,
    }

    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq)]
    struct PrivateInnerB {
        #[slot]
        y: String,
    }

    #[formwork]
    #[derive(Clone, Debug, PartialEq)]
    enum PrivateEnum {
        A(PrivateInnerA),
        B(PrivateInnerB),
    }

    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) struct CrateInner {
        #[slot]
        pub(crate) z: i32,
    }

    #[formwork]
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) enum CrateEnum {
        C(CrateInner),
    }

    pub mod child {
        use super::*;

        #[formwork(builder)]
        #[derive(Clone, Debug, PartialEq)]
        pub(super) struct SuperInner {
            #[slot]
            pub(super) w: i32,
        }

        #[formwork]
        #[derive(Clone, Debug, PartialEq)]
        pub(super) enum SuperEnum {
            S(SuperInner),
        }
    }

    pub fn exercise_all() {
        let p: PrivateEnumFormwork = PrivateInnerAFormwork::builder().x(1).build().into();
        let p_dry: PrivateEnum = p.resolve().unwrap();
        assert_eq!(p_dry, PrivateEnum::A(PrivateInnerA { x: 1 }));

        let s: child::SuperEnumFormwork =
            child::SuperInnerFormwork::builder().w(7).build().into();
        let s_dry: child::SuperEnum = s.resolve().unwrap();
        assert_eq!(s_dry, child::SuperEnum::S(child::SuperInner { w: 7 }));
    }
}

#[test]
fn test_visibility_levels() {
    visibility_mod::exercise_all();
    let c: visibility_mod::CrateEnumFormwork = visibility_mod::CrateInnerFormwork::builder()
        .z(42)
        .build()
        .into();
    let c_dry: visibility_mod::CrateEnum = c.resolve().unwrap();
    assert_eq!(
        c_dry,
        visibility_mod::CrateEnum::C(visibility_mod::CrateInner { z: 42 })
    );
}

// ============================================================================
// 4. Serde & Attributes (all tagging modes, repr(u8)+discriminants, cfg)
// ============================================================================

#[formwork]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InternallyTaggedEffect {
    Proficiency(Proficiency),
    Bonus(AbilityBonus),
}

#[formwork]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AdjacentlyTaggedEffect {
    Proficiency(Proficiency),
    Bonus(AbilityBonus),
}

#[formwork]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UntaggedEffect {
    Proficiency(Proficiency),
    Bonus(AbilityBonus),
}

#[formwork]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenamedEffect {
    #[serde(rename = "grant_prof")]
    ProficiencyGrant(Proficiency),
    AbilityScoreBonus(AbilityBonus),
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SerdeContainer {
    #[slot(cascade)]
    pub single: Effect,
    #[slot(option, cascade)]
    pub optional: Option<Effect>,
    #[slot(collection, cascade)]
    pub list: Vec<Effect>,
    #[slot(pick, cascade)]
    pub picked: Vec<Effect>,
}

#[test]
fn test_serde_roundtrip_all_tagging_modes_dry_and_wet() {
    let prof_dry: Proficiency = Proficiency::builder()
        .category("Skill")
        .skill("Perception")
        .build();
    let prof_wet = Proficiency::builder()
        .category("Skill")
        .skill(SingleChoice::new(
            "Skill",
            vec!["Perception".to_string(), "Insight".to_string()],
        ))
        .build();

    // 1. External tagging (default)
    let ext_dry = Effect::Proficiency(prof_dry.clone());
    let ext_dry_json = serde_json::to_string(&ext_dry).unwrap();
    assert_eq!(serde_json::from_str::<Effect>(&ext_dry_json).unwrap(), ext_dry);

    let ext_wet: EffectFormwork = Effect::Proficiency(prof_wet.clone());
    let ext_wet_json = serde_json::to_string(&ext_wet).unwrap();
    assert_eq!(
        serde_json::from_str::<EffectFormwork>(&ext_wet_json).unwrap(),
        ext_wet
    );

    // 2. Internally tagged
    let int_dry = InternallyTaggedEffect::Proficiency(prof_dry.clone());
    let int_dry_json = serde_json::to_string(&int_dry).unwrap();
    assert!(int_dry_json.contains(r#""type":"Proficiency""#));
    assert_eq!(
        serde_json::from_str::<InternallyTaggedEffect>(&int_dry_json).unwrap(),
        int_dry
    );

    let int_wet: InternallyTaggedEffectFormwork =
        InternallyTaggedEffect::Proficiency(prof_wet.clone());
    let int_wet_json = serde_json::to_string(&int_wet).unwrap();
    assert!(int_wet_json.contains(r#""type":"Proficiency""#));
    assert_eq!(
        serde_json::from_str::<InternallyTaggedEffectFormwork>(&int_wet_json).unwrap(),
        int_wet
    );

    // 3. Adjacently tagged
    let adj_dry = AdjacentlyTaggedEffect::Proficiency(prof_dry.clone());
    let adj_dry_json = serde_json::to_string(&adj_dry).unwrap();
    assert!(adj_dry_json.contains(r#""type":"Proficiency""#));
    assert!(adj_dry_json.contains(r#""data":"#));
    assert_eq!(
        serde_json::from_str::<AdjacentlyTaggedEffect>(&adj_dry_json).unwrap(),
        adj_dry
    );

    let adj_wet: AdjacentlyTaggedEffectFormwork =
        AdjacentlyTaggedEffect::Proficiency(prof_wet.clone());
    let adj_wet_json = serde_json::to_string(&adj_wet).unwrap();
    assert_eq!(
        serde_json::from_str::<AdjacentlyTaggedEffectFormwork>(&adj_wet_json).unwrap(),
        adj_wet
    );

    // 4. Untagged
    let unt_dry = UntaggedEffect::Proficiency(prof_dry.clone());
    let unt_dry_json = serde_json::to_string(&unt_dry).unwrap();
    assert_eq!(
        serde_json::from_str::<UntaggedEffect>(&unt_dry_json).unwrap(),
        unt_dry
    );

    let unt_wet: UntaggedEffectFormwork = UntaggedEffect::Proficiency(prof_wet.clone());
    let unt_wet_json = serde_json::to_string(&unt_wet).unwrap();
    assert_eq!(
        serde_json::from_str::<UntaggedEffectFormwork>(&unt_wet_json).unwrap(),
        unt_wet
    );

    // 5. rename_all + variant rename
    let ren_a = RenamedEffect::ProficiencyGrant(prof_dry.clone());
    let ren_a_json = serde_json::to_string(&ren_a).unwrap();
    assert!(ren_a_json.contains(r#""grant_prof""#));
    assert_eq!(
        serde_json::from_str::<RenamedEffect>(&ren_a_json).unwrap(),
        ren_a
    );

    let ren_b: RenamedEffectFormwork = RenamedEffect::AbilityScoreBonus(
        AbilityBonusFormwork::builder()
            .amount(1)
            .ability("DEX".to_string())
            .build(),
    );
    let ren_b_json = serde_json::to_string(&ren_b).unwrap();
    assert!(ren_b_json.contains(r#""ability_score_bonus""#));
    assert_eq!(
        serde_json::from_str::<RenamedEffectFormwork>(&ren_b_json).unwrap(),
        ren_b
    );

    // 6. Nested inside parent cascading slots
    let container_wet: SerdeContainerFormwork = SerdeContainerFormwork::builder()
        .single(prof_wet.clone())
        .optional(
            AbilityBonusFormwork::builder()
                .amount(2)
                .ability("STR".to_string())
                .build(),
        )
        .with_list(prof_wet.clone())
        .picked(vec![
            StaticTagFormwork::builder()
                .label("Keen")
                .note(None::<String>)
                .build(),
        ])
        .build();
    let cw_json = serde_json::to_string(&container_wet).unwrap();
    assert_eq!(
        serde_json::from_str::<SerdeContainerFormwork>(&cw_json).unwrap(),
        container_wet
    );

    // Dry container round-trip
    let container_dry: SerdeContainer = SerdeContainer::builder()
        .single(prof_dry.clone())
        .optional(AbilityBonus::builder().amount(2).ability("STR").build())
        .with_list(prof_dry.clone())
        .picked(vec![
            StaticTag::builder()
                .label("Keen")
                .note(None::<String>)
                .build(),
        ])
        .build();
    let cd_json = serde_json::to_string(&container_dry).unwrap();
    assert_eq!(
        serde_json::from_str::<SerdeContainer>(&cd_json).unwrap(),
        container_dry
    );
}

#[formwork]
#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum ReprDiscriminantEnum {
    Prof(Proficiency) = 1,
    Bonus(AbilityBonus) = 2,
}

#[test]
fn test_repr_u8_and_explicit_discriminants() {
    let e = ReprDiscriminantEnum::Prof(Proficiency {
        category: "Skill".to_string(),
        skill: "Arcana".to_string(),
    });
    // Verify discriminant value via pointer cast on repr(u8) enum
    let disc = unsafe { *(&e as *const ReprDiscriminantEnum as *const u8) };
    assert_eq!(disc, 1);

    let e2: ReprDiscriminantEnumFormwork = AbilityBonusFormwork::builder()
        .amount(1)
        .ability("INT".to_string())
        .build()
        .into();
    let disc2 = unsafe { *(&e2 as *const ReprDiscriminantEnumFormwork as *const u8) };
    assert_eq!(disc2, 2);
    assert!(e2.resolve().is_ok());
}

#[cfg(any())]
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct DisabledInner {
    #[slot]
    pub nope: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomUnvisited(pub u8);

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct UnvisitedSlotInner {
    #[slot]
    pub marker: CustomUnvisited,
}

#[formwork]
#[derive(Clone, Debug, PartialEq)]
pub enum CfgVariantEnum {
    #[cfg(test)]
    Enabled(Proficiency),
    #[cfg(any())]
    DisabledExisting(AbilityBonus),
    #[cfg(any())]
    DisabledNonExistent(DisabledInner),
    #[cfg(any())]
    DisabledUnvisited(UnvisitedSlotInner),
}

struct StringOnlyVisitor;

impl ChoiceVisitor<String> for StringOnlyVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
        let _ = choice.select(0);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<String>) -> VisitorFlow {
        let _ = choice.select(0);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

#[test]
fn test_cfg_variants() {
    let mut e: CfgVariantEnumFormwork = Proficiency::builder()
        .category("Skill")
        .skill(SingleChoice::new("S", vec!["History".to_string()]))
        .build()
        .into();
    // StringOnlyVisitor does NOT implement ChoiceVisitor<CustomUnvisited>.
    // This proves disabled #[cfg(any())] variants impose zero spurious ChoiceVisitor bounds!
    e.walk_choices(&mut StringOnlyVisitor);
    let dry = e.resolve().unwrap();
    assert_eq!(
        dry,
        CfgVariantEnum::Enabled(Proficiency {
            category: "Skill".to_string(),
            skill: "History".to_string(),
        })
    );
}

// ============================================================================
// 5. Traversal Transparency (direct, session, 2-level nested enum BFS & DFS, Back)
// ============================================================================

struct RecordingVisitor {
    prompts: Vec<String>,
    select_idx: usize,
}

impl RecordingVisitor {
    fn new(select_idx: usize) -> Self {
        Self {
            prompts: Vec::new(),
            select_idx,
        }
    }
}

impl<T> ChoiceVisitor<T> for RecordingVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        self.prompts.push(choice.prompt().to_string());
        let _ = choice.select(self.select_idx);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        self.prompts.push(choice.prompt().to_string());
        let _ = choice.select(self.select_idx);
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

struct StepVisitor {
    select_idx: usize,
    flow: VisitorFlow,
    seen_prompt: String,
}

impl StepVisitor {
    fn new(select_idx: usize, flow: VisitorFlow) -> Self {
        Self {
            select_idx,
            flow,
            seen_prompt: String::new(),
        }
    }
}

impl<T> ChoiceVisitor<T> for StepVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        self.seen_prompt = choice.prompt().to_string();
        if self.flow != VisitorFlow::Back {
            let _ = choice.select(self.select_idx);
            let _ = choice.confirm();
        }
        self.flow
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        self.seen_prompt = choice.prompt().to_string();
        if self.flow != VisitorFlow::Back {
            let _ = choice.select(self.select_idx);
            let _ = choice.confirm();
        }
        self.flow
    }
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct TwoStepInner {
    #[slot]
    pub first: String,
    #[slot]
    pub second: String,
}

#[formwork]
#[derive(Clone, Debug, PartialEq)]
pub enum InnerEnum {
    Prof(Proficiency),
    TwoStep(TwoStepInner),
}

#[formwork]
#[derive(Clone, Debug, PartialEq)]
pub enum OuterEnum {
    Inner(InnerEnum),
    Bonus(AbilityBonus),
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct BfsParent {
    #[slot]
    pub root_slot: String,
    #[slot(cascade)]
    pub enum_child: OuterEnum,
    #[slot(cascade)]
    pub struct_sibling: TwoStepInner,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DfsParent {
    #[slot]
    pub root_slot: String,
    #[slot(cascade)]
    pub enum_child: OuterEnum,
    #[slot(cascade)]
    pub struct_sibling: TwoStepInner,
}

#[test]
fn test_direct_root_enum_walk_session_unmake_confirm() {
    let two_step = TwoStepInner::builder()
        .first(SingleChoice::new("E-First", vec!["A1".to_string(), "A2".to_string()]))
        .second(SingleChoice::new("E-Second", vec!["B1".to_string(), "B2".to_string()]))
        .build();
    let mut root: OuterEnumFormwork = OuterEnum::Inner(InnerEnum::TwoStep(two_step));

    // Session: Continue, Back, AtStart, Break, Continue, Complete
    let mut session = root.session();

    // AtStart when going Back at step 0
    let mut v_back0 = StepVisitor::new(0, VisitorFlow::Back);
    assert_eq!(session.step(&mut v_back0), StepResult::AtStart);
    assert_eq!(v_back0.seen_prompt, "E-First");

    // Step 0 -> Continue (select index 0 = "A1")
    let mut v0 = StepVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(
        session.step(&mut v0),
        StepResult::Choice(VisitorFlow::Continue)
    );
    assert_eq!(v0.seen_prompt, "E-First");

    // Step 1 -> Back (unmakes step 0 and rewinds)
    let mut v1_back = StepVisitor::new(0, VisitorFlow::Back);
    assert_eq!(
        session.step(&mut v1_back),
        StepResult::Choice(VisitorFlow::Back)
    );
    assert_eq!(v1_back.seen_prompt, "E-Second");

    // Step 0 again -> Break (select index 1 = "A2" and pause)
    let mut v0_break = StepVisitor::new(1, VisitorFlow::Break);
    assert_eq!(
        session.step(&mut v0_break),
        StepResult::Choice(VisitorFlow::Break)
    );
    assert_eq!(v0_break.seen_prompt, "E-First");

    // Step 0 -> Continue (confirm index 1 = "A2")
    let mut v0_again = StepVisitor::new(1, VisitorFlow::Continue);
    assert_eq!(
        session.step(&mut v0_again),
        StepResult::Choice(VisitorFlow::Continue)
    );

    // Step 1 -> Continue (select index 1 = "B2")
    let mut v1 = StepVisitor::new(1, VisitorFlow::Continue);
    assert_eq!(
        session.step(&mut v1),
        StepResult::Choice(VisitorFlow::Continue)
    );
    assert_eq!(v1.seen_prompt, "E-Second");

    // Complete
    let mut v_done = StepVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(session.step(&mut v_done), StepResult::Complete);
    drop(session);

    assert!(root.is_complete());

    // unmake_all & confirm_all on root enum
    root.unmake_all();
    assert!(!root.is_complete());
    root.confirm_all();
    assert!(root.is_complete());

    let dry = root.resolve().unwrap();
    assert_eq!(
        dry,
        OuterEnum::Inner(InnerEnum::TwoStep(TwoStepInner {
            first: "A2".to_string(),
            second: "B2".to_string(),
        }))
    );
}

#[test]
fn test_bfs_transparency_exact_level_interleaving_and_back_across_enum() {
    let make_bfs = || {
        BfsParent::builder()
            .root_slot(SingleChoice::new("Root", vec!["R0".to_string(), "R1".to_string()]))
            .enum_child(OuterEnumFormwork::Inner(InnerEnumFormwork::TwoStep(
                TwoStepInnerFormwork::builder()
                    .first(SingleChoice::new(
                        "EnumChild-1",
                        vec!["EC1-0".to_string(), "EC1-1".to_string()],
                    ))
                    .second(SingleChoice::new(
                        "EnumChild-2",
                        vec!["EC2-0".to_string(), "EC2-1".to_string()],
                    ))
                    .build(),
            )))
            .struct_sibling(
                TwoStepInner::builder()
                    .first(SingleChoice::new(
                        "Sibling-1",
                        vec!["S1-0".to_string(), "S1-1".to_string()],
                    ))
                    .second(SingleChoice::new(
                        "Sibling-2",
                        vec!["S2-0".to_string(), "S2-1".to_string()],
                    ))
                    .build(),
            )
            .build()
    };

    // In BFS, OuterEnum -> InnerEnum must be completely transparent (depth unchanged!).
    // Depth 0: Root
    // Depth 1: EnumChild-1, EnumChild-2, Sibling-1, Sibling-2
    // Notice both EnumChild (through 2 enums!) and Sibling are at Depth 1!
    let mut bfs = make_bfs();
    let mut rec = RecordingVisitor::new(0);
    bfs.walk_choices(&mut rec);
    assert_eq!(
        rec.prompts,
        vec!["Root", "EnumChild-1", "EnumChild-2", "Sibling-1", "Sibling-2"]
    );

    // Now test VisitorFlow::Back across the enum boundary (from Sibling-1 back to EnumChild-2,
    // and from EnumChild-1 back to Root).
    let mut bfs2 = make_bfs();
    let mut sess = bfs2.session();

    // 0: Root -> Continue (0)
    let mut v = StepVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(v.seen_prompt, "Root");

    // 1: EnumChild-1 -> Back (re-opens Root!)
    let mut v = StepVisitor::new(0, VisitorFlow::Back);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(v.seen_prompt, "EnumChild-1");

    // 0: Root -> Continue (1)
    let mut v = StepVisitor::new(1, VisitorFlow::Continue);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(v.seen_prompt, "Root");

    // 1: EnumChild-1 -> Continue (0)
    let mut v = StepVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(v.seen_prompt, "EnumChild-1");

    // 2: EnumChild-2 -> Continue (0)
    let mut v = StepVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(v.seen_prompt, "EnumChild-2");

    // 3: Sibling-1 -> Back (re-opens EnumChild-2 inside 2-level nested enum!)
    let mut v = StepVisitor::new(0, VisitorFlow::Back);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(v.seen_prompt, "Sibling-1");

    // 2: EnumChild-2 -> Continue (1)
    let mut v = StepVisitor::new(1, VisitorFlow::Continue);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(v.seen_prompt, "EnumChild-2");

    // 3: Sibling-1 -> Continue (0), 4: Sibling-2 -> Continue (0)
    let mut v = StepVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(v.seen_prompt, "Sibling-1");
    let mut v = StepVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(sess.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(v.seen_prompt, "Sibling-2");
    assert_eq!(sess.step(&mut v), StepResult::Complete);
    drop(sess);

    let dry = bfs2.resolve().unwrap();
    assert_eq!(dry.root_slot, "R1");
    assert_eq!(
        dry.enum_child,
        OuterEnum::Inner(InnerEnum::TwoStep(TwoStepInner {
            first: "EC1-0".to_string(),
            second: "EC2-1".to_string(),
        }))
    );
}

#[test]
fn test_dfs_parent_with_nested_enum_and_cascading_choices() {
    let mut dfs = DfsParent::builder()
        .root_slot(SingleChoice::new("Root", vec!["R".to_string()]))
        .enum_child(SingleChoice::new(
            "Choose Outer",
            vec![
                OuterEnumFormwork::Inner(InnerEnumFormwork::Prof(
                    ProficiencyFormwork::builder()
                        .category("Skill")
                        .skill(SingleChoice::new("InnerSkill", vec!["Stealth".to_string()]))
                        .build(),
                )),
                OuterEnumFormwork::Bonus(
                    AbilityBonusFormwork::builder()
                        .amount(2)
                        .ability(SingleChoice::new("InnerAbility", vec!["DEX".to_string()]))
                        .build(),
                ),
            ],
        ))
        .struct_sibling(
            TwoStepInner::builder()
                .first(SingleChoice::new("Sib-1", vec!["A".to_string()]))
                .second(SingleChoice::new("Sib-2", vec!["B".to_string()]))
                .build(),
        )
        .build();

    let mut rec = RecordingVisitor::new(0);
    dfs.walk_choices(&mut rec);
    // DFS: Root -> Choose Outer -> InnerSkill (selected option 0) -> Sib-1 -> Sib-2
    assert_eq!(
        rec.prompts,
        vec!["Root", "Choose Outer", "InnerSkill", "Sib-1", "Sib-2"]
    );

    let dry = dfs.resolve().unwrap();
    assert_eq!(
        dry.enum_child,
        OuterEnum::Inner(InnerEnum::Prof(Proficiency {
            category: "Skill".to_string(),
            skill: "Stealth".to_string(),
        }))
    );
}

// Also test deeper BFS level interleaving: a struct inside a 2-level enum that itself
// has a cascading child struct, vs a sibling struct with a cascading child struct!
#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepChild {
    #[slot]
    pub top: String,
    #[slot(cascade)]
    pub leaf: Proficiency,
}

#[formwork]
#[derive(Clone, Debug, PartialEq)]
pub enum DeepEnum {
    Deep(DeepChild),
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepBfsRoot {
    #[slot(cascade)]
    pub via_enum: DeepEnum,
    #[slot(cascade)]
    pub direct: DeepChild,
}

#[test]
fn test_bfs_deep_interleaving_proves_zero_extra_depth_for_enum() {
    let mut root = DeepBfsRoot::builder()
        .via_enum(DeepChild::builder()
            .top(SingleChoice::new("Enum-Top", vec!["ET".to_string()]))
            .leaf(
                Proficiency::builder()
                    .category("C")
                    .skill(SingleChoice::new("Enum-Leaf", vec!["EL".to_string()]))
                    .build(),
            )
            .build())
        .direct(
            DeepChild::builder()
                .top(SingleChoice::new("Direct-Top", vec!["DT".to_string()]))
                .leaf(
                    Proficiency::builder()
                        .category("C")
                        .skill(SingleChoice::new("Direct-Leaf", vec!["DL".to_string()]))
                        .build(),
                )
                .build(),
        )
        .build();

    let mut rec = RecordingVisitor::new(0);
    root.walk_choices(&mut rec);
    // If DeepEnum added +1 depth, Direct-Top would appear BEFORE Enum-Top, or Enum-Leaf would be
    // at depth 3 instead of depth 2!
    // Because DeepEnum is transparent (depth unchanged):
    // Depth 1: Enum-Top, Direct-Top
    // Depth 2: Enum-Leaf, Direct-Leaf
    assert_eq!(
        rec.prompts,
        vec!["Enum-Top", "Direct-Top", "Enum-Leaf", "Direct-Leaf"]
    );
}

// ============================================================================
// 6. Full `From` & Higher-Up Builder Ergonomics Matrix
// ============================================================================

#[test]
fn test_direct_from_conversions() {
    let prof_dry: Proficiency = Proficiency::builder()
        .category("Skill")
        .skill("Athletics")
        .build();
    let prof_wet: ProficiencyFormwork = ProficiencyFormwork::builder()
        .category("Skill")
        .skill("Athletics".to_string())
        .build();

    // InnerDry -> EnumDry
    let e_dry: Effect = prof_dry.clone().into();
    assert_eq!(e_dry, Effect::Proficiency(prof_dry.clone()));

    // InnerWet -> EnumWet
    let e_wet_from_wet: EffectFormwork = prof_wet.clone().into();
    assert_eq!(e_wet_from_wet, EffectFormwork::Proficiency(prof_wet));

    // InnerDry -> EnumWet
    let e_wet_from_inner_dry: EffectFormwork = prof_dry.clone().into();
    assert!(e_wet_from_inner_dry.is_complete());
    assert_eq!(e_wet_from_inner_dry.resolve().unwrap(), e_dry);

    // EnumDry -> EnumWet
    let e_wet_from_enum_dry: EffectFormwork = e_dry.clone().into();
    assert!(e_wet_from_enum_dry.is_complete());
    assert_eq!(e_wet_from_enum_dry.resolve().unwrap(), e_dry);
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct FullMatrixParent {
    // Regular fields
    pub reg_dry: Effect,
    pub reg_dry_vec: Vec<Effect>,
    pub reg_wet: EffectFormwork,

    // 4 cascading slots
    #[slot(cascade)]
    pub c_single: Effect,
    #[slot(option, cascade)]
    pub c_option: Option<Effect>,
    #[slot(collection, cascade)]
    pub c_collection: Vec<Effect>,
    #[slot(pick, cascade)]
    pub c_pick: Vec<Effect>,

    // 7 non-cascading slots holding dry Effect
    #[slot]
    pub nc_single: Effect,
    #[slot(option)]
    pub nc_option: Option<Effect>,
    #[slot(collection)]
    pub nc_collection: Vec<Effect>,
    #[slot(pick)]
    pub nc_pick: Vec<Effect>,
    #[slot(freeform)]
    pub nc_ff_single: Effect,
    #[slot(freeform, option)]
    pub nc_ff_option: Option<Effect>,
    #[slot(freeform, collection)]
    pub nc_ff_collection: Vec<Effect>,
}

#[test]
fn test_builder_ergonomics_wet_mode_and_fixed_wet_vs_dry_semantics() {
    let prof_dry: Proficiency = Proficiency::builder()
        .category("Skill")
        .skill("Athletics")
        .build();
    let prof_wet: ProficiencyFormwork = ProficiencyFormwork::builder()
        .category("Skill")
        .skill("Stealth".to_string())
        .build();
    let bonus_dry: AbilityBonus = AbilityBonus::builder()
        .amount(2)
        .ability("STR")
        .build();
    let bonus_wet: AbilityBonusFormwork = AbilityBonusFormwork::builder()
        .amount(1)
        .ability("DEX".to_string())
        .build();
    let tag_dry: StaticTag = StaticTag::builder()
        .label("Alert")
        .note(None::<String>)
        .build();

    // Case A: Passing WET inner types into cascading slots -> Slot::FixedWet
    let wet_parent_a: FullMatrixParentFormwork = FullMatrixParentFormwork::builder()
        .reg_dry(prof_dry.clone())
        .with_reg_dry_vec(bonus_dry.clone())
        .with_reg_dry_vec(tag_dry.clone())
        .reg_wet(prof_wet.clone())
        // 4 cascading slots with WET inner types
        .c_single(prof_wet.clone())
        .c_option(bonus_wet.clone())
        .with_c_collection(prof_wet.clone())
        .with_c_collection(bonus_dry.clone())
        .c_pick(vec![prof_wet.clone()])
        // 7 non-cascading slots with DRY inner types
        .nc_single(prof_dry.clone())
        .nc_option(bonus_dry.clone())
        .with_nc_collection(prof_dry.clone())
        .with_nc_collection(tag_dry.clone())
        .nc_pick([prof_dry.clone(), prof_dry.clone()])
        .nc_ff_single(tag_dry.clone())
        .nc_ff_option(Some(prof_dry.clone()))
        .with_nc_ff_collection(bonus_dry.clone())
        .build();

    // Verify Slot::FixedWet vs Slot::FixedDry preservation!
    assert!(wet_parent_a.c_single.as_wet().is_some());
    assert!(wet_parent_a.c_single.as_dry().is_none());

    assert!(wet_parent_a.c_option.as_ref().unwrap().as_wet().is_some());
    assert!(wet_parent_a.c_option.as_ref().unwrap().as_dry().is_none());

    assert!(wet_parent_a.c_collection[0].as_wet().is_some());
    assert!(wet_parent_a.c_collection[1].as_dry().is_some());

    assert!(wet_parent_a.c_pick.as_wet().is_some());
    assert!(wet_parent_a.c_pick.as_dry().is_none());

    assert!(wet_parent_a.is_complete());
    let resolved_a = wet_parent_a.resolve().unwrap();
    assert_eq!(resolved_a.c_single, Effect::Proficiency(Proficiency {
        category: "Skill".to_string(),
        skill: "Stealth".to_string(),
    }));
    assert_eq!(resolved_a.c_collection.len(), 2);

    // Case B: Passing DRY inner types into cascading slots -> Slot::FixedDry
    let wet_parent_b: FullMatrixParentFormwork = FullMatrixParentFormwork::builder()
        .reg_dry(Effect::Tag(tag_dry.clone()))
        .reg_wet(prof_dry.clone())
        // 4 cascading slots with DRY inner types (and Option<InnerDry> + [InnerDry; N])
        .c_single(prof_dry.clone())
        .c_option(Some(bonus_dry.clone()))
        .c_collection([prof_dry.clone()])
        .c_pick([prof_dry.clone(), prof_dry.clone()])
        // 7 non-cascading slots with Choice/Freeform/Slot/None variants
        .nc_single(SingleChoice::new("NC", vec![Effect::from(prof_dry.clone()), Effect::from(bonus_dry.clone())]))
        .nc_option(None::<Proficiency>)
        .nc_collection(vec![Slot::FixedWet(Effect::Tag(tag_dry.clone()))])
        .nc_pick(
            MultiChoice::new(
                "NCPick",
                vec![Effect::from(prof_dry.clone()), Effect::from(bonus_dry.clone())],
            )
            .with_cardinality(Cardinality::Exact(1)),
        )
        .nc_ff_single(Freeform::new("FF").with_default(Effect::from(tag_dry.clone())))
        .nc_ff_option(None::<Effect>)
        .nc_ff_collection(vec![Freeform::new("FFC").with_default(Effect::from(bonus_dry.clone()))])
        .build();

    assert!(wet_parent_b.c_single.as_dry().is_some());
    assert!(wet_parent_b.c_single.as_wet().is_none());

    assert!(wet_parent_b.c_option.as_ref().unwrap().as_dry().is_some());
    assert!(wet_parent_b.c_option.as_ref().unwrap().as_wet().is_none());

    assert!(wet_parent_b.c_collection[0].as_dry().is_some());
    assert!(wet_parent_b.c_pick.as_dry().is_some());
    assert!(wet_parent_b.c_pick.as_wet().is_none());

    let mut wet_parent_b = wet_parent_b;
    wet_parent_b.walk_choices(&mut AutoSelectFirst);
    assert!(wet_parent_b.is_complete());
    let resolved_b = wet_parent_b.resolve().unwrap();
    assert_eq!(resolved_b.nc_single, Effect::Proficiency(prof_dry.clone()));
    assert_eq!(resolved_b.nc_option, None);
    assert_eq!(resolved_b.nc_pick, vec![Effect::Proficiency(prof_dry.clone())]);

    // Case C: SingleChoice::new / MultiChoice::new with homogeneous InnerFormwork / Inner
    // and heterogeneous variants, plus MultiSlot directly
    let homogeneous_wet_choice: SingleChoice<EffectFormwork> =
        SingleChoice::new("HomogWet", vec![prof_wet.clone()]);
    let heterogeneous_wet_choice: SingleChoice<EffectFormwork> = SingleChoice::new(
        "HeteroWet",
        vec![EffectFormwork::from(prof_wet.clone()), EffectFormwork::from(bonus_wet.clone())],
    );
    let homogeneous_dry_multi: MultiChoice<EffectFormwork> =
        MultiChoice::new("HomogDry", vec![prof_dry.clone()])
            .with_cardinality(Cardinality::Exact(1));
    let multi_slot: MultiSlot<EffectFormwork, Effect> = Slot::Choose(homogeneous_dry_multi);

    let mut wet_parent_c: FullMatrixParentFormwork = FullMatrixParentFormwork::builder()
        .reg_dry(prof_dry.clone())
        .reg_wet(bonus_wet.clone())
        .c_single(homogeneous_wet_choice)
        .c_option(Some(heterogeneous_wet_choice))
        .with_c_collection(SingleChoice::new("ColChoice", vec![prof_dry.clone()]))
        .c_pick(multi_slot)
        .nc_single(prof_dry.clone())
        .nc_option(Some(bonus_dry.clone()))
        .nc_collection(vec![prof_dry.clone()])
        .nc_pick(vec![bonus_dry.clone()])
        .nc_ff_single(tag_dry.clone())
        .nc_ff_option(tag_dry.clone())
        .nc_ff_collection(vec![tag_dry.clone()])
        .build();

    wet_parent_c.walk_choices(&mut AutoSelectFirst);
    assert!(wet_parent_c.is_complete());
    let resolved_c = wet_parent_c.resolve().unwrap();
    assert_eq!(resolved_c.c_pick, vec![Effect::Proficiency(prof_dry)]);
}

#[test]
fn test_builder_ergonomics_dry_mode() {
    let prof_dry: Proficiency = Proficiency::builder()
        .category("Skill")
        .skill("Athletics")
        .build();
    let bonus_dry: AbilityBonus = AbilityBonus::builder()
        .amount(2)
        .ability("STR")
        .build();
    let tag_dry: StaticTag = StaticTag::builder()
        .label("Alert")
        .note(None::<String>)
        .build();

    let dry_parent: FullMatrixParent = FullMatrixParent::builder()
        .reg_dry(prof_dry.clone())
        .with_reg_dry_vec(bonus_dry.clone())
        .reg_wet(prof_dry.clone())
        .c_single(prof_dry.clone())
        .c_option(bonus_dry.clone())
        .with_c_collection(prof_dry.clone())
        .with_c_collection(tag_dry.clone())
        .c_pick([prof_dry.clone()])
        .nc_single(bonus_dry.clone())
        .nc_option(Some(tag_dry.clone()))
        .with_nc_collection(prof_dry.clone())
        .nc_pick(vec![bonus_dry.clone()])
        .nc_ff_single(tag_dry.clone())
        .nc_ff_option(None::<Proficiency>)
        .with_nc_ff_collection(bonus_dry.clone())
        .build();

    assert_eq!(dry_parent.reg_dry, Effect::Proficiency(prof_dry.clone()));
    assert_eq!(dry_parent.c_single, Effect::Proficiency(prof_dry.clone()));
    assert_eq!(dry_parent.c_option, Some(Effect::Bonus(bonus_dry.clone())));
    assert_eq!(dry_parent.c_collection.len(), 2);
    assert_eq!(dry_parent.nc_option, Some(Effect::Tag(tag_dry)));
    assert_eq!(dry_parent.nc_ff_option, None);
}
