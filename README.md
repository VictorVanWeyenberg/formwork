# `fifth` — blueprint redesign: handoff

This folder is a self-contained backup of a Copilot planning session (2026-09-16) about `problem.md`
("where do blueprints live?") for the Rust workspace in `x:\fifth`. It exists so the chat can be deleted
and the work continued later — by a person or by a fresh Copilot session.

Nothing in `x:\fifth` was changed by that session. At handoff the repo was on `master` at
`991750c Write logic for the (sub)race blueprints.` with a single untracked file, `problem.md`
(a copy is kept here; **the original in the repo must not be deleted**).

## Status

- **Design decided** — see `plan.md`: a blueprint is the *same* domain type in `Draft` mode
  (`Race<M: Mode = Final>`, `Trait<M>`, …); a list position is `M::Kind` = the value (`Final`) or
  `Slot<T> = Fixed(T) | Choose(Choice<T>)` (`Draft`). No patches: a `Choice` sits at the position it fills and
  its options carry whole values. `#[derive(Blueprint)]` generates the per-struct `resolve`/lift impls.
- **Delivery split into tickets** — see `tickets.md` (S0, T0 … T9) and `todos.md` (status).
- **S0 (throwaway spike) is done** — `spike/` is a tested reference implementation of the whole design
  (18 integration tests + 4 doc-tests, two of them `compile_fail` proofs that a `Final` value cannot hold a
  choice). Verdicts and the two fallbacks that turned out to be necessary are in `spike/FINDINGS.md`.
- **Next: T0** (fix `Cardinality::good_len`), then T1 → T2 → … as listed in `tickets.md`.

## Reading order for a fresh start

1. `problem.md` — the original problem statement (5 requirements) + the later bonus requirement:
   the JSON of a source should be as small and human-readable as possible.
2. `plan.md` — the design, alternatives considered, requirement checklist.
3. `tickets.md` — one PR per ticket, each leaving the workspace green; acceptance criteria per ticket.
4. `todos.md` — what is done / pending.
5. `spike/FINDINGS.md`, then the spike sources — copy shapes from there (`traits.rs`, `race.rs`,
   `blueprint/choice.rs`, `spike_derive/src/lib.rs`), they are tested.

## Working agreements from the session

- Do **not** change code in `x:\fifth` until the user approves a specific ticket; design questions go through
  plan mode first.
- One ticket at a time; stop after each ticket for review.
- Never delete `x:\fifth\problem.md`.
- Legacy code (`Blueberry`, `Patchable`, `TraitPatch`, `RaceBlueprint` struct, `ChoiceKind`, `either`) stays until
  it has zero users, then goes in T4.

## Environment notes (Windows box used in the session)

- `x:\` is the SMB share `//server/fedora`; git refuses it by default → run
  `git -c safe.directory='*' …` per invocation (do not edit the global config).
- The repo is normally built on the Linux host (`x:\fifth\target\.rustc_info.json`: `x86_64-unknown-linux-gnu`,
  rustc 1.97.1). The Windows box has **no MSVC linker**; the self-contained
  `stable-x86_64-pc-windows-gnu` toolchain was installed with rustup and works:
  `cargo +stable-x86_64-pc-windows-gnu test --workspace --exclude fifth_macros`
  (proc-macro crates' own unit-test harness needs mingw `dlltool.exe`, which is absent).
- crates.io behind the corporate TLS proxy needs `$env:CARGO_HTTP_CHECK_REVOKE = "false"` for the cargo process.
- Consider `$env:CARGO_TARGET_DIR` pointing to a local disk to avoid building on the share.

## Re-running the spike

```powershell
Set-Location x:\fifth-planning\spike
$env:CARGO_HTTP_CHECK_REVOKE = "false"
cargo +stable-x86_64-pc-windows-gnu test -p hkd_spike                       # 18 tests + 4 doc-tests
cargo +stable-x86_64-pc-windows-gnu check -p probe --features typed_builder_e0220_repro   # reproduces the typed-builder limitation
```
(On Linux, plain `cargo test -p hkd_spike` and `cargo test --workspace` work.)

## Files

| Path | What |
|------|------|
| `README.md` | this handoff |
| `problem.md` | copy of the problem statement (original stays untracked in `x:\fifth`) |
| `plan.md` | design rationale + requirement checklist |
| `tickets.md` | incremental tickets S0, T0 … T9 with acceptance criteria |
| `todos.md` | ticket status |
| `spike/FINDINGS.md` | S0 verdicts per question, fallbacks, run notes |
| `spike/hkd_spike/` | the design as a small crate mirroring `fifth_domain` (modes, slots, choices, `Trait<M>`, `Race<M>`, Dwarf sample, nested samples) + integration tests |
| `spike/spike_derive/` | `#[derive(Blueprint)]` proc-macro (~150 lines) |
| `spike/probe/` | typed-builder bisection (positive controls; failing variants behind a feature flag) |
| `spike/dwarf-draft.json.txt` | the Dwarf blueprint in the target JSON shape (1 646 bytes compact) |
| `spike/Cargo.lock` | pins the crate versions the findings apply to (serde 1.0.229, serde_json 1.0.151, typed-builder 0.23.2) |

## Prompt to resume in a new chat (working directory `x:\fifth`)

> Continue the blueprint redesign for `x:\fifth`. Read `x:\fifth-planning\README.md` first, then `plan.md`,
> `tickets.md`, `todos.md` and `spike\FINDINGS.md` in that folder. Do not change the repository until I approve a
> ticket. Start by proposing ticket **T0** from `tickets.md` in plan mode; after approval implement only T0, keep
> `todos.md` up to date, and stop for review. Never delete `problem.md`.
