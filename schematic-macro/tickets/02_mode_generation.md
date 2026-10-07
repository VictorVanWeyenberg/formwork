# Ticket 2: Trait and Mode Structs Generation

## Objective
Generate the typestate structures (`Draft` and `Final` structs) and the generic `Mode` trait for the blueprint.

## Requirements
1. Create `src/gen_mode.rs`.
2. Accept a parsed `BlueprintStruct` (from Ticket 1) as input.
3. Generate the `Trait` named `<StructName>Mode` containing an associated type for each field of the struct.
4. Generate the `Draft` and `Final` marker structs.
5. Implement the `Mode` trait for both marker structs:
   - For `Final`, the associated type should resolve to the original base type of the field.
   - For `Draft`, generate the correct `Slot<...>` wrapper depending on the `SlotConfig`:
     - If `is_vec`, generate `Vec<Slot<T>>` or `Slot<Vec<T>>` depending on whether it's a vector of slots or a single choice resolving to a vector.
     - If `is_nested`, wrap the inner type appropriately.
6. Provide a function `generate_modes(ast: &BlueprintStruct) -> TokenStream`.

## Output
A `TokenStream` containing the trait and marker struct definitions and their implementations.
