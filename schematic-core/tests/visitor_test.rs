use schematic_core::choice::{Cardinality, MultiChoice, SingleChoice};
use schematic_core::visitor::{AutoSelectFirst, ChoiceVisitor, ConfirmAll, UnmakeAll, VisitorFlow};

#[test]
fn test_auto_select_first_stages_and_confirms() {
    let mut single = SingleChoice::new("Hero", vec!["Paladin", "Ranger"]);
    let mut multi = MultiChoice::new("Feats", vec!["Alert", "Tough", "Lucky"])
        .with_cardinality(Cardinality::Exact(2));

    assert!(!single.is_made());
    assert!(!multi.is_made());

    let mut visitor = AutoSelectFirst;
    assert_eq!(visitor.visit_single(&mut single), VisitorFlow::Continue);
    assert_eq!(visitor.visit_multi(&mut multi), VisitorFlow::Continue);

    assert!(single.is_made());
    assert_eq!(single.selected_index(), Some(0));
    assert_eq!(single.resolve_exact().unwrap(), "Paladin");

    assert!(multi.is_made());
    assert_eq!(multi.selected_indices(), &[0, 1]);
    assert_eq!(multi.resolve().unwrap(), vec!["Alert", "Tough"]);
}

#[test]
fn test_unmake_all_and_confirm_all_cycle() {
    let mut single = SingleChoice::new("Hero", vec!["Paladin", "Ranger"]);
    single.select(1).unwrap();
    single.confirm().unwrap();
    assert!(single.is_made());

    let mut unmake_visitor = UnmakeAll;
    assert_eq!(
        unmake_visitor.visit_single(&mut single),
        VisitorFlow::Continue
    );
    assert!(!single.is_made(), "UnmakeAll must revert choice to unmade");

    let mut confirm_visitor = ConfirmAll;
    assert_eq!(
        confirm_visitor.visit_single(&mut single),
        VisitorFlow::Continue
    );
    assert!(
        single.is_made(),
        "ConfirmAll must restore unmade choice with history"
    );
    assert_eq!(single.resolve_exact().unwrap(), "Ranger");
}
