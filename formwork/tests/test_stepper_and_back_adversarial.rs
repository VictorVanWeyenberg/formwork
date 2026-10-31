use formwork::choice::{Cardinality, ChoiceState, MultiChoice, SingleChoice};
use formwork::visitor::{AutoSelectFirst, ChoiceVisitor, VisitorFlow};
use formwork::{formwork, Formwork, StepResult};

// ---------------------------------------------------------------------------
// Adversarial Models
// ---------------------------------------------------------------------------

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct LeafBfs {
    #[slot]
    pub val: String,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct LeafDfs {
    #[slot]
    pub val: String,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct ChildCollectionBfs {
    pub tag: String,
    #[slot(collection, cascade)]
    pub leaves: Vec<LeafBfs>,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct ChildCollectionDfs {
    pub tag: String,
    #[slot(collection, cascade)]
    pub leaves: Vec<LeafDfs>,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct RootCollectionBfs {
    #[slot]
    pub root_choice: String,
    #[slot(collection, cascade)]
    pub children: Vec<ChildCollectionBfs>,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct RootCollectionDfs {
    #[slot]
    pub root_choice: String,
    #[slot(collection, cascade)]
    pub children: Vec<ChildCollectionDfs>,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct OptionalChildBfs {
    #[slot]
    pub root_choice: String,
    #[slot(option, cascade)]
    pub opt_child: Option<LeafBfs>,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct OptionalChildDfs {
    #[slot]
    pub root_choice: String,
    #[slot(option, cascade)]
    pub opt_child: Option<LeafDfs>,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct MultiChoiceModel {
    #[slot(pick)]
    pub multi: Vec<String>,
    #[slot]
    pub single: String,
}

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ZeroChoiceModel {
    #[slot]
    pub fixed_text: String,
}

// ---------------------------------------------------------------------------
// Adversarial Tests
// ---------------------------------------------------------------------------

#[test]
fn test_adversarial_alternating_back_and_continue() {
    let mut root = RootCollectionBfs::builder()
        .root_choice(SingleChoice::new("Root", vec!["r1".to_string()]))
        .children(vec![
            ChildCollectionBfs::builder()
                .tag("c0".to_string())
                .leaves(vec![LeafBfs::builder()
                    .val(SingleChoice::new("Leaf0_0", vec!["l0_0".to_string()]))
                    .build()])
                .build(),
            ChildCollectionBfs::builder()
                .tag("c1".to_string())
                .leaves(vec![LeafBfs::builder()
                    .val(SingleChoice::new("Leaf1_0", vec!["l1_0".to_string()]))
                    .build()])
                .build(),
        ])
        .build();

    let mut session = root.session();

    // Choice 0: Root
    let mut v = AutoSelectFirst;
    assert_eq!(session.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Choice 1: Leaf0_0 -> User changes mind, calls Back
    struct BackVisitor;
    impl<T> ChoiceVisitor<T> for BackVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
    }

    let mut v_back = BackVisitor;
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // Re-visiting Choice 0: Continue
    assert_eq!(session.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Choice 1: Continue
    assert_eq!(session.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 2);

    // Choice 2: Leaf1_0 -> Call Back
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 1);

    // Choice 1 again -> Call Back again
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // Choice 0 again -> Call Back at start!
    assert_eq!(session.step(&mut v_back), StepResult::AtStart);
    assert_eq!(session.current_step(), 0);

    // Call Back AGAIN while at start (should stay at 0 and return AtStart)
    assert_eq!(session.step(&mut v_back), StepResult::AtStart);
    assert_eq!(session.current_step(), 0);

    // Now proceed forward all the way to completion
    assert_eq!(session.step(&mut v), StepResult::Choice(VisitorFlow::Continue)); // 0
    assert_eq!(session.step(&mut v), StepResult::Choice(VisitorFlow::Continue)); // 1
    assert_eq!(session.step(&mut v), StepResult::Choice(VisitorFlow::Continue)); // 2
    assert_eq!(session.step(&mut v), StepResult::Complete);
    assert!(session.is_complete());

    let resolved = root.resolve().expect("Should resolve cleanly after rewinds");
    assert_eq!(resolved.root_choice, "r1");
    assert_eq!(resolved.children[0].leaves[0].val, "l0_0");
    assert_eq!(resolved.children[1].leaves[0].val, "l1_0");
}

#[test]
fn test_adversarial_optional_cascading_none_vs_some() {
    // 1. Optional child is None (omitted in builder)
    let mut none_parent = OptionalChildBfs::builder()
        .root_choice(SingleChoice::new("RootChoice", vec!["r".to_string()]))
        .build();

    let mut session_none = none_parent.session();
    let mut v = AutoSelectFirst;
    assert_eq!(session_none.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    // Since opt_child is None, step 1 is exhausted -> Complete!
    assert_eq!(session_none.step(&mut v), StepResult::Complete);
    assert!(session_none.is_complete());

    let resolved_none = none_parent.resolve().unwrap();
    assert_eq!(resolved_none.root_choice, "r");
    assert_eq!(resolved_none.opt_child, None);

    // 2. Optional child is Some
    let leaf = LeafBfs::builder()
        .val(SingleChoice::new("LeafChoice", vec!["leaf".to_string()]))
        .build();
    let mut some_parent = OptionalChildBfs::builder()
        .root_choice(SingleChoice::new("RootChoice", vec!["r".to_string()]))
        .opt_child(leaf)
        .build();

    let mut session_some = some_parent.session();
    assert_eq!(session_some.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session_some.current_step(), 1);

    // At LeafChoice, call Back!
    struct BackVisitor;
    impl<T> ChoiceVisitor<T> for BackVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
    }
    let mut v_back = BackVisitor;
    assert_eq!(session_some.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session_some.current_step(), 0);

    // Root choice was unmade
    assert!(matches!(
        session_some.formwork().root_choice.as_choice().unwrap().state,
        ChoiceState::UnmadeWithHistory(_)
    ));
}

#[test]
fn test_adversarial_skipped_unconfirmed_choices_then_back() {
    // Visitor returns Continue without selecting or confirming anything!
    struct SkipVisitor;
    impl<T> ChoiceVisitor<T> for SkipVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
            VisitorFlow::Continue
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    struct BackVisitor;
    impl<T> ChoiceVisitor<T> for BackVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
    }

    let mut root = RootCollectionDfs::builder()
        .root_choice(SingleChoice::new("RootChoice", vec!["r".to_string()]))
        .children(vec![ChildCollectionDfs::builder()
            .tag("child".to_string())
            .leaves(vec![LeafDfs::builder()
                .val(SingleChoice::new("LeafChoice", vec!["l".to_string()]))
                .build()])
            .build()])
        .build();

    let mut session = root.session();

    // Step 0: RootChoice skipped (remains Pending)
    let mut v_skip = SkipVisitor;
    assert_eq!(session.step(&mut v_skip), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);
    assert!(session.formwork().root_choice.as_choice().unwrap().state.is_pending());

    // Step 1: LeafChoice -> user clicks Back!
    let mut v_back = BackVisitor;
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // RootChoice was Pending when unmade. It shouldn't panic or corrupt state!
    assert!(session.formwork().root_choice.as_choice().unwrap().state.is_pending());

    // Now confirm it cleanly
    let mut v_auto = AutoSelectFirst;
    assert_eq!(session.step(&mut v_auto), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.step(&mut v_auto), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.step(&mut v_auto), StepResult::Complete);

    let resolved = root.resolve().unwrap();
    assert_eq!(resolved.root_choice, "r");
    assert_eq!(resolved.children[0].leaves[0].val, "l");
}

#[test]
fn test_adversarial_multichoice_cardinality_and_rewind() {
    let mut model = MultiChoiceModel::builder()
        .multi(
            MultiChoice::new(
                "Select abilities",
                vec!["Fire".to_string(), "Ice".to_string(), "Lightning".to_string()],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )
        .single(SingleChoice::new(
            "Select weapon",
            vec!["Staff".to_string(), "Wand".to_string()],
        ))
        .build();

    let mut session = model.session();

    // Step 0: MultiChoice -> select 2 items and confirm
    struct MultiSelectVisitor;
    impl ChoiceVisitor<String> for MultiSelectVisitor {
        fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
            let _ = choice.select(0);
            let _ = choice.confirm();
            VisitorFlow::Continue
        }
        fn visit_multi(&mut self, choice: &mut MultiChoice<String>) -> VisitorFlow {
            let _ = choice.select(0);
            let _ = choice.select(1);
            let _ = choice.confirm();
            VisitorFlow::Continue
        }
    }

    let mut v_multi = MultiSelectVisitor;
    assert_eq!(session.step(&mut v_multi), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Step 1: SingleChoice -> click Back!
    struct BackVisitor;
    impl ChoiceVisitor<String> for BackVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<String>) -> VisitorFlow {
            VisitorFlow::Back
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<String>) -> VisitorFlow {
            VisitorFlow::Back
        }
    }

    let mut v_back = BackVisitor;
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // MultiChoice was unmade to UnmadeWithHistory(vec![0, 1])
    assert!(matches!(
        session.formwork().multi.as_choice().unwrap().state,
        ChoiceState::UnmadeWithHistory(ref indices) if indices == &[0, 1]
    ));

    // Re-visiting MultiChoice: user changes selection from [0, 1] to [1, 2]
    struct ChangeSelectionVisitor;
    impl ChoiceVisitor<String> for ChangeSelectionVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<String>) -> VisitorFlow {
            VisitorFlow::Continue
        }
        fn visit_multi(&mut self, choice: &mut MultiChoice<String>) -> VisitorFlow {
            // Unselect 0, select 2
            let _ = choice.deselect(0);
            let _ = choice.select(2);
            let _ = choice.confirm();
            VisitorFlow::Continue
        }
    }

    let mut v_change = ChangeSelectionVisitor;
    assert_eq!(session.step(&mut v_change), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Select weapon and finish
    let mut v_auto = AutoSelectFirst;
    assert_eq!(session.step(&mut v_auto), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.step(&mut v_auto), StepResult::Complete);

    let resolved = model.resolve().expect("MultiChoice should resolve");
    assert_eq!(resolved.multi, vec!["Ice", "Lightning"]);
    assert_eq!(resolved.single, "Staff");
}

#[test]
fn test_adversarial_zero_choice_model() {
    let mut zero = ZeroChoiceModel::builder()
        .fixed_text("constant".to_string())
        .build();

    let mut session = zero.session();
    assert_eq!(session.current_step(), 0);
    assert!(session.is_complete());

    let mut v = AutoSelectFirst;
    // Stepping immediately returns Complete because target 0 is exhausted
    assert_eq!(session.step(&mut v), StepResult::Complete);

    // Resolves cleanly
    let resolved = zero.resolve().unwrap();
    assert_eq!(resolved.fixed_text, "constant");
}

#[test]
fn test_adversarial_break_does_not_advance_cursor() {
    let mut model = MultiChoiceModel::builder()
        .multi(
            MultiChoice::new("Abilities", vec!["Fire".to_string()])
                .with_cardinality(Cardinality::Exact(1)),
        )
        .single(SingleChoice::new("Weapon", vec!["Sword".to_string()]))
        .build();

    let mut session = model.session();

    struct BreakVisitor;
    impl ChoiceVisitor<String> for BreakVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<String>) -> VisitorFlow {
            VisitorFlow::Break
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<String>) -> VisitorFlow {
            VisitorFlow::Break
        }
    }

    let mut v_break = BreakVisitor;
    // Returning Break at step 0
    assert_eq!(session.step(&mut v_break), StepResult::Choice(VisitorFlow::Break));
    assert_eq!(session.current_step(), 0); // Cursor did not advance

    // Break again
    assert_eq!(session.step(&mut v_break), StepResult::Choice(VisitorFlow::Break));
    assert_eq!(session.current_step(), 0);

    // Now continue
    let mut v_auto = AutoSelectFirst;
    assert_eq!(session.step(&mut v_auto), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);
}

#[test]
fn test_adversarial_walk_choices_with_rewind_loop() {
    let mut model = MultiChoiceModel::builder()
        .multi(
            MultiChoice::new("Abilities", vec!["Fire".to_string()])
                .with_cardinality(Cardinality::Exact(1)),
        )
        .single(SingleChoice::new("Weapon", vec!["Sword".to_string()]))
        .build();

    // Visitor that confirms step 0, calls Back at step 1 ONCE, then on retry completes step 1
    struct BackOnceVisitor {
        step1_visited_count: usize,
    }

    impl ChoiceVisitor<String> for BackOnceVisitor {
        fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
            self.step1_visited_count += 1;
            if self.step1_visited_count == 1 {
                VisitorFlow::Back
            } else {
                let _ = choice.select(0);
                let _ = choice.confirm();
                VisitorFlow::Continue
            }
        }

        fn visit_multi(&mut self, choice: &mut MultiChoice<String>) -> VisitorFlow {
            let _ = choice.select(0);
            let _ = choice.confirm();
            VisitorFlow::Continue
        }
    }

    let mut visitor = BackOnceVisitor {
        step1_visited_count: 0,
    };

    let flow = model.walk_choices(&mut visitor);
    assert_eq!(flow, VisitorFlow::Continue);
    assert!(model.is_complete());
    assert_eq!(visitor.step1_visited_count, 2);

    let resolved = model.resolve().unwrap();
    assert_eq!(resolved.multi, vec!["Fire"]);
    assert_eq!(resolved.single, "Sword");
}

// ---------------------------------------------------------------------------
// Deep Branching Tree Models for BFS vs DFS Step Rewinding
// ---------------------------------------------------------------------------

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepLeafBfs {
    #[slot]
    pub val: String,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepBranchBfs {
    #[slot]
    pub branch_choice: String,
    #[slot(cascade)]
    pub leaf1: DeepLeafBfs,
    #[slot(cascade)]
    pub leaf2: DeepLeafBfs,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepRootBfs {
    #[slot]
    pub root_choice: String,
    #[slot(cascade)]
    pub b1: DeepBranchBfs,
    #[slot(cascade)]
    pub b2: DeepBranchBfs,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepLeafDfs {
    #[slot]
    pub val: String,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepBranchDfs {
    #[slot]
    pub branch_choice: String,
    #[slot(cascade)]
    pub leaf1: DeepLeafDfs,
    #[slot(cascade)]
    pub leaf2: DeepLeafDfs,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct DeepRootDfs {
    #[slot]
    pub root_choice: String,
    #[slot(cascade)]
    pub b1: DeepBranchDfs,
    #[slot(cascade)]
    pub b2: DeepBranchDfs,
}

struct PromptRecordingVisitor {
    prompts: Vec<String>,
    auto_confirm: bool,
}

impl PromptRecordingVisitor {
    fn new(auto_confirm: bool) -> Self {
        Self {
            prompts: Vec::new(),
            auto_confirm,
        }
    }
}

impl<T> ChoiceVisitor<T> for PromptRecordingVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        self.prompts.push(choice.prompt().to_string());
        if self.auto_confirm {
            if choice.selected_index().is_none() && !choice.options().is_empty() {
                let _ = choice.select(0);
            }
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        self.prompts.push(choice.prompt().to_string());
        if self.auto_confirm {
            if choice.selected_indices().is_empty() && !choice.options().is_empty() {
                let _ = choice.select(0);
            }
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }
}

#[test]
fn test_adversarial_deep_bfs_tree_stepping_and_back_rewind() {
    let mut tree = DeepRootBfs::builder()
        .root_choice(SingleChoice::new("R0", vec!["r0".to_string()]))
        .b1(
            DeepBranchBfs::builder()
                .branch_choice(SingleChoice::new("B1", vec!["b1".to_string()]))
                .leaf1(
                    DeepLeafBfs::builder()
                        .val(SingleChoice::new("L1_1", vec!["l1_1".to_string()]))
                        .build(),
                )
                .leaf2(
                    DeepLeafBfs::builder()
                        .val(SingleChoice::new("L1_2", vec!["l1_2".to_string()]))
                        .build(),
                )
                .build(),
        )
        .b2(
            DeepBranchBfs::builder()
                .branch_choice(SingleChoice::new("B2", vec!["b2".to_string()]))
                .leaf1(
                    DeepLeafBfs::builder()
                        .val(SingleChoice::new("L2_1", vec!["l2_1".to_string()]))
                        .build(),
                )
                .leaf2(
                    DeepLeafBfs::builder()
                        .val(SingleChoice::new("L2_2", vec!["l2_2".to_string()]))
                        .build(),
                )
                .build(),
        )
        .build();

    let mut session = tree.session();

    // Step through first 4 choices in BFS order:
    // Level 0: R0
    // Level 1: B1, B2
    // Level 2: L1_1
    let mut v = PromptRecordingVisitor::new(true);

    for _ in 0..4 {
        assert_eq!(session.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    }

    assert_eq!(v.prompts, vec!["R0", "B1", "B2", "L1_1"]);
    assert_eq!(session.current_step(), 4);

    // Now call Back: from cursor 4 (L1_2) back to L1_1 (cursor 3)
    struct BackVisitor;
    impl<T> ChoiceVisitor<T> for BackVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
    }

    let mut v_back = BackVisitor;
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 3);

    // Back again -> cursor 2 (B2)
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 2);

    // Back again -> cursor 1 (B1)
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 1);

    // Back again -> cursor 0 (R0)
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // Back again at 0 -> AtStart!
    assert_eq!(session.step(&mut v_back), StepResult::AtStart);
    assert_eq!(session.current_step(), 0);

    // All choices were unmade to UnmadeWithHistory!
    assert!(session.formwork().root_choice.as_choice().unwrap().state.is_unmade());
    assert!(session.formwork().b1.as_choice().is_none()); // branch is FixedWet slot
    assert!(session.formwork().b1.as_wet().unwrap().branch_choice.as_choice().unwrap().state.is_unmade());

    // Now forward all the way to completion (7 choices)
    let mut v_forward = PromptRecordingVisitor::new(true);
    while let StepResult::Choice(flow) = session.step(&mut v_forward) {
        assert_eq!(flow, VisitorFlow::Continue);
    }

    // Verify BFS prompt order
    assert_eq!(v_forward.prompts, vec!["R0", "B1", "B2", "L1_1", "L1_2", "L2_1", "L2_2"]);
    assert!(session.is_complete());

    let resolved = tree.resolve().expect("Deep BFS tree should resolve");
    assert_eq!(resolved.root_choice, "r0");
    assert_eq!(resolved.b1.branch_choice, "b1");
    assert_eq!(resolved.b1.leaf1.val, "l1_1");
    assert_eq!(resolved.b1.leaf2.val, "l1_2");
    assert_eq!(resolved.b2.branch_choice, "b2");
    assert_eq!(resolved.b2.leaf1.val, "l2_1");
    assert_eq!(resolved.b2.leaf2.val, "l2_2");
}

#[test]
fn test_adversarial_deep_dfs_tree_stepping_and_back_rewind() {
    let mut tree = DeepRootDfs::builder()
        .root_choice(SingleChoice::new("R0", vec!["r0".to_string()]))
        .b1(
            DeepBranchDfs::builder()
                .branch_choice(SingleChoice::new("B1", vec!["b1".to_string()]))
                .leaf1(
                    DeepLeafDfs::builder()
                        .val(SingleChoice::new("L1_1", vec!["l1_1".to_string()]))
                        .build(),
                )
                .leaf2(
                    DeepLeafDfs::builder()
                        .val(SingleChoice::new("L1_2", vec!["l1_2".to_string()]))
                        .build(),
                )
                .build(),
        )
        .b2(
            DeepBranchDfs::builder()
                .branch_choice(SingleChoice::new("B2", vec!["b2".to_string()]))
                .leaf1(
                    DeepLeafDfs::builder()
                        .val(SingleChoice::new("L2_1", vec!["l2_1".to_string()]))
                        .build(),
                )
                .leaf2(
                    DeepLeafDfs::builder()
                        .val(SingleChoice::new("L2_2", vec!["l2_2".to_string()]))
                        .build(),
                )
                .build(),
        )
        .build();

    let mut session = tree.session();

    // Step through all 7 choices in DFS order:
    // R0, B1, L1_1, L1_2, B2, L2_1, L2_2
    let mut v = PromptRecordingVisitor::new(true);
    for _ in 0..7 {
        assert_eq!(session.step(&mut v), StepResult::Choice(VisitorFlow::Continue));
    }
    assert_eq!(
        v.prompts,
        vec!["R0", "B1", "L1_1", "L1_2", "B2", "L2_1", "L2_2"]
    );
    assert_eq!(session.current_step(), 7);
    assert_eq!(session.step(&mut v), StepResult::Complete);

    // Call Back from Complete:
    struct BackVisitor;
    impl<T> ChoiceVisitor<T> for BackVisitor {
        fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
            VisitorFlow::Back
        }
    }
    let mut v_back = BackVisitor;
    // When Complete has been reached (all choices exhausted), step() returns Complete directly
    assert_eq!(session.step(&mut v_back), StepResult::Complete);
    assert_eq!(session.current_step(), 7);
}
