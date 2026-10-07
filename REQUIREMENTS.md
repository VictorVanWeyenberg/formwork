# Architecture & Design Requirements

This document outlines the strict functional and architectural requirements for this crate. It serves as a persistent reference to ensure that future developments and refactorings do not mistakenly identify these intentional design choices as "over-engineering".

## 1. Multi-Value Fields vs. Choice Cardinality
**Requirement**: The system must support mixed multi-value collections (e.g., a list of items where some are fixed and some are choices, such as D&D Dwarf traits) alongside high-cardinality multi-select choices (e.g., a single choice where the user must pick exactly 3 skills from a list of N>3 options to be proficient with, resolving to a `Vec<T>`).
**Implementation**: This is deliberately achieved using `Vec<Slot<T>>` for multi-value fields combined with `Choice<T>` maintaining its own cardinality rules (e.g., `UpTo(N)` or `Exact(N)`). 
- Using `Vec<Slot<T>>` allows the user to supply multiple distinct slots (both fixed and choice) in a single array.
- Using Choice-level cardinality enables a single choice slot to independently resolve to multiple selected items, seamlessly flattening into the final `Vec<T>`.
- *Do not attempt to merge these layers or reduce `Choice` to always picking a single item*, as it would break support for multi-select choices like the Class Skills scenario.

## 2. Choice History and Reversible State
**Requirement**: The system must allow users to step backward in a wizard-like UI flow and unmake choices without losing their intermediate selections.
**Implementation**: `ChoiceState` includes an `UnmadeWithHistory(Vec<usize>)` variant. 
- When a choice is unmade, its previous selections are preserved in this history state.
- *Do not attempt to replace this with a simple boolean flag*, as discarding selection history violates the requirement for non-destructive backwards navigation.

## 3. Strong Compile-Time Guarantees (Typestate Pattern)
**Requirement**: A finalized blueprint must be structurally guaranteed to contain zero pending choices. Accessing fields on a finalized blueprint should not require `unwrap()` or pattern matching to bypass "Draft" wrappers.
**Implementation**: Generic typestates (e.g., `Branch<M: BranchMode>`) are used. 
- In `Draft` mode, fields are wrapped in `Slot<T>` or `Vec<Slot<T>>`. 
- In `Final` mode, fields are strictly `T` or `Vec<T>`. 
- *Do not attempt to merge Draft and Final into a single struct with optional/enum fields.* A little boilerplate (like `PushLeafChoice` constraints for builders) is a necessary trade-off to ensure zero-cost abstractions and flawless compile-time safety for finalized structures.

## 4. Progressive Disclosure via Visitor
**Requirement**: Nested choices must remain hidden until their parent trait/branch is actually selected. 
**Implementation**: The `Walkable` traversal delegates level-by-level depth tracking.
- The `depth` parameter explicitly controls the breadth-first expansion.
- Choices at deeper levels are only exposed if the parent structure has been resolved/selected and the visitor progresses to that depth.
- *Do not attempt to flatten the traversal or unconditionally recurse into children at `depth == 0`*, as it breaks the progressive disclosure requirement.
