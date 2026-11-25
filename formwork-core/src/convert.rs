//! Internal slot conversion plumbing traits used by `#[formwork]`-generated code.

use crate::choice::{Freeform, SingleChoice};
use crate::slot::{MultiSlot, Slot};

/// Internal plumbing: converts a value, choice, or slot into a [`Slot<W, D, C>`].
#[doc(hidden)]
pub trait IntoSlot<W, D, C> {
    fn into_slot(self) -> Slot<W, D, C>;
}

impl<W, D, C> IntoSlot<W, D, C> for Slot<W, D, C> {
    fn into_slot(self) -> Slot<W, D, C> {
        self
    }
}

impl<W, D> IntoSlot<W, D, SingleChoice<W>> for SingleChoice<W> {
    fn into_slot(self) -> Slot<W, D, SingleChoice<W>> {
        Slot::Choose(self)
    }
}

impl<W, D> IntoSlot<W, D, Freeform<W>> for Freeform<W> {
    fn into_slot(self) -> Slot<W, D, Freeform<W>> {
        Slot::Choose(self)
    }
}

impl<W, D, C> IntoSlot<W, D, C> for W {
    fn into_slot(self) -> Slot<W, D, C> {
        Slot::FixedWet(self)
    }
}

/// Internal plumbing: converts a value, choice, slot, or `Option` thereof into an `Option<Slot<W, D, C>>`.
#[doc(hidden)]
pub trait IntoOptionSlot<W, D, C> {
    fn into_option_slot(self) -> Option<Slot<W, D, C>>;
}

impl<W, D, C> IntoOptionSlot<W, D, C> for Slot<W, D, C> {
    fn into_option_slot(self) -> Option<Slot<W, D, C>> {
        Some(self)
    }
}

impl<W, D> IntoOptionSlot<W, D, SingleChoice<W>> for SingleChoice<W> {
    fn into_option_slot(self) -> Option<Slot<W, D, SingleChoice<W>>> {
        Some(Slot::Choose(self))
    }
}

impl<W, D> IntoOptionSlot<W, D, Freeform<W>> for Freeform<W> {
    fn into_option_slot(self) -> Option<Slot<W, D, Freeform<W>>> {
        Some(Slot::Choose(self))
    }
}

impl<W, D, C> IntoOptionSlot<W, D, C> for W {
    fn into_option_slot(self) -> Option<Slot<W, D, C>> {
        Some(Slot::FixedWet(self))
    }
}

impl<W, D, C> IntoOptionSlot<W, D, C> for Option<Slot<W, D, C>> {
    fn into_option_slot(self) -> Option<Slot<W, D, C>> {
        self
    }
}

impl<W, D> IntoOptionSlot<W, D, SingleChoice<W>> for Option<SingleChoice<W>> {
    fn into_option_slot(self) -> Option<Slot<W, D, SingleChoice<W>>> {
        self.map(Slot::Choose)
    }
}

impl<W, D> IntoOptionSlot<W, D, Freeform<W>> for Option<Freeform<W>> {
    fn into_option_slot(self) -> Option<Slot<W, D, Freeform<W>>> {
        self.map(Slot::Choose)
    }
}

impl<W, D, C> IntoOptionSlot<W, D, C> for Option<W> {
    fn into_option_slot(self) -> Option<Slot<W, D, C>> {
        self.map(Slot::FixedWet)
    }
}

/// Internal plumbing: converts an iterator of wet or dry elements into a [`MultiSlot<W, D>`].
#[doc(hidden)]
pub trait CascadePickElement<W, D>: Sized {
    fn into_multi_slot<I: IntoIterator<Item = Self>>(items: I) -> MultiSlot<W, D>;
}

impl<W, D> CascadePickElement<W, D> for W {
    fn into_multi_slot<I: IntoIterator<Item = Self>>(items: I) -> MultiSlot<W, D> {
        Slot::FixedWet(items.into_iter().collect())
    }
}

/// Internal plumbing: converts a value or `Option` thereof into an `Option<D>` for dry option slots.
#[doc(hidden)]
pub trait IntoDryOption<D> {
    fn into_dry_option(self) -> Option<D>;
}

impl<D> IntoDryOption<D> for D {
    fn into_dry_option(self) -> Option<D> {
        Some(self)
    }
}

impl<D> IntoDryOption<D> for Option<D> {
    fn into_dry_option(self) -> Option<D> {
        self
    }
}
