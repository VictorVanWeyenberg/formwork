use blueprint::abstract_model::*;
use blueprint::cardinality::Cardinality;
use blueprint::choice::{Choice, ChoiceCollection, ChoiceOption};
use blueprint::error::BlueprintError;
use blueprint::slot::{Slot, SlotCollection};
use blueprint::visitor::{
    AutoSelectFirst, ChoiceCollectionVisitor, ChoiceVisitor,
    VisitorFlow, StepBack,
};
use blueprint::Blueprint;

#[test]
fn test_all_fixed_draft_resolves_immediately() {
    let leaf = Leaf::new("Oak", 42);
    let branch = Branch::draft("Main", [leaf.clone().into()], vec!["tall".into(), "green".into()]);
    let root = Root::draft("root-1", branch.clone(), [branch.clone().into()]);

    assert!(root.is_complete());
    let resolved = root.resolve().expect("should resolve without open choices");

    assert_eq!(resolved.id, "root-1");
    assert_eq!(resolved.primary_branch.name, "Main");
    assert_eq!(resolved.primary_branch.leaves, vec![leaf]);
    assert_eq!(resolved.primary_branch.tags, vec!["tall", "green"]);
    assert_eq!(resolved.branches.len(), 1);
}

#[test]
fn test_unresolved_draft_fails_to_resolve() {
    let leaf_choice = Choice::new(
        "Choose a leaf:",
        vec![
            ChoiceOption::new("Maple", Leaf::new("Maple", 10)),
            ChoiceOption::new("Pine", Leaf::new("Pine", 20)),
        ],
    );

    let branch = Branch::draft("Branch-A", [leaf_choice.into()], vec!["needle".into()]);
    let root = Root::draft("root-fail", branch, vec![]);

    assert!(!root.is_complete());
    let err = root.resolve().expect_err("must fail because choice is unresolved");

    match err {
        BlueprintError::UnresolvedChoice { prompt } => {
            assert_eq!(prompt, "Choose a leaf:");
        }
        other => panic!("expected UnresolvedChoice, got: {other:?}"),
    }
}

// Custom typed visitor for testing scalar choice selection
struct LeafPicker {
    target_index: usize,
    recorded_prompt: Option<String>,
    recorded_options: Vec<String>,
}

impl ChoiceVisitor<Leaf> for LeafPicker {
    fn visit_choice(&mut self, choice: &mut Choice<Leaf>) -> VisitorFlow {
        self.recorded_prompt = Some(choice.prompt().to_string());
        self.recorded_options = choice.option_labels().into_iter().map(String::from).collect();
        choice.select(self.target_index).unwrap();
        VisitorFlow::Break
    }
}

impl ChoiceCollectionVisitor<String> for LeafPicker {
    fn visit_choice_collection(
        &mut self,
        _choice: &mut ChoiceCollection<String>,
    ) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

#[test]
fn test_scalar_choice_selection_and_resolution() {
    let leaf_choice = Choice::new(
        "Choose a leaf:",
        vec![
            ChoiceOption::new("Maple", Leaf::new("Maple", 10)),
            ChoiceOption::new("Pine", Leaf::new("Pine", 20)),
        ],
    );

    let mut branch = Branch::draft("Branch-A", [leaf_choice.into()], vec!["seasonal".into()]);
    assert!(!branch.is_complete());

    // Select second option (Pine, index 1) using typed visitor
    let mut picker = LeafPicker {
        target_index: 1,
        recorded_prompt: None,
        recorded_options: vec![],
    };
    branch.walk_choices(&mut picker);

    assert_eq!(picker.recorded_prompt.as_deref(), Some("Choose a leaf:"));
    assert_eq!(picker.recorded_options, vec!["Maple", "Pine"]);
    assert!(branch.is_complete());

    let final_branch = branch.resolve().expect("resolved successfully");
    assert_eq!(final_branch.leaves, vec![Leaf::new("Pine", 20)]);
}

#[test]
fn test_collection_slot_cardinality_and_validation() {
    let tag_choice = ChoiceCollection::new(
        "Pick exactly 2 tags:",
        Cardinality::Exact(2),
        vec![
            ChoiceOption::new("alpha", "alpha".to_string()),
            ChoiceOption::new("beta", "beta".to_string()),
            ChoiceOption::new("gamma", "gamma".to_string()),
        ],
    );

    let mut slot = SlotCollection::pending(tag_choice);

    // Try selecting 1 option when 2 are required
    let err = slot.select(&[0]).unwrap_err();
    match err {
        BlueprintError::InvalidCardinality { expected, actual, .. } => {
            assert_eq!(expected, Cardinality::Exact(2));
            assert_eq!(actual, 1);
        }
        other => panic!("expected InvalidCardinality, got: {other:?}"),
    }

    // Try duplicate selection
    let err = slot.select(&[0, 0]).unwrap_err();
    assert!(matches!(err, BlueprintError::DuplicateSelection { .. }));

    // Try out of bounds selection
    let err = slot.select(&[0, 5]).unwrap_err();
    assert!(matches!(err, BlueprintError::IndexOutOfBounds { .. }));

    // Valid selection: options 0 and 2 ("alpha", "gamma")
    slot.select(&[0, 2]).unwrap();
    assert!(slot.is_made());

    let resolved = slot.resolve().unwrap();
    assert_eq!(resolved, vec!["alpha", "gamma"]);
}

#[test]
fn test_unmake_choice_preserves_selection() {
    let leaf_choice = Choice::new(
        "Pick a leaf:",
        vec![
            ChoiceOption::new("First", Leaf::new("First", 1)),
            ChoiceOption::new("Second", Leaf::new("Second", 2)),
        ],
    );

    let mut branch = Branch::draft("Branch-History", [leaf_choice.into()], vec![]);

    // 1. Initial pending state: select option 1 (Second)
    let mut picker = LeafPicker {
        target_index: 1,
        recorded_prompt: None,
        recorded_options: vec![],
    };
    branch.walk_choices(&mut picker);
    assert!(branch.is_complete());

    // 2. Unmake choice via step-back
    let unmade = branch.walk_choices(&mut StepBack) == VisitorFlow::Back;
    assert!(unmade);

    // 3. Choice is now pending again, but previous selection is preserved!
    assert!(!branch.is_complete());

    struct HistoryChecker {
        checked: bool,
    }
    impl ChoiceVisitor<Leaf> for HistoryChecker {
        fn visit_choice(&mut self, choice: &mut Choice<Leaf>) -> VisitorFlow {
            assert!(!choice.is_made());
            assert_eq!(choice.selected_index(), Some(1));
            // Change selection to option 0 (First)
            choice.select(0).unwrap();
            self.checked = true;
            VisitorFlow::Break
        }
    }
    impl ChoiceCollectionVisitor<String> for HistoryChecker {
        fn visit_choice_collection(
            &mut self,
            _choice: &mut ChoiceCollection<String>,
        ) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    let mut checker = HistoryChecker { checked: false };
    branch.walk_choices(&mut checker);
    assert!(checker.checked);

    assert!(branch.is_complete());
    let resolved = branch.resolve().unwrap();
    assert_eq!(resolved.leaves, vec![Leaf::new("First", 1)]);
}

#[test]
fn test_transitive_blueprint_and_progressive_disclosure() {
    let leaf_choice_a = Choice::new(
        "Leaf for Branch A:",
        vec![
            ChoiceOption::new("A-1", Leaf::new("A-1", 100)),
            ChoiceOption::new("A-2", Leaf::new("A-2", 200)),
        ],
    );
    let branch_a = Branch::draft("Branch-A", [leaf_choice_a.into()], vec!["tag-a".into()]);

    let leaf_choice_b = Choice::new(
        "Leaf for Branch B:",
        vec![
            ChoiceOption::new("B-1", Leaf::new("B-1", 1000)),
            ChoiceOption::new("B-2", Leaf::new("B-2", 2000)),
        ],
    );
    let branch_b = Branch::draft("Branch-B", [leaf_choice_b.into()], vec!["tag-b".into()]);

    let primary_branch_choice = Choice::new(
        "Choose primary branch:",
        vec![
            ChoiceOption::new("Pick Branch A", branch_a),
            ChoiceOption::new("Pick Branch B", branch_b),
        ],
    );

    let mut root = Root::draft("root-transitive", primary_branch_choice, vec![]);

    // Progressive disclosure:
    // When primary_branch is pending, its inner leaf choice is NOT visible yet!
    struct PromptRecorder {
        prompts: Vec<String>,
    }
    impl ChoiceVisitor<Branch<BranchDraft>> for PromptRecorder {
        fn visit_choice(&mut self, choice: &mut Choice<Branch<BranchDraft>>) -> VisitorFlow {
            self.prompts.push(choice.prompt().to_string());
            VisitorFlow::Continue
        }
    }
    impl ChoiceVisitor<Leaf> for PromptRecorder {
        fn visit_choice(&mut self, choice: &mut Choice<Leaf>) -> VisitorFlow {
            self.prompts.push(choice.prompt().to_string());
            VisitorFlow::Continue
        }
    }
    impl ChoiceCollectionVisitor<String> for PromptRecorder {
        fn visit_choice_collection(
            &mut self,
            choice: &mut ChoiceCollection<String>,
        ) -> VisitorFlow {
            self.prompts.push(choice.prompt().to_string());
            VisitorFlow::Continue
        }
    }

    let mut rec = PromptRecorder { prompts: vec![] };
    root.walk_choices(&mut rec);
    assert_eq!(rec.prompts, vec!["Choose primary branch:"]);

    // Select Branch B (index 1)
    struct BranchSelect(usize);
    impl ChoiceVisitor<Branch<BranchDraft>> for BranchSelect {
        fn visit_choice(&mut self, choice: &mut Choice<Branch<BranchDraft>>) -> VisitorFlow {
            choice.select(self.0).unwrap();
            VisitorFlow::Break
        }
    }
    impl ChoiceVisitor<Leaf> for BranchSelect {
        fn visit_choice(&mut self, _choice: &mut Choice<Leaf>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }
    impl ChoiceCollectionVisitor<String> for BranchSelect {
        fn visit_choice_collection(&mut self, _c: &mut ChoiceCollection<String>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    root.walk_choices(&mut BranchSelect(1));

    // Now, inner choice for Branch B is revealed!
    rec.prompts.clear();
    root.walk_choices(&mut rec);
    assert_eq!(
        rec.prompts,
        vec!["Choose primary branch:", "Leaf for Branch B:"]
    );

    // Select B-2 (index 1) for the leaf
    struct LeafSelect(usize);
    impl ChoiceVisitor<Branch<BranchDraft>> for LeafSelect {
        fn visit_choice(&mut self, _choice: &mut Choice<Branch<BranchDraft>>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }
    impl ChoiceVisitor<Leaf> for LeafSelect {
        fn visit_choice(&mut self, choice: &mut Choice<Leaf>) -> VisitorFlow {
            choice.select(self.0).unwrap();
            VisitorFlow::Break
        }
    }
    impl ChoiceCollectionVisitor<String> for LeafSelect {
        fn visit_choice_collection(&mut self, _c: &mut ChoiceCollection<String>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    root.walk_choices(&mut LeafSelect(1));

    assert!(root.is_complete());
    let resolved = root.resolve().unwrap();
    assert_eq!(resolved.primary_branch.name, "Branch-B");
    assert_eq!(resolved.primary_branch.leaves, vec![Leaf::new("B-2", 2000)]);
}

#[test]
fn test_breadth_first_traversal_order() {
    let mut root = Root::draft(
        "root-bf",
        Choice::new(
            "Root Primary Choice",
            vec![
                ChoiceOption::new(
                    "Branch 1",
                    Branch::draft("B1", [Choice::new("B1 Leaf Choice", vec![ChoiceOption::new("L1", Leaf::new("L1", 1))]).into()], vec![]),
                ),
            ],
        ),
        vec![
            Slot::pending(Choice::new(
                "Root Sibling Branch Choice",
                vec![
                    ChoiceOption::new(
                        "Branch 2",
                        Branch::draft("B2", [Choice::new("B2 Leaf Choice", vec![ChoiceOption::new("L2", Leaf::new("L2", 2))]).into()], vec![]),
                    ),
                ],
            )),
        ],
    );

    // Make both root level choices so their children become visitable
    root.walk_choices(&mut AutoSelectFirst);

    // Now visit all choices and record prompts
    struct PromptRecorder(Vec<String>);
    impl ChoiceVisitor<Branch<BranchDraft>> for PromptRecorder {
        fn visit_choice(&mut self, choice: &mut Choice<Branch<BranchDraft>>) -> VisitorFlow {
            self.0.push(choice.prompt().to_string());
            VisitorFlow::Continue
        }
    }
    impl ChoiceVisitor<Leaf> for PromptRecorder {
        fn visit_choice(&mut self, choice: &mut Choice<Leaf>) -> VisitorFlow {
            self.0.push(choice.prompt().to_string());
            VisitorFlow::Continue
        }
    }
    impl ChoiceCollectionVisitor<String> for PromptRecorder {
        fn visit_choice_collection(&mut self, choice: &mut ChoiceCollection<String>) -> VisitorFlow {
            self.0.push(choice.prompt().to_string());
            VisitorFlow::Continue
        }
    }

    let mut recorder = PromptRecorder(vec![]);
    root.walk_choices(&mut recorder);

    // Pure breadth-first: Root level choices must appear before nested choices!
    assert_eq!(
        recorder.0,
        vec![
            "Root Primary Choice",
            "Root Sibling Branch Choice",
            "B1 Leaf Choice",
            "B2 Leaf Choice",
        ]
    );
}

#[test]
fn test_lifting_final_to_draft_identity() {
    let leaf = Leaf::new("Maple", 123);
    let branch = Branch {
        name: "Branch-1".into(),
        leaves: vec![leaf.clone()],
        tags: vec!["a".into(), "b".into()],
    };
    let root = Root {
        id: "r1".into(),
        primary_branch: branch.clone(),
        branches: vec![branch],
    };

    // Lift from Final to Draft
    let lifted: RootBlueprint = root.clone().into();
    assert!(lifted.is_complete());

    // Resolve back to Final
    let resolved = lifted.resolve().unwrap();
    assert_eq!(resolved, root);
}

#[test]
fn test_serde_roundtrip() {
    let branch = Branch::draft(
        "Oak Branch",
        [Choice::new("Pick leaf:", vec![ChoiceOption::new("L1", Leaf::new("L1", 1))]).into()],
        vec!["hardwood".into()],
    );
    let root = Root::draft("r-serde", branch, vec![]);

    // JSON serialization of Draft
    let json_draft = serde_json::to_string(&root).unwrap();
    let deserialized_draft: RootBlueprint = serde_json::from_str(&json_draft).unwrap();
    assert_eq!(deserialized_draft, root);

    // Make choice and resolve
    let mut root_to_resolve = deserialized_draft;
    root_to_resolve.walk_choices(&mut AutoSelectFirst);
    let final_root = root_to_resolve.resolve().unwrap();

    let json_final = serde_json::to_string(&final_root).unwrap();
    let deserialized_final: Root<RootFinal> = serde_json::from_str(&json_final).unwrap();
    assert_eq!(deserialized_final, final_root);
}

#[test]
fn test_choice_serialization_minimal_footprint() {
    // 1. Scalar choice fresh state
    let scalar_choice = Choice::new(
        "Pick one",
        vec![ChoiceOption::new("A", 10), ChoiceOption::new("B", 20)],
    );
    let scalar_json = serde_json::to_string(&scalar_choice).unwrap();
    assert!(!scalar_json.contains("selected_index"));
    assert!(!scalar_json.contains("is_made"));

    // Deserialization from minimal JSON initializes defaults
    let minimal_scalar: Choice<i32> = serde_json::from_str(r#"{"prompt":"Test","options":[]}"#).unwrap();
    assert_eq!(minimal_scalar.selected_index, None);
    assert!(!minimal_scalar.is_made);

    // 2. Collection choice fresh state
    let coll_choice = ChoiceCollection::new(
        "Pick some",
        Cardinality::AtLeast(1),
        vec![ChoiceOption::new("X", "x".to_string())],
    );
    let coll_json = serde_json::to_string(&coll_choice).unwrap();
    assert!(!coll_json.contains("selected_indices"));
    assert!(!coll_json.contains("is_made"));

    // Deserialization from minimal JSON initializes defaults
    let minimal_coll: ChoiceCollection<String> =
        serde_json::from_str(r#"{"prompt":"Test","cardinality":{"exact":1},"options":[]}"#).unwrap();
    assert!(minimal_coll.selected_indices.is_empty());
    assert!(!minimal_coll.is_made);

    // 3. Made vs Unmade serialization
    let mut made_scalar = scalar_choice;
    made_scalar.select(1).unwrap();
    let made_json = serde_json::to_string(&made_scalar).unwrap();
    assert!(made_json.contains(r#""selected_index":1"#));
    assert!(made_json.contains(r#""is_made":true"#));

    made_scalar.unmake().unwrap();
    let unmade_json = serde_json::to_string(&made_scalar).unwrap();
    assert!(unmade_json.contains(r#""selected_index":1"#));
    assert!(!unmade_json.contains("is_made"), "is_made: false must be omitted");

    let roundtripped_unmade: Choice<i32> = serde_json::from_str(&unmade_json).unwrap();
    assert!(!roundtripped_unmade.is_made, "omitted is_made deserializes as false");
    assert_eq!(roundtripped_unmade.selected_index, Some(1));
}

#[test]
fn test_typed_builder_usage() {
    let branch = Branch::<BranchFinal>::builder()
        .name("BuilderBranch")
        .with_leaf(Leaf::new("L", 5))
        .tags(vec!["fast".into()])
        .build();

    let root = Root::<RootFinal>::builder()
        .id("r-builder")
        .primary_branch(branch.clone())
        .with_branch(branch)
        .build();

    assert_eq!(root.id, "r-builder");
    assert_eq!(root.primary_branch.name, "BuilderBranch");
    assert_eq!(root.primary_branch.leaves, vec![Leaf::new("L", 5)]);
    assert_eq!(root.branches.len(), 1);
}

#[test]
fn test_typed_builder_adding_leaves_and_branches_one_at_a_time() {
    // 1. Build BranchDraft adding one leaf at a time: fixed and choice
    let leaf_choice = Choice::new(
        "Pick a special leaf:",
        vec![
            ChoiceOption::new("Golden", Leaf::new("Golden Leaf", 100)),
            ChoiceOption::new("Silver", Leaf::new("Silver Leaf", 50)),
        ],
    );

    let branch_draft = BranchBlueprint::builder()
        .name("IncrementalBranch")
        .with_leaf(Leaf::new("Regular Leaf", 10))
        .with_leaf_choice(leaf_choice)
        .build();

    assert_eq!(branch_draft.leaves.len(), 2);
    assert!(branch_draft.leaves[0].is_fixed());
    assert!(branch_draft.leaves[1].is_pending());

    // 2. Build RootDraft adding one branch at a time: fixed branch and branch choice
    let branch_choice = Choice::new(
        "Choose an extra branch:",
        vec![
            ChoiceOption::new(
                "Secondary Branch",
                BranchBlueprint::builder()
                    .name("Secondary")
                    .with_leaf(Leaf::new("Sec Leaf", 2))
                    .build(),
            ),
        ],
    );

    let mut root_draft = RootBlueprint::builder()
        .id("root-incremental")
        .primary_branch(branch_draft)
        .with_branch(
            BranchBlueprint::builder()
                .name("Fixed Branch")
                .with_leaf(Leaf::new("Fixed Leaf", 1))
                .build(),
        )
        .with_branch_choice(branch_choice)
        .build();

    assert_eq!(root_draft.branches.len(), 2);
    assert!(root_draft.branches[0].is_fixed());
    assert!(root_draft.branches[1].is_pending());
    assert!(!root_draft.is_complete());

    // Make choices using AutoSelectFirst
    root_draft.walk_choices(&mut AutoSelectFirst);

    assert!(root_draft.is_complete());
    let resolved = root_draft.resolve().expect("resolved");
    assert_eq!(resolved.id, "root-incremental");
    assert_eq!(resolved.primary_branch.leaves.len(), 2);
    assert_eq!(resolved.primary_branch.leaves[1].name, "Golden Leaf");
    assert_eq!(resolved.branches.len(), 2);
    assert_eq!(resolved.branches[1].name, "Secondary");
}

#[test]
fn test_scalar_slot_cannot_have_cardinality_discrepancy() {
    let scalar_choice = Choice::new(
        "Choose one leaf:",
        vec![
            ChoiceOption::new("A", Leaf::new("A", 1)),
            ChoiceOption::new("B", Leaf::new("B", 2)),
        ],
    );

    let slot = Slot::pending(scalar_choice);
    assert_eq!(slot.cardinality(), Cardinality::Exact(1));

    let collection_choice = ChoiceCollection::new(
        "Pick up to 2 tags:",
        Cardinality::UpTo(2),
        vec![
            ChoiceOption::new("t1", "t1".to_string()),
            ChoiceOption::new("t2", "t2".to_string()),
        ],
    );
    let collection_slot = SlotCollection::pending(collection_choice);
    assert_eq!(collection_slot.cardinality(), Cardinality::UpTo(2));
}

// -------------------------------------------------------------
// NEW TESTS: Strongly typed multi-impl visitor & step-back
// -------------------------------------------------------------

struct TypedWizardUI {
    leaf_choices_visited: usize,
    branch_choices_visited: usize,
    tag_choices_visited: usize,
}

impl ChoiceVisitor<Leaf> for TypedWizardUI {
    fn visit_choice(&mut self, choice: &mut Choice<Leaf>) -> VisitorFlow {
        self.leaf_choices_visited += 1;
        // Strongly typed access to Leaf options!
        for opt in choice.options() {
            assert!(opt.value.value >= 0);
        }
        choice.select(0).unwrap();
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<Branch<BranchDraft>> for TypedWizardUI {
    fn visit_choice(&mut self, choice: &mut Choice<Branch<BranchDraft>>) -> VisitorFlow {
        self.branch_choices_visited += 1;
        // Strongly typed access to Branch options!
        for opt in choice.options() {
            assert!(!opt.value.name.is_empty());
        }
        choice.select(0).unwrap();
        VisitorFlow::Continue
    }
}

impl ChoiceCollectionVisitor<String> for TypedWizardUI {
    fn visit_choice_collection(
        &mut self,
        choice: &mut ChoiceCollection<String>,
    ) -> VisitorFlow {
        self.tag_choices_visited += 1;
        choice.select(&[0]).unwrap();
        VisitorFlow::Continue
    }
}

#[test]
fn test_strongly_typed_multi_impl_visitor() {
    let leaf_choice = Choice::new(
        "Pick a leaf",
        vec![ChoiceOption::new("L1", Leaf::new("Leaf1", 10))],
    );
    let tag_choice = ChoiceCollection::new(
        "Pick tags",
        Cardinality::ONE,
        vec![ChoiceOption::new("t1", "Tag1".to_string())],
    );

    let branch = BranchBlueprint {
        name: "Branch 1".into(),
        leaves: vec![Slot::pending(leaf_choice)],
        tags: SlotCollection::pending(tag_choice),
    };

    let branch_choice = Choice::new("Pick branch", vec![ChoiceOption::new("B1", branch)]);

    let mut root = RootBlueprint {
        id: "root-wizard".into(),
        primary_branch: Slot::pending(branch_choice),
        branches: vec![],
    };

    let mut wizard = TypedWizardUI {
        leaf_choices_visited: 0,
        branch_choices_visited: 0,
        tag_choices_visited: 0,
    };

    root.walk_choices(&mut wizard);

    assert_eq!(wizard.branch_choices_visited, 1);
    assert_eq!(wizard.leaf_choices_visited, 1);
    assert_eq!(wizard.tag_choices_visited, 1);
    assert!(root.is_complete());

    let final_root = root.resolve().unwrap();
    assert_eq!(final_root.primary_branch.leaves[0].name, "Leaf1");
    assert_eq!(final_root.primary_branch.tags[0], "Tag1");
}

#[test]
fn test_step_back_navigation_with_selection_preservation() {
    let leaf_choice = Choice::new(
        "Pick a leaf",
        vec![
            ChoiceOption::new("L1", Leaf::new("L1", 1)),
            ChoiceOption::new("L2", Leaf::new("L2", 2)),
        ],
    );

    let branch = BranchBlueprint {
        name: "Branch 1".into(),
        leaves: vec![Slot::pending(leaf_choice)],
        tags: SlotCollection::Fixed(vec![]),
    };

    let branch_choice = Choice::new(
        "Pick branch",
        vec![
            ChoiceOption::new("B1", branch),
        ],
    );

    let mut root = RootBlueprint {
        id: "root-step-back".into(),
        primary_branch: Slot::pending(branch_choice),
        branches: vec![],
    };

    // 1. Make all choices using AutoSelectFirst
    root.walk_choices(&mut AutoSelectFirst);
    assert!(root.is_complete());

    // 2. Step back choice 1 (the leaf choice, which is at level 1)
    let unmade1 = root.walk_choices(&mut StepBack) == VisitorFlow::Back;
    assert!(unmade1);
    assert!(!root.is_complete());

    // Verify leaf choice is unmade, but index 0 remains pre-selected!
    struct CheckSelection {
        leaf_is_made: bool,
        leaf_selected: Option<usize>,
    }
    impl ChoiceVisitor<Branch<BranchDraft>> for CheckSelection {
        fn visit_choice(&mut self, _c: &mut Choice<Branch<BranchDraft>>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }
    impl ChoiceVisitor<Leaf> for CheckSelection {
        fn visit_choice(&mut self, c: &mut Choice<Leaf>) -> VisitorFlow {
            self.leaf_is_made = c.is_made();
            self.leaf_selected = c.selected_index();
            VisitorFlow::Continue
        }
    }
    impl ChoiceCollectionVisitor<String> for CheckSelection {
        fn visit_choice_collection(&mut self, _c: &mut ChoiceCollection<String>) -> VisitorFlow {
            VisitorFlow::Continue
        }
    }

    let mut check = CheckSelection { leaf_is_made: true, leaf_selected: None };
    root.walk_choices(&mut check);
    assert!(!check.leaf_is_made, "leaf choice must be unmade");
    assert_eq!(check.leaf_selected, Some(0), "previous choice must be pre-selected");

    // 3. Step back choice 2 (the branch choice, which is at level 0)
    let unmade2 = root.walk_choices(&mut StepBack) == VisitorFlow::Back;
    assert!(unmade2);

    // 4. No more choices to unmake

    // 5. Re-confirm all choices using confirm_all()
    root.confirm_all();
    assert!(root.is_complete());

    let final_root = root.resolve().unwrap();
    assert_eq!(final_root.primary_branch.leaves[0].name, "L1");
}
