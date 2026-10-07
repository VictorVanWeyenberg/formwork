# Plan: blueprints as *modes* of the domain types (verified design for `problem.md`)

## Problem (restated)

`problem.md` asks for a way to represent "not-yet-chosen" content (blueprints) for a nested domain
(Race → Traits → Effects, Class → Features → Effects, …) such that:

1. the finished domain stays water-tight (a `Race` can never hold a partial trait or a blueprint),
2. no unmaintainable boilerplate unless it is macro-generated from the domain language,
3. patches are as small and as close to their change as possible,
4. the API is writable from plain Rust,
5. everything is strongly typed,
6. **(added)** the JSON of a source is as small as possible and fluently human readable.

Two shapes have been tried in this repo and both hit a wall:

- *Top-level template + patches* (`093ff20`): `Blueberry<Race>` with `RacePatch::AddTrait(Trait)`; every
  option duplicates a whole trait and deeper changes would need path-addressed patches.
- *Lowest-level blueprints* (current `master`): `RaceBlueprint { traits: Vec<Either<Trait, Blueberry<Trait>>> }`
  (`fifth_domain/src/blueprint/race.rs:31`), duplicated for `SubraceBlueprint`
  (`fifth_domain/src/blueprint/subrace.rs:34`), each with its own `Blueprint` impl
  (`race.rs:42-80`, `subrace.rs:45-83`), its own `serde_untagged_vec` module (`race.rs:83`,
  `subrace.rs:93`), a `TraitPatch` enum (`fifth_domain/src/patch/traits.rs:8-9`) and a `ChoiceKind`
  variant per patchable type (`fifth_domain/src/blueprint/choice.rs:9-10`). Every new level
  (Class, ClassLevel, Subclass, Background, Feature) would add ~120 lines of the same code.

## Key insight

The *only* thing that differs between a `Race` and a `RaceBlueprint` is **how list positions are
represented**: in the finished object a position *is* a value; in a blueprint a position is either a
fixed value or a `Choice` over values. Make that representation a **type parameter** ("mode") of the
one and only domain type:

```rust
pub struct Trait<M: Mode = Final> { title, description, effects: Vec<M::Effect> }
pub struct Race <M: Mode = Final> { id, name, description, traits: Vec<M::Trait> }

pub trait Mode { type Effect: Node; type Trait: Node; /* Feature, Item, … later */ }
impl Mode for Final { type Effect = Effect;       type Trait = Trait<Final>;       }
impl Mode for Draft { type Effect = Slot<Effect>; type Trait = Slot<Trait<Draft>>; }

pub enum Slot<T> { Fixed(T), Choose(Choice<T>) }
```

- `Race` (= `Race<Final>`) is exactly today's `Race`; it *cannot* contain a `Slot`/`Choice` — the compiler
  rejects it (requirement 1, proven by a `compile_fail` doc-test).
- `Race<Draft>` **is** the blueprint (`pub type RaceBlueprint = Race<Draft>` keeps the vocabulary). No
  second struct, no `Either`, no `Blueberry`, no `serde_untagged_vec`, no `ChoiceKind`, no patch enum.
- There are **no patches**: a `Choice<T>` sits *at the position it fills* and its options carry complete
  `T` values (an `Effect`, or a whole `Trait<Draft>`). The "delta" is the value itself, distance zero
  (requirement 3). The author never writes a partial value like `Proficiency::Tool()`; scalar variability
  is expressed by enumerating whole values programmatically (`Tools::all().map(option)`).
- Choices nest naturally: an option of `Choice<Trait<Draft>>` may itself contain choices
  (Ability Score Improvement: "+2 to one **or** +1 to two", `fifth_srd/resources/features.json:26-29`).
- One option may yield several values (`ChoiceOption<T>.values: Vec<T>`), so a single pick can add
  e.g. both a resistance and a breath weapon (Draconic Ancestry), and `Exactly(2)` picks yield 2 effects.

### Public API (fifth_domain::blueprint)

```rust
pub trait Node: Clone + PartialEq + Serialize + DeserializeOwned {}   // blanket impl
pub trait Mode: Copy + Debug + PartialEq + 'static { type Effect: Node; type Trait: Node; }
pub struct Final; pub struct Draft;

#[serde(untagged)] pub enum Slot<T> { Choose(Choice<T>), Fixed(T) }
impl<T> From<T>            for Slot<T>            // fixed
impl<T> From<Choice<T>>    for Slot<T>            // pending
impl    From<Trait<Final>> for Slot<Trait<Draft>> // lift a finished helper trait into a draft

pub struct Choice<T>       { cardinality: Cardinality, prompt: String, options: Vec<ChoiceOption<T>>, selected: Vec<String> }
pub struct ChoiceOption<T> { label: String, values: Vec<T> }
pub enum   Cardinality     { Exactly(NonZeroUsize), UpTo(NonZeroUsize), AtLeast(NonZeroUsize) }

/// Object-safe view for UIs: no enum per choosable kind needed.
pub trait ChoicePrompt { fn prompt(&self)->&str; fn cardinality(&self)->&Cardinality; fn labels(&self)->Vec<&str>;
                         fn is_selected(&self, i: usize)->bool; fn select(&mut self, i: usize); fn unselect(&mut self, i: usize);
                         fn is_ok(&self)->bool; }

/// Implemented (derived) for every `X<Draft>`.
pub trait Blueprint: Sized {
    type Final;
    fn resolve(self) -> Result<Self::Final, ChoiceError>;                 // Draft -> Final, flattens picks
    fn visible_choices(&self) -> Vec<&dyn ChoicePrompt>;                  // DFS; descends only into selected options
    fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt>;  // wizard flow, borrow-safe
    fn is_complete(&self) -> bool;
}
pub trait Lift { type Draft; fn lift(self) -> Self::Draft; }             // Final -> Draft (all slots fixed)
```

Builders (`typed_builder`, feature `builders`) get one mutator per slot field:
`with_trait(impl Into<M::Trait>)` — in `Final` mode it accepts `Trait`; in `Draft` mode it accepts
`Trait` (lifted), `Trait<Draft>` or `Choice<Trait<Draft>>`. Same for `with_effect`.

### Author side (fifth_srd) — Dwarf

```rust
pub fn dwarf() -> RaceBlueprint {
    Race::<Draft>::builder().id(dwarf_race_ref()).name("Dwarf").description("…")
        .with_trait(ability_score_increase_trait(AbilityScoreIncrease::racial(2, Ability::Constitution)))
        .with_trait(size_trait(Size::Medium))  /* … unchanged helpers returning plain `Trait` … */
        .with_trait(tool_proficiency())
        .build()
}

fn tool_proficiency() -> Trait<Draft> {
    Trait::<Draft>::builder()
        .title("Tool Proficiency")
        .description("You gain proficiency with the artisan's tools of your choice: smith's tools, brewer's supplies, or mason's tools.")
        .with_effect(Choice::one("Choose one tool to be proficient with.",
            [Tools::SmithTools, Tools::BrewerSupplies, Tools::MasonTools].map(|t|
                ChoiceOption::single(t.tool().name(), Effect::new(SheetOperation::Proficiency(t.reference().into()))))))
        .build()
}
```

### Engine side

```rust
let mut draft = book.race(&dwarf_ref)?;                  // Race<Draft>, cloned from the book
while let Some(c) = draft.next_open_choice_mut() { ui.ask(c); }  // &mut dyn ChoicePrompt
let race: Race = draft.resolve()?;                       // Race<Final>; Err names the unanswered prompt
```

### JSON (requirement 6)

Fixed slots serialize **transparently** (a fixed trait/effect costs nothing extra). A choice is one inline
object whose discriminating key doubles as the cardinality:

```json
{
  "title": "Tool Proficiency",
  "description": "You gain proficiency with the artisan's tools of your choice: …",
  "effects": [
    {
      "choose": 1,
      "prompt": "Choose one tool to be proficient with.",
      "options": {
        "Smith's tools":     { "proficiency": { "tool": "srd/tool/smith_tools" } },
        "Brewer's supplies": { "proficiency": { "tool": "srd/tool/brewer_supplies" } },
        "Mason's tools":     { "proficiency": { "tool": "srd/tool/mason_tools" } }
      }
    }
  ]
}
```

- `"choose": 1 | {"upTo": 2} | {"atLeast": 1}` (`Cardinality`, `Exactly` is an untagged number).
- `options` is an ordered map `label → value | [values]` (custom serde; document order is kept).
- `"selected": ["Smith's tools"]` appears only on an in-progress character, never in a book.
- Optional but recommended: externally tagged `SheetOperation`/`Proficiency`/`DamageModulation`
  (`{"proficiency": {"tool": "…"}}` instead of `{"operation": "proficiency", "argument": {"proficiency_kind": "tool", "proficient_with": "…"}}`),
  and `effects` omitted when empty. Sizes will be measured before/after on the SRD book.

## Requirement checklist (evidence to be produced)

| # | Requirement | Mechanism | Evidence |
|---|-------------|-----------|----------|
| 1 | Tight domain | `Race<Final>` has no `Slot`; `with_trait` on `Final` rejects `Trait<Draft>`/`Choice` | `compile_fail` doc-test (E0277) |
| 2 | No hand boilerplate | one struct per concept; per-struct `Blueprint`/`Lift` impls generated by `#[derive(Blueprint)]`; `Mode` grows by one line per kind | derive in spike + line counts before/after |
| 3 | Minimal, local patches | no patches; `Choice` sits at the position it fills; options carry whole values | resolve tests, JSON sample |
| 4 | Rust-writable API | builders + `Into` conversions; helpers keep returning plain `Trait` | ported `dwarf.rs`/`hill_dwarf.rs` compile |
| 5 | Type safety | `Choice<Effect>` vs `Choice<Trait<Draft>>` are distinct types; no stringly maps | compiles; wrong kind is a type error |
| 6 | Small, readable JSON | transparent `Fixed`, inline `choose`, label-keyed options, externally tagged effects | measured byte counts + pretty sample |

## Alternatives considered (and why not)

- **Status quo** (`RaceBlueprint`/`SubraceBlueprint` + `Blueberry<Trait>` + `TraitPatch`): O(levels) duplicated
  structs and serde helpers; `ChoiceKind` grows per kind; cannot nest; a pick cannot span kinds.
- **Top-level template + path-addressed patches**: patches become `traits[3].effects += …`; brittle, far from
  the change; templates must be complete values (the `Proficiency::Tool()` problem).
- **Proc-macro generating a parallel `XBlueprint` struct per `X`**: works, but doubles nominal types, the
  macro must rewrite field types/builders/serde, and errors point into generated code. The mode parameter
  lets the *compiler* do this generation; only the trivial `resolve`/`lift` bodies need a derive.
- **GAT `type Slot<T>`** instead of one associated type per kind: fewer lines in `Mode`, but serde/derive
  bounds on `M::Slot<Trait<M>>` are much harder to reason about; may be revisited later.
- **Closures / `serde_json::Value` + JSON-patch / `HashMap<String, _>` answers**: not serializable, not typed,
  or far from the change — all previously rejected in this repo's history (`093ff20`, `dc81f25`).

## Pre-existing bugs tightly coupled to the change (T0, T4)

- `Cardinality::good_len` is inverted for `UpTo`/`AtLeast` (`fifth_domain/src/blueprint/choice.rs:95-96`:
  `exp.get() <= size` / `exp.get() >= size`). Covered by a test.
- `ChoiceKind::Trait` holds `&Choice`, not `&mut` (`choice.rs:10,13`), so `choices_mut` cannot select anything.

## Delivery

Superseded the earlier "temporary port, then restore" protocol: the work is split into small, independently
mergeable tickets in **`tickets.md`** (same folder); progress is tracked in `todos.md`. Each ticket leaves the workspace compiling and tested;
`fifth_srd`/`fifth_engine` build after every step; legacy code is removed only once it has no users (T4).
Order: T0 → T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8 (S0 optional spike before T2; T6 independent).
Work pauses after each ticket for review.

## Known limitations / follow-ups (out of scope)

- Option sets are static lists; "any language from any book" or "a skill you are already proficient in"
  need a query/constraint extension (`Options::Query`, engine-evaluated `requires`).
- Class progression: `ClassLevel<Draft>` resolves per level as the character levels up.
- Descriptions do not yet reflect the chosen option.

## Todos

One todo per ticket in `todos.md` (ids `S0`, `T0` … `T8`); details in `tickets.md`.
