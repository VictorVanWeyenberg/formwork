use formwork::{choice::SingleChoice, formwork, visitor::AutoSelectFirst, Formwork};

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct HeroClass {
    pub name: String,
    #[slot]
    pub primary_weapon: String,
    #[slot(collection)]
    pub proficiencies: Vec<String>,
}

#[test]
fn test_facade_end_to_end_smoke() {
    let mut hero_formwork = HeroClassFormwork::builder()
        .name("Fighter".to_string())
        .primary_weapon(SingleChoice::new(
            "Primary Weapon",
            vec![
                "Longsword".to_string(),
                "Greatsword".to_string(),
                "Shortbow".to_string(),
            ],
        ))
        .with_proficiencies("Shields".to_string())
        .build();

    // 1. Initial state is incomplete
    assert!(!hero_formwork.is_complete());

    // 2. AutoSelectFirst satisfies all choices (selects option 0 and confirms)
    hero_formwork.walk_choices(&mut AutoSelectFirst);
    assert!(hero_formwork.is_complete());

    // 3. Resolve into watertight domain model
    let hero: HeroClass = hero_formwork.resolve().expect("must resolve cleanly");
    assert_eq!(hero.name, "Fighter");
    assert_eq!(hero.primary_weapon, "Longsword");
    assert_eq!(hero.proficiencies, vec!["Shields".to_string()]);
}

#[test]
fn test_facade_manual_select_confirm_contract() {
    let mut choice: SingleChoice<&str> = SingleChoice::new("Skill", vec!["Athletics", "Acrobatics"]);
    assert!(!choice.is_made());

    choice.select(1).unwrap();
    assert!(!choice.is_made(), "select must not set made state");

    choice.confirm().unwrap();
    assert!(choice.is_made(), "confirm transitions to made");

    let val = choice.resolve_exact().unwrap();
    assert_eq!(val, "Acrobatics");
}
