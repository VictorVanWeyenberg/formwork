use schematic::choice::SingleChoice;
use schematic::slot::SingleSlot;
use schematic::visitor::AutoSelectFirst;
use schematic::{schematic, Schematic};

// ----------------------------------------------------------------------------
// Leaf #[slot(option)]
// ----------------------------------------------------------------------------

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeroProfile {
    pub name: String,
    #[slot(option)]
    pub title: Option<String>,
}

#[test]
fn test_leaf_option_slot_default_omission() {
    let mut blueprint = HeroProfileSchematic::builder()
        .name("Conan".to_string())
        .build();

    assert!(blueprint.is_complete());
    assert_eq!(blueprint.title, None);

    // Visitor skipping None
    blueprint.walk_choices(&mut AutoSelectFirst);
    assert!(blueprint.is_complete());

    let resolved = blueprint.resolve().expect("must resolve None option slot");
    assert_eq!(resolved.name, "Conan");
    assert_eq!(resolved.title, None);

    // From<Resolved> roundtrip
    let back_to_schematic: HeroProfileSchematic = resolved.into();
    assert_eq!(back_to_schematic.title, None);
    assert!(back_to_schematic.is_complete());
}

#[test]
fn test_leaf_option_slot_bare_value_and_some() {
    // Setting bare value
    let bp1 = HeroProfileSchematic::builder()
        .name("Arthur".to_string())
        .title("King".to_string())
        .build();
    assert!(bp1.is_complete());
    let r1 = bp1.resolve().unwrap();
    assert_eq!(r1.title, Some("King".to_string()));

    // Setting Some(value)
    let bp2 = HeroProfileSchematic::builder()
        .name("Arthur".to_string())
        .title(Some("King".to_string()))
        .build();
    assert!(bp2.is_complete());
    let r2 = bp2.resolve().unwrap();
    assert_eq!(r2.title, Some("King".to_string()));

    // Setting None explicitly
    let bp3 = HeroProfileSchematic::builder()
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

    let mut blueprint = HeroProfileSchematic::builder()
        .name("Lancelot".to_string())
        .title(choice)
        .build();

    assert!(!blueprint.is_complete());
    assert!(blueprint.clone().resolve().is_err());

    // AutoSelectFirst satisfies the choice
    blueprint.walk_choices(&mut AutoSelectFirst);
    assert!(blueprint.is_complete());

    let resolved = blueprint.resolve().expect("must resolve after choice made");
    assert_eq!(resolved.title, Some("Knight".to_string()));

    // From<Resolved> produces FixedResolved slot
    let back: HeroProfileSchematic = resolved.into();
    assert!(back.is_complete());
    assert!(matches!(back.title, Some(SingleSlot::FixedResolved(_))));
}

// ----------------------------------------------------------------------------
// Cascading #[slot(option, cascade)]
// ----------------------------------------------------------------------------

#[schematic(builder)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pet {
    pub species: String,
    #[slot]
    pub trick: String,
}

#[schematic(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdventurerBfs {
    pub name: String,
    #[slot(option, cascade)]
    pub pet: Option<Pet>,
}

#[schematic(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdventurerDfs {
    pub name: String,
    #[slot(cascade, option)]
    pub pet: Option<Pet>,
}

#[test]
fn test_cascading_option_slot_none() {
    let mut adv = AdventurerBfsSchematic::builder()
        .name("Ranger".to_string())
        .build();

    assert!(adv.is_complete());
    adv.walk_choices(&mut AutoSelectFirst);
    assert!(adv.is_complete());

    let resolved = adv.resolve().expect("must resolve empty cascade option");
    assert_eq!(resolved.name, "Ranger");
    assert_eq!(resolved.pet, None);

    let back: AdventurerBfsSchematic = resolved.into();
    assert!(back.is_complete());
    assert_eq!(back.pet, None);
}

#[test]
fn test_cascading_option_slot_bfs_walk() {
    let pet_choice = SingleChoice::new("Trick", vec!["Fetch".to_string(), "Roll".to_string()]);
    let pet_schematic = PetSchematic::builder()
        .species("Wolf".to_string())
        .trick(pet_choice)
        .build();

    let mut adv = AdventurerBfsSchematic::builder()
        .name("Hunter".to_string())
        .pet(pet_schematic)
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
    let pet_schematic = PetSchematic::builder()
        .species("Cat".to_string())
        .trick(pet_choice)
        .build();

    let mut adv = AdventurerDfsSchematic::builder()
        .name("Mage".to_string())
        .pet(Some(pet_schematic))
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
