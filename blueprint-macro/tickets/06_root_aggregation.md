# Ticket 6: Root Macro Aggregation and Error Handling

## Objective
Stitch all generation modules together inside the main `lib.rs` and provide user-friendly compiler errors.

## Requirements
1. Update `src/lib.rs` to expose the main `#[blueprint]` proc-macro.
2. Delegate the AST parsing to `ast::BlueprintStruct::parse(item)`.
3. If parsing fails, cleanly return `syn::Error::into_compile_error()` instead of panicking.
4. Pass the parsed AST through:
   - `gen_mode::generate_modes`
   - `gen_struct::generate_struct`
   - `gen_blueprint::generate_blueprint`
   - `gen_visitor::generate_visitor`
5. Collect the returned `TokenStream`s, package them, and return them as the macro output.
6. Verify against `tests/integration.rs` to ensure the final structured implementation passes all POC tests flawlessly.

## Output
A fully refactored, robust `blueprint-macro` crate ready for production use.
