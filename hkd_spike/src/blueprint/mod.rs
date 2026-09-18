//! The whole "blueprint" machinery: modes, slots, choices and the [`Blueprint`] trait.

mod choice;
mod mode;
mod prompt;
mod slot;

pub use choice::*;
pub use mode::*;
pub use prompt::*;
pub use slot::*;

/// `#[derive(Blueprint)]` for `struct X<M: Mode> { .., slots: Vec<M::Kind>, .. }` (see `spike_derive`).
/// Same name as the trait below, different namespace — exactly like `serde::Serialize`.
pub use spike_derive::Blueprint;

/// Implemented (derived) for every `X<Draft>`.
pub trait Blueprint: Sized {
    /// The finished, water-tight counterpart: `X<Final>`.
    type Final;

    /// Consume the draft. Fails on the first choice whose selection does not satisfy its cardinality.
    fn resolve(self) -> Result<Self::Final, ChoiceError>;

    /// Every choice a UI should currently show, depth-first. Descends only into *selected* options,
    /// so nested choices are disclosed progressively.
    fn visible_choices(&self) -> Vec<&dyn ChoicePrompt>;

    /// The first visible choice that still blocks [`Blueprint::resolve`] (wizard flow).
    fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt>;

    fn is_complete(&self) -> bool {
        self.visible_choices().iter().all(|choice| choice.is_ok())
    }
}
