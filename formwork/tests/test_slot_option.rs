use formwork::choice::SingleChoice;
use formwork::slot::SingleSlot;
use formwork::visitor::AutoSelectFirst;
use formwork::{formwork, Formwork};

// ----------------------------------------------------------------------------
// Leaf #[slot(option)]
// ----------------------------------------------------------------------------

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeroProfile {
    pub name: String,
    #[slot(option)]
    pub title: Option<String>,
}

#[test]
fn test_leaf_option_slot_default_omission() {
    let mut formwork = HeroProfileFormwork::builder()
        .name("Conan".to_string())
        .build();

    assert!(formwork.is_complete());
    assert_eq!(formwork.title, None);

    // Visitor skipping None
    formwork.walk_choices(&mut AutoSelectFirst);
    assert!(formwork.is_complete());

    let resolved = formwork.resolve().expect("must resolve None option slot");
    assert_eq!(resolved.name, "Conan");
    assert_eq!(resolved.title, None);

    // From<Dry> roundtrip
    let back_to_formwork: HeroProfileFormwork = resolved.into();
    assert_eq!(back_to_formwork.title, None);
    assert!(back_to_formwork.is_complete());
}

#[test]
fn test_leaf_option_slot_bare_value_and_some() {
    // Setting bare value
    let bp1 = HeroProfileFormwork::builder()
        .name("Arthur".to_string())
        .title("King".to_string())
        .build();
    assert!(bp1.is_complete());
    let r1 = bp1.resolve().unwrap();
    assert_eq!(r1.title, Some("King".to_string()));

    // Setting Some(value)
    let bp2 = HeroProfileFormwork::builder()
        .name("Arthur".to_string())
        .title(Some("King".to_string()))
        .build();
    assert!(bp2.is_complete());
    let r2 = bp2.resolve().unwrap();
    assert_eq!(r2.title, Some("King".to_string()));

    // Setting None explicitly
    let bp3 = HeroProfileFormwork::builder()
        .name("Arthur".to_string())
        .title(Option::<String>::None)
        .build();
    assert!(bp3.is_complete());
    let r3 = bp3.resolve().unwrap();
    assert_eq!(r3.title, None);
}

#[test]
fn test_leaf_option_slot_with_choice() {
    let choice = SingleChoice::new(
        "Choose Title",
        vec!["Knight".to_string(), "Paladin".to_string()],
    );

    let mut formwork = HeroProfileFormwork::builder()
        .name("Lancelot".to_string())
        .title(choice)
        .build();

    assert!(!formwork.is_complete());
    assert!(formwork.clone().resolve().is_err());

    // AutoSelectFirst satisfies the choice
    formwork.walk_choices(&mut AutoSelectFirst);
    assert!(formwork.is_complete());

    let resolved = formwork.resolve().expect("must resolve after choice made");
    assert_eq!(resolved.title, Some("Knight".to_string()));

    // From<Resolved> produces FixedDry slot
    let back: HeroProfileFormwork = resolved.into();
    assert!(back.is_complete());
    assert!(matches!(back.title, Some(SingleSlot::FixedDry(_))));
}

// ----------------------------------------------------------------------------
// Cascading #[slot(option, cascade)]
// ----------------------------------------------------------------------------

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pet {
    pub species: String,
    #[slot]
    pub trick: String,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdventurerBfs {
    pub name: String,
    #[slot(option, cascade)]
    pub pet: Option<Pet>,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdventurerDfs {
    pub name: String,
    #[slot(cascade, option)]
    pub pet: Option<Pet>,
}

#[test]
fn test_cascading_option_slot_none() {
    let mut adv = AdventurerBfsFormwork::builder()
        .name("Ranger".to_string())
        .build();

    assert!(adv.is_complete());
    adv.walk_choices(&mut AutoSelectFirst);
    assert!(adv.is_complete());

    let resolved = adv.resolve().expect("must resolve empty cascade option");
    assert_eq!(resolved.name, "Ranger");
    assert_eq!(resolved.pet, None);

    let back: AdventurerBfsFormwork = resolved.into();
    assert!(back.is_complete());
    assert_eq!(back.pet, None);
}

#[test]
fn test_cascading_option_slot_bfs_walk() {
    let pet_choice = SingleChoice::new("Trick", vec!["Fetch".to_string(), "Roll".to_string()]);
    let pet_formwork = PetFormwork::builder()
        .species("Wolf".to_string())
        .trick(pet_choice)
        .build();

    let mut adv = AdventurerBfsFormwork::builder()
        .name("Hunter".to_string())
        .pet(pet_formwork)
        .build();

    assert!(!adv.is_complete());
    assert!(adv.clone().resolve().is_err());

    adv.walk_choices(&mut AutoSelectFirst);
    assert!(adv.is_complete());

    let resolved = adv.resolve().expect("must resolve complete cascading pet");
    assert_eq!(
        resolved.pet,
        Some(Pet {
            species: "Wolf".to_string(),
            trick: "Fetch".to_string(),
        })
    );
}

#[test]
fn test_cascading_option_slot_dfs_walk() {
    let pet_choice = SingleChoice::new("Trick", vec!["Jump".to_string(), "Sit".to_string()]);
    let pet_formwork = PetFormwork::builder()
        .species("Cat".to_string())
        .trick(pet_choice)
        .build();

    let mut adv = AdventurerDfsFormwork::builder()
        .name("Mage".to_string())
        .pet(Some(pet_formwork))
        .build();

    assert!(!adv.is_complete());
    adv.walk_choices(&mut AutoSelectFirst);
    assert!(adv.is_complete());

    let resolved = adv.resolve().expect("must resolve dfs cascading pet");
    assert_eq!(
        resolved.pet,
        Some(Pet {
            species: "Cat".to_string(),
            trick: "Jump".to_string(),
        })
    );
}

#[test]
fn test_direct_resolved_mode_builders() {
    let hero: HeroProfile = HeroProfile::builder()
        .name("Direct".to_string())
        .title("Master".to_string())
        .build();
    assert_eq!(hero.title, Some("Master".to_string()));

    let hero_none: HeroProfile = HeroProfile::builder()
        .name("DirectNone".to_string())
        .build();
    assert_eq!(hero_none.title, None);

    let adv: AdventurerBfs = AdventurerBfs::builder()
        .name("Beastmaster".to_string())
        .pet(Pet {
            species: "Bear".to_string(),
            trick: "Roar".to_string(),
        })
        .build();
    assert!(adv.pet.is_some());
}
