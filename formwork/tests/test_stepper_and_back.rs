use formwork::choice::{ChoiceState, MultiChoice, SingleChoice};
use formwork::visitor::{AutoSelectFirst, ChoiceVisitor, VisitorFlow};
use formwork::{Formwork, StepResult, formwork};

// ---------------------------------------------------------------------------
// Stepping Models: Simple Multi-Choice Struct
// ---------------------------------------------------------------------------

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct WizardForm {
    pub title: String,
    #[slot]
    pub step1: String,
    #[slot]
    pub step2: String,
    #[slot]
    pub step3: String,
}

struct SelectIndexVisitor {
    index: usize,
    flow: VisitorFlow,
}

impl SelectIndexVisitor {
    fn new(index: usize, flow: VisitorFlow) -> Self {
        Self { index, flow }
    }
}

impl<T> ChoiceVisitor<T> for SelectIndexVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        let _ = choice.select(self.index);
        let _ = choice.confirm();
        self.flow
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        let _ = choice.select(self.index);
        let _ = choice.confirm();
        self.flow
    }
}

struct InspectChoiceVisitor {
    last_prompt: String,
    last_state_unmade: bool,
    flow: VisitorFlow,
}

impl InspectChoiceVisitor {
    fn new(flow: VisitorFlow) -> Self {
        Self {
            last_prompt: String::new(),
            last_state_unmade: false,
            flow,
        }
    }
}

impl<T> ChoiceVisitor<T> for InspectChoiceVisitor {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        self.last_prompt = choice.prompt().to_string();
        self.last_state_unmade = choice.state().is_unmade();
        self.flow
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        self.last_prompt = choice.prompt().to_string();
        self.last_state_unmade = choice.state().is_unmade();
        self.flow
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_step_one_by_one_forward() {
    let mut form = WizardForm::builder()
        .title("Setup")
        .step1(SingleChoice::new("Step 1", vec!["A1".to_string(), "A2".to_string()]))
        .step2(SingleChoice::new("Step 2", vec!["B1".to_string(), "B2".to_string()]))
        .step3(SingleChoice::new("Step 3", vec!["C1".to_string(), "C2".to_string()]))
        .build();

    assert!(!form.is_complete());

    let mut session = form.session();
    assert_eq!(session.current_step(), 0);
    assert!(!session.is_complete());

    // Step 0
    let mut v0 = SelectIndexVisitor::new(0, VisitorFlow::Continue);
    let res0 = session.step(&mut v0);
    assert_eq!(res0, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Step 1
    let mut v1 = SelectIndexVisitor::new(1, VisitorFlow::Continue);
    let res1 = session.step(&mut v1);
    assert_eq!(res1, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 2);

    // Step 2
    let mut v2 = SelectIndexVisitor::new(0, VisitorFlow::Continue);
    let res2 = session.step(&mut v2);
    assert_eq!(res2, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 3);

    // Step 3: All exhausted -> Complete
    let mut v_extra = InspectChoiceVisitor::new(VisitorFlow::Continue);
    let res3 = session.step(&mut v_extra);
    assert_eq!(res3, StepResult::Complete);
    assert!(session.is_complete());

    // Formwork resolves cleanly
    let resolved = form.resolve().expect("Form should resolve");
    assert_eq!(resolved.step1, "A1");
    assert_eq!(resolved.step2, "B2");
    assert_eq!(resolved.step3, "C1");
}

#[test]
fn test_back_returns_previous_choice() {
    let mut form = WizardForm::builder()
        .title("Setup")
        .step1(SingleChoice::new("Step 1", vec!["A1".to_string(), "A2".to_string()]))
        .step2(SingleChoice::new("Step 2", vec!["B1".to_string(), "B2".to_string()]))
        .step3(SingleChoice::new("Step 3", vec!["C1".to_string(), "C2".to_string()]))
        .build();

    let mut session = form.session();

    // Step 0: confirm A1
    let mut v0 = SelectIndexVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(session.step(&mut v0), StepResult::Choice(VisitorFlow::Continue));

    // Step 1: confirm B1
    let mut v1 = SelectIndexVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(session.step(&mut v1), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 2);

    // Step 2: visitor returns Back!
    let mut v_back = InspectChoiceVisitor::new(VisitorFlow::Back);
    let res_back = session.step(&mut v_back);
    assert_eq!(res_back, StepResult::Choice(VisitorFlow::Back));

    // Cursor rewound to Step 1
    assert_eq!(session.current_step(), 1);

    // Step 1 is presented again on the subsequent step()!
    let mut v_inspect = InspectChoiceVisitor::new(VisitorFlow::Continue);
    let res_step = session.step(&mut v_inspect);
    assert_eq!(res_step, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(v_inspect.last_prompt, "Step 2");
    // Choice was unconfirmed into ChoiceState::UnmadeWithHistory
    assert!(v_inspect.last_state_unmade);
}

#[test]
fn test_back_on_choice_0_noop() {
    let mut form = WizardForm::builder()
        .title("Setup")
        .step1(SingleChoice::new("Step 1", vec!["A1".to_string()]))
        .step2(SingleChoice::new("Step 2", vec!["B1".to_string()]))
        .step3(SingleChoice::new("Step 3", vec!["C1".to_string()]))
        .build();

    let mut session = form.session();
    assert_eq!(session.current_step(), 0);

    // Visitor returns Back on Choice 0
    let mut v_back = InspectChoiceVisitor::new(VisitorFlow::Back);
    let res = session.step(&mut v_back);
    assert_eq!(res, StepResult::AtStart);
    assert_eq!(session.current_step(), 0);

    // Formwork is 100% untouched
    assert!(matches!(
        form.step1.as_choice().unwrap().state,
        ChoiceState::Pending
    ));
}

#[test]
fn test_multiple_backs_in_sequence() {
    let mut form = WizardForm::builder()
        .title("Setup")
        .step1(SingleChoice::new("Step 1", vec!["A1".to_string()]))
        .step2(SingleChoice::new("Step 2", vec!["B1".to_string()]))
        .step3(SingleChoice::new("Step 3", vec!["C1".to_string()]))
        .build();

    let mut session = form.session();

    // Confirm step 0 and step 1
    let mut v0 = SelectIndexVisitor::new(0, VisitorFlow::Continue);
    session.step(&mut v0);
    let mut v1 = SelectIndexVisitor::new(0, VisitorFlow::Continue);
    session.step(&mut v1);
    assert_eq!(session.current_step(), 2);

    // Back from Choice 2 -> Choice 1
    let mut v_back = InspectChoiceVisitor::new(VisitorFlow::Back);
    assert_eq!(session.step(&mut v_back), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 1);

    // Choice 1 presented again, and user clicks Back again!
    let mut v_back2 = InspectChoiceVisitor::new(VisitorFlow::Back);
    assert_eq!(session.step(&mut v_back2), StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // Choice 0 presented again, and user clicks Back again -> AtStart!
    let mut v_back3 = InspectChoiceVisitor::new(VisitorFlow::Back);
    assert_eq!(session.step(&mut v_back3), StepResult::AtStart);
    assert_eq!(session.current_step(), 0);
}

// ---------------------------------------------------------------------------
// Stepping Models: Cascading Hierarchy
// ---------------------------------------------------------------------------

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct ChildModel {
    #[slot]
    pub child_choice: String,
}

#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct ParentBfs {
    #[slot]
    pub parent_choice: String,
    #[slot(cascade)]
    pub child: ChildModel,
}

#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct ParentDfs {
    #[slot]
    pub parent_choice: String,
    #[slot(cascade)]
    pub child: ChildModel,
}

#[test]
fn test_cascading_child_back_rewind() {
    let child = ChildModel::builder()
        .child_choice(SingleChoice::new("Child Choice", vec!["child_v1".to_string()]))
        .build();

    let mut parent = ParentBfs::builder()
        .parent_choice(SingleChoice::new("Parent Choice", vec!["parent_v1".to_string()]))
        .child(child)
        .build();

    let mut session = parent.session();

    // Step 0: Parent Choice confirmed
    let mut v0 = SelectIndexVisitor::new(0, VisitorFlow::Continue);
    assert_eq!(session.step(&mut v0), StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Step 1: Child Choice visited -> user clicks Back
    let mut v_back = InspectChoiceVisitor::new(VisitorFlow::Back);
    let res = session.step(&mut v_back);
    assert_eq!(res, StepResult::Choice(VisitorFlow::Back));
    assert_eq!(session.current_step(), 0);

    // Parent choice was unconfirmed to UnmadeWithHistory
    assert!(matches!(
        session.formwork().parent_choice.as_choice().unwrap().state,
        ChoiceState::UnmadeWithHistory(_)
    ));

    // Next step re-presents Parent Choice!
    let mut v_inspect = InspectChoiceVisitor::new(VisitorFlow::Continue);
    session.step(&mut v_inspect);
    assert_eq!(v_inspect.last_prompt, "Parent Choice");
}

#[test]
fn test_bfs_vs_dfs_session_parity() {
    let child_bfs = ChildModel::builder()
        .child_choice(SingleChoice::new("Child", vec!["c".to_string()]))
        .build();
    let mut bfs = ParentBfs::builder()
        .parent_choice(SingleChoice::new("Parent", vec!["p".to_string()]))
        .child(child_bfs)
        .build();

    let mut bfs_prompts = Vec::new();
    let mut bfs_session = bfs.session();
    while let StepResult::Choice(_) = bfs_session.step(&mut AutoSelectFirst) {
        bfs_prompts.push(bfs_session.current_step());
    }

    let child_dfs = ChildModel::builder()
        .child_choice(SingleChoice::new("Child", vec!["c".to_string()]))
        .build();
    let mut dfs = ParentDfs::builder()
        .parent_choice(SingleChoice::new("Parent", vec!["p".to_string()]))
        .child(child_dfs)
        .build();

    let mut dfs_prompts = Vec::new();
    let mut dfs_session = dfs.session();
    while let StepResult::Choice(_) = dfs_session.step(&mut AutoSelectFirst) {
        dfs_prompts.push(dfs_session.current_step());
    }

    assert_eq!(bfs_prompts, vec![1, 2]);
    assert_eq!(dfs_prompts, vec![1, 2]);
}

#[test]
fn test_existing_walk_choices_regression() {
    let child = ChildModel::builder()
        .child_choice(SingleChoice::new("Child", vec!["c_val".to_string()]))
        .build();
    let mut parent = ParentBfs::builder()
        .parent_choice(SingleChoice::new("Parent", vec!["p_val".to_string()]))
        .child(child)
        .build();

    let flow = parent.walk_choices(&mut AutoSelectFirst);
    assert_eq!(flow, VisitorFlow::Continue);
    assert!(parent.is_complete());

    let resolved = parent.resolve().expect("Should resolve");
    assert_eq!(resolved.parent_choice, "p_val");
    assert_eq!(resolved.child.child_choice, "c_val");
}
