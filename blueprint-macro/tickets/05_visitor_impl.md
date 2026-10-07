# Ticket 5: Visitor and Walkable Implementation

## Objective
Implement the `Walkable` trait for the blueprint, allowing choices to be visited in a breadth-first manner as specified in the design doc.

## Requirements
1. Create `src/gen_visitor.rs`.
2. Generate `impl<V> blueprint::Walkable<V> for <StructName>Blueprint`.
3. Implement `fn walk_level(&mut self, visitor: &mut V, depth: usize) -> blueprint::Level`.
4. Ensure the visitor checks slots at depth `0` (immediate fields) and visits any `Slot::Choose`.
5. If the field is nested, delegate deeper walks to the nested blueprint.
6. Fully populate the `walk_choices`, `unmake_all`, and `confirm_all` methods stubbed in Ticket 4, making them route to the appropriate visitors.
7. Reference `blueprint/src/dnd_model/race.rs` for manual implementation details.

## Output
A `TokenStream` providing the visitor implementations, completing the recursive choice iteration requirements.
