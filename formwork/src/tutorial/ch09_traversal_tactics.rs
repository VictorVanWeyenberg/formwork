//! # Chapter 9: The Weave of Sight (Traversal Strategies & Flow Control)
//!
//! Before commissioning a vessel for flight, the High Inspector inspects the blueprint
//! using a **Divining Lens**. Inspection can follow two distinct protocols: inspecting all
//! broad vessel decks first before entering compartments (**BFS**), or thoroughly inspecting
//! a compartment down to its deepest conduits before moving to the next deck (**DFS**).
//!
//! ## Concept: Traversal Strategies (`traverse = "bfs"` vs `"dfs"`)
//!
//! When declaring a formwork struct with cascading children, you can control the order in
//! which [`walk_choices`](crate::Formwork::walk_choices) traverses choices using the `traverse` attribute:
//!
//! - `#[formwork(builder, traverse = "bfs")]` (**Breadth-First Search, default**):
//!   Visits all choices at the current struct level first, then descends level-by-level into child formworks.
//!   *Best for*: Guided wizards, top-down configuration where parent context guides child options.
//! - `#[formwork(builder, traverse = "dfs")]` (**Depth-First Search**):
//!   Immediately descends through cascading children down to the leaf choices of a branch before
//!   returning to sibling fields on the parent.
//!   *Best for*: Deep component configuration, fully fleshing out a subsystem before moving on.
//!
//! ### Interactive Stepping and Rewinding (`VisitorFlow::Back`)
//!
//! Beyond batch walking, `formwork` supports fine-grained interactive traversal via
//! [`FormworkSession`](crate::FormworkSession):
//! - [`session.step(&mut visitor)`](crate::FormworkSession::step): Steps one choice at a time.
//! - [`VisitorFlow::Continue`]: Advance to the next choice.
//! - [`VisitorFlow::Break`]: Halt traversal early.
//! - [`VisitorFlow::Back`]: **Rewind!** Unmakes the previous choice
//!   and steps the cursor backwards, preserving selection history in `ChoiceState::UnmadeWithHistory`.
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::tutorial::ch09_traversal_tactics::{run, PromptRecorder};
//!
//! let (bfs_order, dfs_order) = run();
//!
//! // BFS visits root-level choices first across the breadth, then child choices:
//! assert_eq!(bfs_order, vec!["Vessel Stance", "Vessel Horn", "Compartment Valve"]);
//!
//! // DFS descends down into the child compartment before moving to sibling fields:
//! assert_eq!(dfs_order, vec!["Vessel Stance", "Compartment Valve", "Vessel Horn"]);
//! ```

use crate::choice::{MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, StepResult, formwork};

/// A nested vessel compartment.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SubCompartment {
    /// Compartment name.
    pub label: String,
    /// Internal valve setting (choice).
    #[slot]
    pub valve: String,
}

/// A vessel inspected with Breadth-First Search (`traverse = "bfs"`).
#[formwork(builder, traverse = "bfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct VesselBfs {
    /// Initial root choice.
    #[slot]
    pub stance: String,
    /// Cascading child compartment.
    #[slot(cascade)]
    pub compartment: SubCompartment,
    /// Sibling root choice following the cascade.
    #[slot]
    pub horn: String,
}

/// An identical vessel inspected with Depth-First Search (`traverse = "dfs"`).
#[formwork(builder, traverse = "dfs")]
#[derive(Clone, Debug, PartialEq)]
pub struct VesselDfs {
    /// Initial root choice.
    #[slot]
    pub stance: String,
    /// Cascading child compartment.
    #[slot(cascade)]
    pub compartment: SubCompartment,
    /// Sibling root choice following the cascade.
    #[slot]
    pub horn: String,
}

/// A visitor that records choice prompts in the exact order they are visited.
#[derive(Default)]
pub struct PromptRecorder {
    /// Ordered list of prompt strings encountered during traversal.
    pub visited_prompts: Vec<String>,
}

impl<T> ChoiceVisitor<T> for PromptRecorder {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        self.visited_prompts.push(choice.prompt().to_string());
        if !choice.is_made() && !choice.options().is_empty() {
            let _ = choice.select(0);
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        self.visited_prompts.push(choice.prompt().to_string());
        VisitorFlow::Continue
    }
}

/// Runs the Chapter 9 tutorial, proving the observable differences between BFS and DFS traversal,
/// and demonstrating interactive stepping and history rewinding with `VisitorFlow::Back`.
///
/// Returns a tuple containing `(bfs_prompt_order, dfs_prompt_order)`.
pub fn run() -> (Vec<String>, Vec<String>) {
    // 1. BFS Traversal Order Verification
    let sub_bfs = SubCompartment::builder()
        .label("Aux-Bay".to_string())
        .valve(SingleChoice::new("Compartment Valve", vec!["Open".to_string()]))
        .build();

    let mut vessel_bfs = VesselBfs::builder()
        .stance(SingleChoice::new("Vessel Stance", vec!["Cruising".to_string()]))
        .compartment(sub_bfs)
        .horn(SingleChoice::new("Vessel Horn", vec!["Resonant".to_string()]))
        .build();

    let mut bfs_recorder = PromptRecorder::default();
    vessel_bfs.walk_choices(&mut bfs_recorder);

    // Root level choices visited first, then depth 1:
    assert_eq!(
        bfs_recorder.visited_prompts,
        vec!["Vessel Stance", "Vessel Horn", "Compartment Valve"]
    );

    // 2. DFS Traversal Order Verification
    let sub_dfs = SubCompartment::builder()
        .label("Aux-Bay".to_string())
        .valve(SingleChoice::new("Compartment Valve", vec!["Open".to_string()]))
        .build();

    let mut vessel_dfs = VesselDfs::builder()
        .stance(SingleChoice::new("Vessel Stance", vec!["Cruising".to_string()]))
        .compartment(sub_dfs)
        .horn(SingleChoice::new("Vessel Horn", vec!["Resonant".to_string()]))
        .build();

    let mut dfs_recorder = PromptRecorder::default();
    vessel_dfs.walk_choices(&mut dfs_recorder);

    // Dives down into cascading branch before moving to sibling horn:
    assert_eq!(
        dfs_recorder.visited_prompts,
        vec!["Vessel Stance", "Compartment Valve", "Vessel Horn"]
    );

    // 3. FormworkSession: Step and Back Rewind Verification
    let mut session = vessel_bfs.session();
    assert_eq!(session.current_step(), 0);

    // Step 0: visit stance
    struct StepVisitor(VisitorFlow);
    impl<T> ChoiceVisitor<T> for StepVisitor {
        fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
            let _ = choice.select(0);
            let _ = choice.confirm();
            self.0
        }
        fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
            self.0
        }
    }

    let res0 = session.step(&mut StepVisitor(VisitorFlow::Continue));
    assert_eq!(res0, StepResult::Choice(VisitorFlow::Continue));
    assert_eq!(session.current_step(), 1);

    // Step 1: user decides to backtrack! Return VisitorFlow::Back
    let res_back = session.step(&mut StepVisitor(VisitorFlow::Back));
    assert_eq!(res_back, StepResult::Choice(VisitorFlow::Back));
    // Cursor rewinds to step 0
    assert_eq!(session.current_step(), 0);

    (bfs_recorder.visited_prompts, dfs_recorder.visited_prompts)
}
