//! S0 spike for `tickets.md`: **a blueprint is the domain type in `Draft` mode**.
//!
//! Mirrors the shape of `fifth_domain` (names, serde attributes, `typed_builder` usage) closely
//! enough that every finding transfers 1:1 to tickets T1–T7.
//!
//! # Requirement 1: a finished value can never hold a choice
//!
//! The same builder call compiles in `Draft` mode …
//!
//! ```
//! use hkd_spike::blueprint::{Choice, ChoiceOption, Draft};
//! use hkd_spike::effect::{Effect, SheetOperation, Vision};
//! use hkd_spike::traits::Trait;
//!
//! let _: Trait<Draft> = Trait::<Draft>::builder()
//!     .title("Superior Darkvision")
//!     .description("Choose your vision.")
//!     .with_effect(Choice::one("Pick one.", [
//!         ChoiceOption::new("Darkvision", Effect::new(SheetOperation::Vision(Vision::Darkvision))),
//!     ]))
//!     .build();
//! ```
//!
//! … and is rejected by the compiler in `Final` mode:
//!
//! ```compile_fail,E0277
//! use hkd_spike::blueprint::{Choice, ChoiceOption, Final};
//! use hkd_spike::effect::{Effect, SheetOperation, Vision};
//! use hkd_spike::traits::Trait;
//!
//! let _: Trait<Final> = Trait::<Final>::builder()
//!     .title("Superior Darkvision")
//!     .description("Choose your vision.")
//!     .with_effect(Choice::one("Pick one.", [
//!         ChoiceOption::single("Darkvision", Effect::new(SheetOperation::Vision(Vision::Darkvision))),
//!     ]))
//!     .build();
//! ```
//!
//! Likewise a `Race<Draft>` accepts finished *and* draft traits …
//!
//! ```
//! use hkd_spike::blueprint::Draft;
//! use hkd_spike::race::Race;
//! use hkd_spike::srd::{darkvision_trait, tool_proficiency};
//!
//! let _: Race<Draft> = Race::<Draft>::builder()
//!     .id("srd/race/dwarf").name("Dwarf").description("…")
//!     .with_trait(darkvision_trait())  // Trait (= Trait<Final>), lifted
//!     .with_trait(tool_proficiency())  // Trait<Draft>
//!     .build();
//! ```
//!
//! … while a `Race` (= `Race<Final>`) rejects the draft trait:
//!
//! ```compile_fail,E0277
//! use hkd_spike::blueprint::Final;
//! use hkd_spike::race::Race;
//! use hkd_spike::srd::{darkvision_trait, tool_proficiency};
//!
//! let _: Race<Final> = Race::<Final>::builder()
//!     .id("srd/race/dwarf").name("Dwarf").description("…")
//!     .with_trait(darkvision_trait())
//!     .with_trait(tool_proficiency())
//!     .build();
//! ```

pub mod blueprint;
pub mod effect;
pub mod race;
pub mod samples;
pub mod srd;
pub mod traits;
mod test;
