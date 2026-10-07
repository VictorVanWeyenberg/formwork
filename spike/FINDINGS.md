# S0 findings — throwaway spike for `tickets.md`

Spike location: `spike/` next to this file — originally `files/spike` in the Copilot session folder (workspace: `hkd_spike` = the design, `spike_derive` = `#[derive(Blueprint)]`,
`probe` = typed-builder bisection). Not meant to be merged; every file mirrors `fifth_domain` names so the
results transfer 1:1 to T1–T7.

Result: **`cargo test -p hkd_spike` → 18 integration tests + 4 doc-tests green** (two of them `compile_fail`
proofs of requirement 1), `cargo check --workspace --all-targets` warning-free.

## How it was run (this Windows box)

- No Visual Studio / `link.exe` here; the repo itself is built on Linux (`x:\fifth\target\.rustc_info.json`:
  `host: x86_64-unknown-linux-gnu`, rustc 1.97.1, `.so` artifacts).
- Installed `stable-x86_64-pc-windows-gnu` (self-contained linker) and ran
  `cargo +stable-x86_64-pc-windows-gnu test -p hkd_spike` with `CARGO_HTTP_CHECK_REVOKE=false`
  (corporate TLS revocation check otherwise breaks crates.io).
- The unit-test *harness of a proc-macro crate* needs mingw `dlltool.exe` (absent) → do not `cargo test`
  proc-macro crates on this box (`--exclude fifth_macros` / `fifth_derive`), or run on the Linux host.

## Verdict per S0 item

| # | Question | Verdict | Fallback needed? |
|---|----------|---------|------------------|
| 1 | Do `Clone`/`PartialEq`/`Serialize`/`Deserialize` derives work on `struct X<M: Mode> { .., Vec<M::Kind> }`? | **Yes.** With the shorthand `Vec<M::Kind>` serde infers *no* bound on `M` (`samples::Both` has no attribute). With the fully-qualified `Vec<<M as Mode>::Kind>` serde *does* see `M` and demands `M: Serialize + Deserialize<'de>` (+ `M: Default` if the field has `#[serde(default)]`). | Only for the FQ spelling: `#[serde(bound = "")]` on the container (the `Node` bounds on the associated types carry the impls). `Final`/`Draft` stay plain unit structs. |
| 2 | Can `typed_builder` mutators mention `M::Kind`? | **Not with the shorthand.** typed-builder 0.23.2 fails with E0220 on any field marked `#[builder(via_mutators)]` whose type is `Vec<M::Kind>` — with or without a mutator, with `default` or `via_mutators(init = …)`, with or without the `= Final` default parameter (`probe`, feature `typed_builder_e0220_repro`). A mutator over a plain `Vec<String>` next to a `Vec<M::Kind>` setter field is fine. | **Yes, cheap:** spell slot fields `Vec<<M as Mode>::Kind>` and mutator arguments `impl Into<<M as Mode>::Kind>` (`probe::k_fully_qualified_projection`). Builder stays usable in both modes with `M` inferred from the return type — `Trait::builder()…build()` in `fn tool_proficiency() -> Trait<Draft>` needs no turbofish. Consequence: item 1's `#[serde(bound = "")]`. |
| 3 | Coherence of `From<T> for Slot<T>`, `From<Choice<T>> for Slot<T>`, `From<Trait<Final>> for Slot<Trait<Draft>>` (+ derived `From<X<Final>> for X<Draft>`)? | **Yes**, all coexist; inference through `impl Into<<M as Mode>::Kind>` resolves `Choice<?T>` from the target slot, and `ChoiceOption::single(label, impl Into<T>)` lifts a finished `Trait` into a `Choice<Trait<Draft>>` option (`samples::fighting_style`). | No. |
| 4 | `#[serde(flatten)]` over an externally tagged `SheetOperation`? | **Yes.** 13 shapes round-trip, incl. nested struct/unit/newtype/untagged payloads, `id` skipped when empty, and through the untagged `Slot` and the label-keyed options map (serde `Content` buffering). Precondition holds: every `SheetOperation` variant carries data. | No. |
| 5 | Label-keyed `options` map keeps document order? | **Yes**: a `MapAccess` visitor into `Vec<ChoiceOption<T>>` preserves order; duplicates fail with ``duplicate option label `a` ``; `value \| [values]` via an untagged `OneOrMany<T>`. | Not a fallback, but required: serde infers no bounds for a `#[serde(with)]` field → `#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]` on that field (or a newtype `Options<T>`). |
| 6 | Minimal `#[derive(Blueprint)]`? | **Yes**, ~150 lines of `syn`/`quote`: detects slot fields in both spellings, rejects other uses of `M`, generates `impl Blueprint for X<Draft>` (`resolve`, `visible_choices`, `next_open_choice_mut`), `impl From<X<Final>> for X<Draft>` (lift) and `impl From<X<Final>> for Slot<X<Draft>>`. Fields visited in declaration order (`the_derive_visits_slot_fields_in_declaration_order`). Same name as the trait, different namespace (`pub use spike_derive::Blueprint;` next to `pub trait Blueprint`). | No. Paths are hard-wired to `crate::blueprint` — fine while the derive is only used inside `fifth_domain`. |

## Other findings

- serde: `#[serde(untagged)]` **variants must be declared last** → `Cardinality { UpTo, AtLeast, #[serde(untagged)] Exactly }`.
- No `Debug` needed on domain types: `Node = Clone + PartialEq + Serialize + DeserializeOwned` suffices
  (tests use `.err().expect()` / `.ok().unwrap()`), so T1 does not have to add `Debug` derives across the crate.
- `Blueprint::next_open_choice_mut` skips choices that are already `is_ok()` — for `UpTo(n)` that includes
  "nothing picked yet". UIs that must show optional choices use `visible_choices()`.
- Borrow-checker: the generated `if let Some(c) = self.<slot>.iter_mut().find_map(Slot::next_open_choice_mut) { return Some(c) }`
  chain compiles because successive slot fields are disjoint places.
- Dwarf draft JSON (spike stand-ins, `dwarf-draft.json.txt`): 1 646 bytes compact / 2 741 pretty; the only
  non-value in the whole document is the one `{"choose": 1, "prompt": …, "options": {…}}` object.

## Consequences folded into the tickets

- **T1**: `Cardinality` variant order; `ChoiceOption::single(label, impl Into<T>)` / `many(..)`.
- **T2 / T3 / T8**: slot fields spelled `Vec<<M as Mode>::Kind>`, mutator args `impl Into<<M as Mode>::Kind>`,
  `#[serde(bound = "")]` on every struct that has both a builder and a slot field.
- **T5**: `#[serde(with = "options_map", bound(...))]` on `options` (or newtype).
- **T7**: derive accepts both spellings; hard-wired `crate::blueprint` paths documented.
- **Ground rules**: how to run the tests on this Windows box vs. the Linux host.
