# Formwork

Higher-Kinded Data (HKD) formwork architecture for Rust.

Separates fluid, choice-bearing in-progress data structures (**wet formworks**) from cured, watertight domain models (**dry models**).

---

## The Problem

Systems with interactive, multi-step, or deferred configuration workflows (such as character creators, product builders, wizard forms, and tax engines) require two incompatible representations of the same domain data:

1. **In-Progress State**: Data is incomplete and mutable. Fields may contain pending choices, candidate pools with cardinality constraints, unconfirmed user inputs, or nested sub-configurations. In this state, an application needs to query open choices, inspect candidates, walk or step through decision trees, and roll back selections without losing input history.
2. **Cured Domain State**: Once all decisions are finalized, business logic (simulation, invoicing, combat engines, persistence) requires rigid, fully validated types. Fields must not contain runtime `Option` wrappers for required data, choice enums, or defensive unwraps.

### Traditional Workarounds and Their Failure Modes

* **Manual Struct Duplication (`Draft` vs `Final`)**:
  Maintaining parallel structs (e.g. `CharacterDraft` and `Character`) causes structural drift, requires verbose manual mapping code, and violates DRY.
* **Over-reliance on `Option<T>` or Choice Enums in Domain Models**:
  Polluting the final domain model with `Option<T>` or custom choice state pushes completeness checks to runtime, forcing callers across the codebase to handle invalid or unselected states defensively.

---

## The Solution

`formwork` uses Higher-Kinded Data (HKD) parameterized by internal mode markers. You define your domain model once with `#[formwork]`. The macro expands this definition into:

* **Dry Domain Model (`StructName`)**: The concrete, watertight struct. Slots are unwrapped into their concrete types (`T`, `Vec<T>`, `Option<T>`).
* **Wet Formwork (`StructNameFormwork`)**: The mutable mold implementing `Formwork`. Slots are wrapped in stateful containers (`SingleSlot`, `MultiSlot`, `FreeformSlot`) that accept concrete values or pending choice structures.

Transitioning from wet to dry is explicit: calling `.resolve()` consumes the wet formwork and produces `Result<StructName, FormworkError>`. If any required slot is unconfirmed or invalid, resolution fails with a typed error.

---

## Features

### Dual-Mode Generation
* Single struct definition generates both the dry model (`StructName`) and the wet formwork (`StructNameFormwork`).
* Invariant fields without attributes remain identical across both modes.

### Slot Types and Cardinality
Configured via field attributes on `#[formwork]` structs:

| Attribute | Field Type | Wet Slot Type | Dry Type | Description |
| :--- | :--- | :--- | :--- | :--- |
| *(None)* | `T` | `T` | `T` | Invariant field; known upfront and identical in both modes. |
| `#[slot]` | `T` | `SingleSlot<T, T>` | `T` | Scalar slot holding a value or `SingleChoice<T>`. |
| `#[slot(option)]` | `Option<T>` | `Option<SingleSlot<T, T>>` | `Option<T>` | Optional slot holding an optional value or choice. |
| `#[slot(collection)]` | `Vec<T>` | `Vec<SingleSlot<T, T>>` | `Vec<T>` | Ordered collection where elements can be concrete values or choices. |
| `#[slot(pick)]` | `Vec<T>` | `MultiSlot<T, T>` | `Vec<T>` | Candidate pool selection governed by a `Cardinality` constraint. |
| `#[slot(freeform)]` | `T` | `FreeformSlot<T, T>` | `T` | Unconstrained input or staged default via `Freeform<T>`. |
| `#[slot(freeform, option)]` | `Option<T>` | `Option<FreeformSlot<T, T>>` | `Option<T>` | Optional freeform slot. |
| `#[slot(freeform, collection)]` | `Vec<T>` | `Vec<FreeformSlot<T, T>>` | `Vec<T>` | Collection of freeform slots. |

### Cascading Hierarchies
Add the `cascade` flag (e.g. `#[slot(cascade)]`, `#[slot(option, cascade)]`, `#[slot(collection, cascade)]`, `#[slot(pick, cascade)]`) to nest child formworks inside a parent. Resolving or traversing the parent recursively resolves and traverses all child formworks.

### Choice Lifecycle & Cardinality Constraints
* **`SingleChoice<T>`**: Requires selecting and confirming exactly one candidate.
* **`MultiChoice<T>`**: Governed by `Cardinality`:
  * `Cardinality::Exact(n)`
  * `Cardinality::UpTo(n)`
  * `Cardinality::AtLeast(n)`
* **`Freeform<T>`**: Handles user-provided values with optional staged candidates.
* **Two-Phase Commit**: Selections are staged via `.select()` and committed via `.confirm()`, enabling inspection and rollback without destroying previous state.

### Traversal and Visitor Pattern
* **`Formwork::walk_choices`**: Traverses all pending choices using either breadth-first (`traverse = "bfs"`, default) or depth-first (`traverse = "dfs"`).
* **Built-in Visitors**:
  * `AutoSelectFirst`: Selects index 0 and confirms every visited choice.
  * `ConfirmAll`: Confirms currently staged choices.
  * `UnmakeAll`: Resets confirmed choices back to unmade state.
* **Custom Visitors**: Implement `ChoiceVisitor<T>` with `VisitorFlow` (`Continue`, `Break`, `Back`) for custom interactive or automated traversal.

### Interactive Stepper Sessions
* `FormworkSession`: Wraps a formwork to drive choices step-by-step. Supports forward stepping, inspection, and rewinding (`StepResult::Choice`, `StepResult::Complete`, `StepResult::AtStart`).

### Type-Safe Builders
* `#[formwork(builder)]`: Generates compile-time checked builders for both wet (`StructNameFormwork::builder()`) and dry (`StructName::builder()`) types.
* Generates `.with_<field>(...)` mutator methods for collection and pick slots to append items individually.

### Serialization
* Full `serde` integration for serializing in-progress formworks (persisting open choices, candidate pools, and selection states) as well as cured dry models.

---

## Example

```rust
use formwork::choice::{Cardinality, MultiChoice, SingleChoice};
use formwork::visitor::AutoSelectFirst;
use formwork::{formwork, Formwork};

#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct Character {
    pub name: String,

    #[slot]
    pub class: String,

    #[slot(collection)]
    pub inventory: Vec<String>,

    #[slot(pick)]
    pub proficiencies: Vec<String>,
}

fn main() {
    // 1. Build a wet formwork with pending choices
    let mut wet = CharacterFormwork::builder()
        .name("Conan".to_string())
        .class(SingleChoice::new(
            "Select Class",
            vec!["Barbarian".to_string(), "Fighter".to_string()],
        ))
        .with_inventory("Rations (5 days)".to_string())
        .with_inventory(SingleChoice::new(
            "Starting Weapon",
            vec!["Greataxe".to_string(), "Longsword".to_string()],
        ))
        .proficiencies(
            MultiChoice::new(
                "Select 2 Proficiencies",
                vec![
                    "Athletics".to_string(),
                    "Survival".to_string(),
                    "Intimidation".to_string(),
                ],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )
        .build();

    // The formwork is incomplete while choices remain unconfirmed
    assert!(!wet.is_complete());

    // 2. Walk and resolve pending choices (e.g. using AutoSelectFirst)
    wet.walk_choices(&mut AutoSelectFirst);
    assert!(wet.is_complete());

    // 3. Resolve the wet formwork into the rigid dry domain model
    let character: Character = wet.resolve().expect("all choices confirmed");

    assert_eq!(character.name, "Conan");
    assert_eq!(character.class, "Barbarian");
    assert_eq!(
        character.inventory,
        vec!["Rations (5 days)", "Greataxe"]
    );
    assert_eq!(
        character.proficiencies,
        vec!["Athletics", "Survival"]
    );
}
```

---

## Crate Hierarchy

* **`formwork`**: Top-level facade crate. Re-exports `formwork-core`, `formwork-macro`, and builder utilities.
* **`formwork-core`**: Core runtime traits (`Formwork`), slot types, choice state machines, and visitor implementations.
* **`formwork-macro`**: Procedural macro implementing `#[formwork]`.
