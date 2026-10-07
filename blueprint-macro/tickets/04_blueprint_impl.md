# Ticket 4: Blueprint Trait Implementation

## Objective
Generate the `Blueprint` trait implementation for the draft struct, providing `is_complete()` checks and `resolve()` logic.

## Requirements
1. Create `src/gen_blueprint.rs`.
2. Accept a parsed `BlueprintStruct`.
3. Generate `impl blueprint::Blueprint for <StructName>Blueprint`.
4. Define `type Final = <StructName><FinalMarker>;`.
5. Implement `fn is_complete(&self) -> bool`:
   - Iterate over all slotted fields and ensure `is_complete()` returns true.
   - For `Vec<Slot>`, use `.iter().all(|s| s.is_complete())`.
   - Ensure nested fields also trigger appropriate readiness checks recursively if needed.
6. Implement `fn resolve(self) -> Result<Self::Final, BlueprintError>`:
   - Handle unpacking slots.
   - For `nested` fields, resolve the outer slot, then call `.resolve()` on the inner blueprint struct recursively.
   - Aggregate all results and construct the Final struct.
7. Stub out `walk_choices`, `unmake_all`, and `confirm_all` (they will be implemented in Ticket 5).

## Output
A `TokenStream` containing the `Blueprint` implementation.
