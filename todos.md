# Todo state at handoff (2026-09-16)

Mirrors the session's todo table. One row per ticket in `tickets.md`; update the status column as tickets land.

| Id | Status | Depends on | Title | Notes |
|----|--------|------------|-------|-------|
| S0 | **done** | — | Throwaway spike (outside the repo) | 18 integration + 4 doc-tests green; see `spike/FINDINGS.md`. typed-builder `via_mutators` needs `Vec<<M as Mode>::Kind>` + `#[serde(bound = "")]`; untagged `Cardinality` variant last; everything else works as designed. |
| T0 | pending | — | Fix `Cardinality::good_len` | `fifth_domain/src/blueprint/choice.rs:95-96` inverted comparisons for `UpTo`/`AtLeast`; derive `Debug`+`PartialEq` on `ChoiceError`; table-driven tests. |
| T1 | pending | — | Add `Mode`/`Final`/`Draft`, `Slot<T>`, `ChoicePrompt` (additive) | New `blueprint/{mode,slot,prompt}.rs`; `PartialEq` on `Choice`/`Cardinality`/`ChoiceOption`; rename `patches`→`values`, `with_patch`→`with_value` (update `fifth_srd/src/races/dwarf.rs:107`); `ChoiceOption::single/many`; `Slot` serde transparency tests. |
| T2 | pending | T1 | `Trait<M: Mode = Final>` + `Blueprint` trait + `From<X<Final>>` lift | Slot field `Vec<<M as Mode>::Effect>` + `#[serde(bound = "")]`; `Mode::Trait`; rename legacy `Blueprint<Target>`→`LegacyBlueprint`; `Slot` resolve helpers; impls for `Effect`/`Trait`; `TraitBlueprint` alias; `BadCardinality` carries the prompt; `Patchable for Trait<Final>` keeps `fifth_srd` untouched; `compile_fail` doc-test. |
| T3 | pending | T2 | `Race<M>`/`Subrace<M>`; `RaceBlueprint = Race<Draft>`; Dwarf on the new API | Delete `blueprint/race.rs` + `subrace.rs`; `Book` derives `Clone`+`PartialEq`; `dwarf.rs` `tool_proficiency() -> Trait<Draft>`; engine asserts round-trip equality; `compile_fail` doc-test. |
| T4 | pending | T3 | Remove legacy blueprint machinery | Delete `patch/`, `Patchable`, `Blueberry`, `LegacyBlueprint`, `ChoiceKind`, `either` dependency; grep confirms no users. |
| T5 | pending | T1 (best after T4) | Compact choice JSON | `"choose": 1 | {"upTo": n} | {"atLeast": n}`; label-keyed ordered options map (`value | [values]`); `selected: Vec<String>` skipped when empty; `effects` skipped when empty; record byte sizes before/after. |
| T6 | pending | — | Compact effect JSON (independent) | Externally tagged `SheetOperation`/`Proficiency`/`DamageModulation`; `expertise` skipped when false; keep `Effect` flatten; round-trip tests; record byte sizes. |
| T7 | pending | T3 (best after T4) | `fifth_derive` with `#[derive(Blueprint)]` | New proc-macro crate (`fifth_macros` depends on `fifth_domain`, so it cannot host it); both slot spellings; generates `Blueprint for X<Draft>`, `From<X<Final>> for X<Draft>`, `From<X<Final>> for Slot<X<Draft>>`; replace hand-written impls. |
| T8 | pending | T7 | Prove scale with `Feature<M>` and `ClassLevel<M>` | `Feature<M>` + `Mode::Feature` (+2 lines); `ClassLevel<M>` with `Vec<<M as Mode>::Feature>`; nested Ability Score Improvement test (`fifth_srd/resources/features.json:26-29`). |
| T9 | backlog | — | Not scheduled | Query-based option sets; sheet-state constraints; description templating; per-level class resolution; friendlier `Slot` deserialization errors. |

Recommended order: T0 → T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8 (T6 is independent).
