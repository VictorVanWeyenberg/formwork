# Ticket 3: Builder and Core Struct Generation

## Objective
Generate the main struct definition, injected with `<M: Mode = Final>` generics, along with `TypedBuilder` attributes and draft mutator methods.

## Requirements
1. Create `src/gen_struct.rs`.
2. Accept a parsed `BlueprintStruct`.
3. Generate the main `struct <StructName><M: Mode>` block.
4. Add `TypedBuilder` derives gated behind `#[cfg_attr(feature = "builders", ...)]`.
5. For vector fields that act as vectors in *both* Draft and Final modes, ensure `#[builder(default, via_mutators)]` is correctly applied.
6. Generate the `TypedBuilder` `mutators(...)` block appropriately for vector fields. Make sure to avoid trait bound errors by ensuring fields properly map to `.push(item.into())`.
7. Generate an `impl <StructName><Draft>` block containing helper methods for slots:
   - `add_<field>_choice(&mut self, choice: Choice<T>)` for choice slots.
   - For vec slots, this method should push a `Slot::Choose(choice)` to the vector.
8. Provide a function `generate_struct(ast: &BlueprintStruct) -> TokenStream`.

## Output
A `TokenStream` with the parameterized struct, its builder configuration, and its draft-mode helper methods.
