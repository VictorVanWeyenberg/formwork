#![deny(missing_docs)]
//! Integration test verifying that `#[formwork]` enums satisfy `#![deny(missing_docs)]`
//! with enum-appropriate documentation on all generated public items.

use formwork::visitor::AutoSelectFirst;
use formwork::{Formwork, formwork};

/// Public documented module containing public formwork enums under `#![deny(missing_docs)]`.
pub mod documented_enums {
    use super::*;

    /// First inner formwork struct.
    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ProficiencyGrant {
        /// Skill or tool category.
        pub category: String,
        /// Chosen proficiency.
        #[slot]
        pub skill: String,
    }

    /// Second inner formwork struct.
    #[formwork(builder)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct AbilityBonus {
        /// Bonus amount.
        pub amount: i32,
        /// Chosen ability score.
        #[slot]
        pub ability: String,
    }

    /// A documented formwork enum with derives forwarding and a cfg-gated variant.
    #[formwork(derives(Eq, Hash))]
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub enum FeatureEffect {
        /// Grants a proficiency.
        #[cfg(test)]
        Proficiency(ProficiencyGrant),
        /// Grants an ability bonus.
        Bonus(AbilityBonus),
    }

    /// A documented nested formwork enum wrapping another formwork enum.
    #[formwork]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum OuterFeature {
        /// Wraps a feature effect enum.
        Effect(FeatureEffect),
    }
}

#[test]
fn test_documented_enum_lifecycle() {
    use documented_enums::*;
    use formwork::choice::SingleChoice;

    let inner = ProficiencyGrant::builder()
        .category("Skill")
        .skill(SingleChoice::new(
            "Choose skill",
            vec!["Athletics".to_string(), "Stealth".to_string()],
        ))
        .build();

    let mut effect: FeatureEffectFormwork = inner.into();
    assert!(!effect.is_complete());
    effect.walk_choices(&mut AutoSelectFirst);
    assert!(effect.is_complete());

    let resolved: FeatureEffect = effect.resolve().expect("should resolve");
    assert_eq!(
        resolved,
        FeatureEffect::Proficiency(ProficiencyGrant {
            category: "Skill".to_string(),
            skill: "Athletics".to_string(),
        })
    );

    let outer: OuterFeatureFormwork = resolved.into();
    assert!(outer.is_complete());
    let resolved_outer: OuterFeature = outer.resolve().expect("should resolve outer");
    assert!(matches!(resolved_outer, OuterFeature::Effect(_)));
}
