# Tickets — "blueprint = domain type in `Draft` mode"

Companion to `plan.md` (design rationale). This file splits the delivery into small, independently
mergeable tickets. Ground rules for every ticket:

- One ticket = one PR. After each ticket `cargo build --workspace`, `cargo test --workspace` and
  `cargo test -p fifth_domain --features builders` (doc-tests that use builders) are green, with no new warnings.
  On *this* Windows box (no MSVC linker; the repo is normally built on the Linux host) use
  `cargo +stable-x86_64-pc-windows-gnu … --workspace --exclude fifth_macros` (proc-macro *unit-test* harnesses
  need mingw `dlltool.exe`), with `CARGO_HTTP_CHECK_REVOKE=false` for crates.io — see `spike/FINDINGS.md`.
- No ticket depends on a *later* ticket; `fifth_srd` and `fifth_engine` compile after every ticket.
- Legacy code (`Blueberry`, `Patchable`, `TraitPatch`, `RaceBlueprint` struct, …) is kept alive until it has
  zero users, then removed in one dedicated ticket.
- Diff size is a rough line count of the change (S < 50, M 50–200, L > 200) — not an effort estimate.
- Work pauses after each ticket for review.

## Overview

| # | Title | Depends on | Diff | Serves requirement |
|---|-------|------------|------|--------------------|
| S0 | ✅ **done** — throwaway spike outside the repo (`spike/FINDINGS.md`) | — | — | de-risked T2, T5, T6, T7 |
| T0 | Fix `Cardinality::good_len` | — | S | correctness |
| T1 | Add `Mode`/`Final`/`Draft`, `Slot<T>`, `ChoicePrompt` (additive) | — | M | 5 |
| T2 | `Trait<M: Mode = Final>` + new `Blueprint` trait and the `From<X<Final>>` lift | T1 | M | 1, 3, 4, 5 |
| T3 | `Race<M>`/`Subrace<M>`; `RaceBlueprint` = `Race<Draft>`; Dwarf on the new API | T2 | L | 1, 2, 3, 4 |
| T4 | Remove legacy blueprint machinery | T3 | M (deletions) | 2 |
| T5 | Compact, readable JSON for choices | T1 (best after T4) | M | 6 |
| T6 | Compact, readable JSON for effects *(independent)* | — | S | 6 |
| T7 | `fifth_derive`: `#[derive(Blueprint)]` | T3 (best after T4) | L | 2 |
| T8 | Prove it scales: `Feature<M>`, `ClassLevel<M>` | T7 | M | 2 |
| T9 | Backlog (not scheduled) | — | — | — |

Recommended order: **T0 → T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8** (S0 anywhere before T2; T6 anywhere).

---

## S0 — ✅ done: throwaway spike outside the repository

**Goal.** Answer the compiler/serde questions the design hinges on *before* touching `x:\fifth`.

**Where.** Kept in `spike/` (copied from the session folder, build artifacts stripped), never merged. Full write-up: `spike/FINDINGS.md`.
Result: `cargo test -p hkd_spike` → 18 integration tests + 4 doc-tests green (2 `compile_fail` proofs of
requirement 1); workspace check warning-free.

**Verdicts.**
1. Derives on `struct X<M: Mode> { .., Vec<M::Kind> }` — **works**; serde infers no bound on `M` with the
   shorthand spelling. With the fully-qualified `Vec<<M as Mode>::Kind>` (needed for 2.) it infers
   `M: Serialize + Deserialize + Default` → **`#[serde(bound = "")]`** on those structs.
2. `typed_builder` mutators over `M::Kind` — **fails** (typed-builder 0.23.2, E0220 on any
   `#[builder(via_mutators)]` field typed `Vec<M::Kind>`, mutator or not). **Fallback found:** spell the field
   `Vec<<M as Mode>::Kind>` and the mutator argument `impl Into<<M as Mode>::Kind>`; builders remain usable in
   both modes with `M` inferred from the return type (no turbofish in `fn tool_proficiency() -> Trait<Draft>`).
3. `From` coherence + inference through `impl Into<..>` (incl. lifting a finished `Trait` into a
   `Choice<Trait<Draft>>` option) — **works**, no fallback.
4. `#[serde(flatten)]` over externally tagged `SheetOperation` — **works** for 13 shapes, also through the
   untagged `Slot` and the options map. No fallback.
5. Label-keyed options map — **works**, document order kept, duplicates rejected; needs
   `#[serde(bound(..))]` on the `with`-field (serde infers none for `with` fields).
6. `#[derive(Blueprint)]` — **works** (~150 lines), both field spellings, declaration order.

Extra: `#[serde(untagged)]` variants must be declared *last* (`Cardinality`); no `Debug` needed on domain types.

---

## T0 — Fix `Cardinality::good_len`

**Goal.** Correct a latent bug that every later ticket would build on.

**Current state.** `fifth_domain/src/blueprint/choice.rs:91-98` — the comparisons for `UpTo` and `AtLeast`
are inverted (`choice.rs:95`: `Cardinality::UpTo(exp) => exp.get() <= size`; `choice.rs:96`:
`Cardinality::AtLeast(exp) => exp.get() >= size`).

**Changes.**
- `UpTo(n) => size <= n.get()`, `AtLeast(n) => size >= n.get()`.
- `#[derive(Debug, PartialEq)]` on `ChoiceError` (`choice.rs:156`) for test ergonomics.
- Table-driven unit tests: for each variant check 0, n−1, n and n+1 picks.

**Acceptance.** `cargo test -p fifth_domain` green; no public API change.

---

## T1 — Add `Mode` / `Final` / `Draft`, `Slot<T>`, `ChoicePrompt` (additive)

**Goal.** Land the new vocabulary next to the existing code without changing behaviour.

**Changes.**
- New `fifth_domain/src/blueprint/mode.rs`
  - `pub trait Node: Clone + PartialEq + Serialize + DeserializeOwned {}` + blanket impl.
  - `pub trait Mode: Copy + Debug + PartialEq + 'static { type Effect: Node + From<Effect>; }`
    (the `Trait` kind is added in T2 once `Trait` is generic).
  - `pub struct Final; pub struct Draft;` deriving `Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`.
  - `impl Mode for Final { type Effect = Effect; }`, `impl Mode for Draft { type Effect = Slot<Effect>; }`.
- New `blueprint/slot.rs`
  - `#[serde(untagged)] pub enum Slot<T> { Choose(Choice<T>), Fixed(T) }` — `Fixed` serializes transparently.
  - `impl<T> From<T> for Slot<T>`, `impl<T> From<Choice<T>> for Slot<T>`; accessors `fixed()`, `choice()`, `choice_mut()`.
- New `blueprint/prompt.rs`
  - Object-safe `pub trait ChoicePrompt { prompt, cardinality, labels, is_selected(i), select(i), unselect(i), is_ok }`.
  - `impl<T: Clone> ChoicePrompt for Choice<T>` on top of the existing `ChoiceOption::select/unselect/is_selected`
    and `Choice::is_ok` (`choice.rs:54`).
- `choice.rs`: derive `PartialEq` on `Choice` (`:24`), `Cardinality` (`:71`), `ChoiceOption` (`:114`) — required by `Node`.
- `choice.rs`: rename `ChoiceOption.patches` → `values`, `item()` → `values()` (`:126`), builder mutator
  `with_patch` → `with_value` (`:110`); add `ChoiceOption::single(label, value)` and `many(label, values)`.
  Update the single call site `fifth_srd/src/races/dwarf.rs:107`.
- `blueprint/mod.rs`: `pub use` the new items.

**Tests.** `Slot<Effect>`: JSON of `Fixed(e)` equals JSON of `e`; `Choose(...)` round-trips; `ChoicePrompt`:
`select(0)` makes a `Cardinality::One` choice `is_ok()`.

**Acceptance.** Workspace green; existing JSON output of `fifth_srd::book()` unchanged except the
`patches` → `values` key rename.

---

## T2 — `Trait<M: Mode = Final>`, the new `Blueprint` trait and the `From<X<Final>>` lift

**Goal.** The leaf level becomes bimodal: `Trait` (= `Trait<Final>`) is exactly today's type;
`Trait<Draft>` is the trait blueprint. Requirement 1 becomes compiler-enforced at this level.

**Changes.**
- `fifth_domain/src/traits.rs:17`: `pub struct Trait<M: Mode = Final> { title, description, effects: Vec<<M as Mode>::Effect> }`
  with `#[serde(bound = "")]` (S0 items 1–2: the fully-qualified spelling is what typed-builder's `via_mutators` accepts,
  and it is what makes serde infer unwanted bounds on `M`).
  Builder mutators (`traits.rs:8`, `:11`) become `with_effect(impl Into<<M as Mode>::Effect>)` and
  `with_permanent_effect(SheetOperation)` (pushes `Effect::new(op).into()`); `add_effect` (`:31`) takes `impl Into<<M as Mode>::Effect>`.
- `mode.rs`: `Mode` gains `type Trait: Node + From<Trait<Final>>`;
  `Final::Trait = Trait<Final>`, `Draft::Trait = Slot<Trait<Draft>>`.
- `blueprint/mod.rs`: rename the legacy `pub trait Blueprint<Target>` (`mod.rs:20-24`) to `LegacyBlueprint<Target>`
  (impls at `blueprint/race.rs:42`, `blueprint/subrace.rs:45`; deleted in T4). Add:
  ```rust
  pub trait Blueprint: Sized {
      type Final;
      fn resolve(self) -> Result<Self::Final, ChoiceError>;
      fn visible_choices(&self) -> Vec<&dyn ChoicePrompt>;                 // DFS, descends only into selected options
      fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt>; // first choice that is not ok
      fn is_complete(&self) -> bool { self.visible_choices().iter().all(|c| c.is_ok()) }
  }
  ```
  The Final → Draft "lift" is plain `From<X<Final>> for X<Draft>` (all slots `Fixed`); no separate `Lift` trait —
  the `Mode` bounds `type Kind: Node + From<Kind<Final>>` let generic code lift with `Into::into` (S0).
- `slot.rs`: `impl<T: Blueprint> Slot<T> { resolve(self) -> Result<Vec<T::Final>, _>, visible_choices, next_open_choice_mut }`
  and `pub fn resolve_slots<T: Blueprint>(Vec<Slot<T>>) -> Result<Vec<T::Final>, ChoiceError>` (flattens picks).
- Leaf impl: `impl Blueprint for Effect` (identity; `visible_choices` empty).
- Hand-written (replaced by the derive in T7): `impl Blueprint for Trait<Draft>`, `impl From<Trait<Final>> for Trait<Draft>`,
  `impl From<Trait<Final>> for Slot<Trait<Draft>>`.
- `pub type TraitBlueprint = Trait<Draft>;`
- `ChoiceError::BadCardinality { prompt: String, expected: Cardinality, got: usize }` so the engine can name the unanswered choice.
- `patch/traits.rs:12`: `impl Patchable for Trait<Final>` — keeps `Blueberry<Trait>` (and therefore `fifth_srd`) compiling untouched.

**Tests.**
- `Trait<Draft>` with one `Choice<Effect>`: `resolve()` → `Err(BadCardinality { prompt, got: 0, .. })`;
  after `next_open_choice_mut().select(0)` → `Ok` with exactly the picked effect.
- `Cardinality::Exact(2)` yields two effects; an option with two values yields both.
- Nested: an option that is itself a `Trait<Draft>` containing a choice is only revealed after it is selected.
- JSON of `Trait<Final>` is byte-identical to before this ticket; `Trait<Draft>` round-trips.
- Doc-test ```` ```compile_fail ````: `Trait::<Final>::builder()...with_effect(Choice::...)` does not compile.

**Risks / fallbacks.** Both risks were resolved by S0: builder mutators need the fully-qualified spelling
`Vec<<M as Mode>::Effect>` / `impl Into<<M as Mode>::Effect>` (shorthand fails with E0220 in typed-builder 0.23.2),
and that spelling requires `#[serde(bound = "")]` on the struct. Reference implementation: `spike/hkd_spike/src/traits.rs`.

**Acceptance.** `fifth_srd` compiles with **no** changes; workspace green.

---

## T3 — `Race<M>` / `Subrace<M>`; `RaceBlueprint` becomes `Race<Draft>`; Dwarf on the new API

**Goal.** Remove the duplicated per-level blueprint structs. The book stores `Race<Draft>`; the sheet stores `Race`.

**Current state.** `RaceBlueprint` (`fifth_domain/src/blueprint/race.rs:22-32`, field `Vec<Either<Trait, Blueberry<Trait>>>`
at `:31`) and `SubraceBlueprint` (`blueprint/subrace.rs:34`) each carry their own `Blueprint` impl
(`race.rs:42-80`, `subrace.rs:45-83`) and a private copy of `serde_untagged_vec` (`race.rs:83`, `subrace.rs:93`).

**Changes.**
- `fifth_domain/src/race.rs:12`, `:40`: `Race<M: Mode = Final>`, `Subrace<M: Mode = Final>` with `traits: Vec<<M as Mode>::Trait>`
  and `#[serde(bound = "")]` (same reasons as T2);
  builder mutator `with_trait(impl Into<<M as Mode>::Trait>)`; `impl<M: Mode> Referable`; `new(..)` takes `Vec<<M as Mode>::Trait>`;
  `Subrace::for_race()`; derive `PartialEq` on `Race` (`race.rs:11`). Reference: `spike/hkd_spike/src/race.rs`.
- Hand-written `impl Blueprint for Race<Draft>` / `Subrace<Draft>` and the `From<Race<Final>>` lifts (derived in T7).
- `blueprint/mod.rs`: `pub type RaceBlueprint = Race<Draft>; pub type SubraceBlueprint = Subrace<Draft>;`
  delete `blueprint/race.rs` and `blueprint/subrace.rs`.
- `book.rs`: field types stay textually the same (`book.rs:25`, `:27` via the aliases); add `#[derive(Clone, PartialEq)]`
  (`book.rs:17`) so the engine can assert round-trip equality.
- `fifth_srd/src/races/dwarf.rs`: `:33` `with_trait_blueprint(tool_proficiency())` → `with_trait(tool_proficiency())`;
  `:86-108` `tool_proficiency() -> Trait<Draft>` built as
  `Trait::<Draft>::builder().title(..).description(..).with_effect(Choice::one(prompt, options)).build()`,
  `tool_proficiency_option(tool) -> ChoiceOption<Effect>` via `ChoiceOption::single(tool.tool().name(), Effect::new(..))`;
  drop the `Blueberry` and `TraitPatch` imports (`dwarf.rs:3`, `:7`).
- `fifth_srd/src/subraces/hill_dwarf.rs`: no change required (`SubraceBlueprint::builder()` at `:15` resolves through the alias;
  `with_trait(Trait)` lifts automatically).
- `fifth_engine/src/lib.rs:6-12`: assert `deserialized == original`; keep the size print.

**Tests.**
- `fifth_srd::book().race(&dwarf_ref)` → walk `next_open_choice_mut` → select "Smith Tools" → `resolve()` returns a `Race`
  whose "Tool Proficiency" trait has exactly one proficiency effect.
- Doc-test ```` ```compile_fail ````: `Race::<Final>::builder()...with_trait(Trait::<Draft>::builder()...)`.

**Acceptance.** Workspace green. `grep` shows `Blueberry`, `Patchable`, `TraitPatch`, `ChoiceKind`, `LegacyBlueprint`
and `either::` have no remaining users → T4.

---

## T4 — Remove the legacy blueprint machinery

**Changes (deletions only).**
- `fifth_domain/src/patch/` (`patch/mod.rs`, `patch/traits.rs`) and `pub mod patch;` (`lib.rs:17`).
- `Patchable` (`blueprint/mod.rs:15-18`), `Blueberry` (`mod.rs:26-55`), `LegacyBlueprint`.
- `ChoiceKind` and its `From<&mut Choice<TraitPatch>>` (`choice.rs:9-17`) — note it handed out `&Choice`, so nothing
  could ever be selected through it.
- `either` dependency (`fifth_domain/Cargo.toml:14`) and its imports.

**Acceptance.** Workspace green; `grep -rn "Blueberry\|Patchable\|TraitPatch\|ChoiceKind\|either" fifth_domain/src fifth_srd/src fifth_engine/src`
returns nothing.

---

## T5 — Compact, readable JSON for choices

**Goal.** A choice is one inline object; fixed values cost nothing; a book never carries selection state.

**Changes (`choice.rs`, `traits.rs`).**
- `Cardinality { UpTo(NonZeroUsize), AtLeast(NonZeroUsize), #[serde(untagged)] Exactly(NonZeroUsize) }`
  with `rename_all = "camelCase"` — serde requires the `untagged` variant to be declared **last** (S0);
  drop `One` in favour of `Cardinality::ONE` (`Exactly(NonZeroUsize::MIN)`) and `Choice::one(..)`.
  JSON: `1` | `{"upTo": 2}` | `{"atLeast": 1}`.
- `Choice<T>`: `#[serde(rename = "choose")] cardinality`, `prompt`,
  `#[serde(with = "options_map", bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))] options`
  (serde infers no bounds for a `with` field — S0 item 5; a newtype `Options<T>` is the alternative),
  `#[serde(default, skip_serializing_if = "Vec::is_empty")] selected: Vec<String>` (labels).
  Remove `ChoiceOption.selected` (`choice.rs:122`); `ChoicePrompt` and `make()` work on labels.
  Duplicate labels are rejected by the builder and, on load, by the map.
- `options_map`: serialize `label → value` (single) or `label → [values]` (several), keeping order;
  deserialize with a `MapAccess` visitor into `Vec<ChoiceOption<T>>` (document order preserved); untagged `OneOrMany<T>` helper.
  Reference implementation (tested): `spike/hkd_spike/src/blueprint/choice.rs`.
- `Trait.effects`: `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.

**Target shape (Dwarf, Tool Proficiency).**
```json
{
  "title": "Tool Proficiency",
  "description": "You gain proficiency with the artisan's tools of your choice: smith's tools, brewer's supplies, or mason's tools.",
  "effects": [
    {
      "choose": 1,
      "prompt": "Choose one tool to be proficient with.",
      "options": {
        "Smith Tools":     { "operation": "proficiency", "argument": { "proficiency_kind": "tool", "proficient_with": "srd/tool/smith_tools" } },
        "Brewer Supplies": { "operation": "proficiency", "argument": { "proficiency_kind": "tool", "proficient_with": "srd/tool/brewer_supplies" } },
        "Mason Tools":     { "operation": "proficiency", "argument": { "proficiency_kind": "tool", "proficient_with": "srd/tool/mason_tools" } }
      }
    }
  ]
}
```
(The effect payload itself shrinks further in T6.)

**Tests.** Round-trip `Trait<Draft>` and `Race<Draft>`; option order preserved; single vs. several values;
`selected` absent in book JSON and present after a selection; the engine test prints compact and pretty byte sizes of
`fifth_srd::book()` — record before/after in the PR description.

**Risk.** Untagged `Slot` tries `Choose` first; a fixed value never has a `choose` key, so no ambiguity.

---

## T6 — Compact, readable JSON for effects *(independent, optional)*

**Goal.** `{"proficiency": {"tool": "srd/tool/smith_tools"}}` instead of
`{"operation": "proficiency", "argument": {"proficiency_kind": "tool", "proficient_with": "srd/tool/smith_tools"}}`.

**Changes.** Switch to serde's default (externally tagged) representation by removing the adjacent-tag attributes on
`SheetOperation` (`fifth_domain/src/effect.rs:57`), `Proficiency` (`proficiency.rs:11`) and `DamageModulation`
(`damage.rs:52`), keeping `rename_all = "camelCase"`; `Proficiency::Skill.expertise` gets
`#[serde(default, skip_serializing_if = "Not::not")]`. `Effect` keeps `#[serde(flatten)]` (`effect.rs:36`), so an effect is
`{"proficiency": {...}}` plus `"id"` only when set. Before starting, grep `fifth_srd/resources/*.json` for
`operation|proficiency_kind|"kind"` — expected none (`features.json` holds only `name`/`description`).

**Tests.** Round-trip every `SheetOperation` variant, including unit-like payloads
(`{"vision": "Darkvision"}`, `{"raceWalkingSpeed": 25}`, `{"ignoreRule": "HeavyArmorSpeedPenalty"}`) and an effect with an id;
record byte sizes before/after.

**Risk / fallback.** `flatten` over an externally tagged enum relies on all variants carrying data (true for every
`SheetOperation` variant, `effect.rs:59-71`); fallback is a hand-written `Serialize`/`Deserialize` for `Effect`.

---

## T7 — `fifth_derive`: `#[derive(Blueprint)]`

**Goal.** Requirement 2 in full: the per-struct `Blueprint`/`From` impls are generated from the struct definition.

**Why a new crate.** `fifth_macros` depends on `fifth_domain` (`fifth_macros/Cargo.toml:18`), so a derive used *inside*
`fifth_domain` cannot live there.

**Changes.**
- New proc-macro crate `fifth_derive` (`syn = "2"`, `quote = "1"`, `proc-macro2 = "1"`, matching `fifth_macros/Cargo.toml:11-16`);
  add to the workspace `members`; `fifth_domain` depends on it and re-exports `pub use fifth_derive::Blueprint;` from `blueprint`.
- Rules: input is `struct X<M: Mode> { .. }`; a **slot field** is exactly a field typed `Vec<M::Kind>` or
  `Vec<<M as Mode>::Kind>` (the spelling T2/T3 need for typed-builder); any other use of `M`
  is a compile error ("unsupported slot shape"). Generated paths are hard-wired to `crate::blueprint` (fine while the
  derive is only used inside `fifth_domain`; a `#[blueprint(crate = "..")]` attribute can lift that later). Generates
  - `impl Blueprint for X<Draft>` — `resolve` moves plain fields and `resolve_slots(..)?` each slot field;
    `visible_choices` / `next_open_choice_mut` chain slot fields in declaration order;
  - `impl From<X<Final>> for X<Draft>` (the lift; slot fields mapped with `Into::into`, which the `Mode` bounds
    `type Kind: From<Kind<Final>>` guarantee) — this replaces the separate `Lift` trait;
  - `impl From<X<Final>> for Slot<X<Draft>>`.
- Replace the hand-written impls for `Trait`, `Race`, `Subrace` (from T2/T3) with the derive.
  Reference implementation (tested, ~150 lines): `spike/spike_derive/src/lib.rs`.

**Acceptance.** All tests from T2/T3 unchanged and green; net LOC reduction; a struct with two slot fields
(test-only) yields choices in declaration order.

---

## T8 — Prove it scales: `Feature<M>` and `ClassLevel<M>`

**Goal.** Show that adding a kind costs one struct definition plus two one-line additions to the `Mode` impls.

**Changes.**
- `fifth_domain/src/feature.rs`: `Feature<M: Mode = Final> { name, description, effects: Vec<<M as Mode>::Effect> }` with derives,
  `#[serde(bound = "")]` + `#[derive(Blueprint)]`.
- `mode.rs`: `Mode` gains `type Feature: Node + From<Feature<Final>>`; `Final::Feature = Feature<Final>`, `Draft::Feature = Slot<Feature<Draft>>`.
- `class.rs`: `ClassLevel<M: Mode = Final> { features: Vec<<M as Mode>::Feature> }` (+ `ClassLevels<M>`), `#[derive(Blueprint)]`.
- Aliases `FeatureBlueprint`, `ClassLevelBlueprint`.

**Test.** Ability Score Improvement (`fifth_srd/resources/features.json:26-29`: "+2 to one ability … or +1 to two") as a
`Choice<Feature<Draft>>` whose two options each contain a nested `Choice<Effect>` (`Exactly(1)` over +2, `Exactly(2)` over +1):
the wizard walk reveals the inner prompt only after the outer pick; `resolve()` yields one or two ASI effects.

**Acceptance.** No file outside `feature.rs`, `class.rs`, `mode.rs` (and the test) is touched.

---

## T9 — Backlog (not scheduled)

- Query-based option sets (`Options::Query`) so "any language" / "any artisan's tool" can span books.
- Engine-evaluated constraints ("choose a skill you are already proficient in").
- Descriptions that reflect the selected option.
- Class progression: resolve `ClassLevel<Draft>` per level as the character levels up.
- A custom `Slot` deserializer for friendlier error messages than serde's untagged fallback.

---

## Requirement coverage

| Requirement (`problem.md`) | Delivered by |
|---|---|
| 1. Tight domain model | T2, T3 (`compile_fail` doc-tests: a `Final` value cannot take a `Choice`/`Draft`) |
| 2. No unmaintainable boilerplate | T3 + T4 (delete duplicated structs/serde modules/patch enum), T7 (derive), T8 (proof: 1 struct + 2 lines per kind) |
| 3. Patches minimal and local | T2, T3 (no patches; a `Choice` sits at the position it fills, options carry whole values) |
| 4. Writable from Rust | T1 (`ChoiceOption::single`, `Choice::one`), T2, T3 (builders accept `Trait`, `Trait<Draft>` or `Choice` via `Into`) |
| 5. Type safety | T1–T3 (`Choice<Effect>` vs `Choice<Trait<Draft>>` are distinct types; no string maps) |
| 6. Small, readable JSON | T5 (choices), T6 (effects) |
| Latent bugs found | T0 (`good_len`), T4 (`ChoiceKind` shared reference) |
