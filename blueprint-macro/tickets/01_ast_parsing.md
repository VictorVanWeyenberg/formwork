# Ticket 1: AST Parsing and Configuration Extraction

## Objective
Implement robust AST parsing for the `#[blueprint]` macro. The POC `core.rs` currently iterates over tokens loosely. We need a strongly-typed representation of the input struct and its fields, separating parsing from code generation.

## Requirements
1. Create `src/ast.rs`.
2. Define a `BlueprintStruct` struct that holds the parsed data of the `ItemStruct`.
3. Define a `BlueprintField` struct for each field, containing its `Ident`, `Type`, and parsed `SlotAttribute`.
4. Parse the `#[slot]` attribute on fields. Support the arguments: `vec`, `choice`, `nested`.
   - Represent this as an enum or bitflags, e.g., `SlotConfig { is_vec: bool, is_choice: bool, is_nested: bool }`.
5. Implement `syn::parse::Parse` or use a struct initialization method that takes `&syn::Field` and returns `Result<BlueprintField, syn::Error>`.
6. Gracefully handle errors: return descriptive `syn::Error::new_spanned` if `#[blueprint]` is used on enums, traits, or structs with unnamed fields (tuple structs).

## Output
A module `ast` that exposes `BlueprintStruct::parse(item: ItemStruct) -> syn::Result<Self>` which will act as the data model for all subsequent code generation modules.
