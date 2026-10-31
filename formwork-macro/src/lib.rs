#![warn(missing_docs)]

//! Procedural macros for the Higher-Kinded Data (HKD) formwork architecture.
//!
//! Provides the primary [`#[formwork]`](macro@formwork) attribute macro to transform struct
//! definitions into dual-mode Higher-Kinded Data structures.

use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod ast;
mod gen_formwork;
mod gen_mode;
mod gen_struct;
mod gen_visitor;

/// Transforms a struct into a Higher-Kinded Data (HKD) formwork type.
///
/// `#[formwork]` generates a wet formwork mold alongside a cured, watertight dry domain
/// model from a single struct definition.
///
/// # The Formwork Architecture Metaphor
///
/// The architecture follows the concrete casting metaphor:
/// - **Formwork**: The temporary mold or structural frame holding incomplete data and pending choices.
/// - **Wet State (`*Wet`, `Slot::FixedWet`, `FormworkError::WetChoice`)**: Data in the fluid, mutable stage where decisions are pending, choices have selection histories, and constraints are evaluated.
/// - **Dry State (`*Dry`, `Slot::FixedDry`, `Formwork::Dry`)**: The solid, cured, permanent domain model. Watertight, rigid, zero optional wrappers, zero choice state.
/// - **Resolution**: Consuming the wet formwork to produce the watertight dry domain model via `.resolve()` (and `.resolve_exact()` for scalar choices and single slots).
///
/// `#[formwork]` generates a single struct parameterized over a mode `M`:
///
/// ```rust,ignore
/// pub struct Character<M: CharacterMode = CharacterDry> {
///     // ...
/// }
/// ```
///
/// - **Dry Domain Model (`Character`)**: Unparameterized, `Character` defaults to dry mode (`CharacterDry`).
///   All fields contain their watertight, concrete types (`T`, `Vec<T>`). No choices or wet slots exist.
/// - **Wet Formwork (`CharacterFormwork`)**: A type alias for `Character<CharacterWet>`.
///   Slot fields wrap values in stateful slots capable of holding pending choices, made choices with history,
///   or fixed wet values.
/// - **Resolution**: Wet formworks implement the `Formwork` trait.
///   Calling `resolve()` verifies that all choices across the entire hierarchy
///   are satisfied and produces the watertight dry domain model.
///
/// # Configuration Attributes
///
/// Pass options directly into `#[formwork(...)]`:
///
/// - `builder`: Generates a fluent, compile-time checked builder powered by `typed-builder`.
///   Collection slots receive ergonomic `with_<field>` (single item, choice, or slot) and `<field>`
///   (iterator of items, choices, or slots) mutator methods.
/// - `traverse = "bfs"` (default) or `"dfs"`: Specifies the tree traversal strategy used when
///   calling `walk_choices`.
/// - `derives(...)`: Optionally specifies additional derive macros (e.g. `Clone`, `PartialEq`,
///   `Serialize`, `Deserialize`) to apply to internal mode marker types. Any `#[derive(...)]`
///   attributes placed directly on the struct are forwarded automatically.
///
/// # Slot Decision Criteria
///
/// When designing a formwork struct, consider whether each field should be a slot:
///
/// - **Normal Field (no `#[slot]`)**: The value is known unconditionally upfront at construction time.
///   It cannot be deferred, picked from choices, or altered by visitors (e.g. an ID, a static title, metadata).
/// - **Slot Field (`#[slot]`)**: The field may involve choices, deferred selection, or hierarchical resolution.
///
/// # 2x4 Orthogonal Slot Matrix
///
/// Slot attributes are designed along two orthogonal dimensions:
/// - **Cardinality / Multiplicity**: Single (scalar value), Option (optional value), Collection (arbitrary items or choices), or Pick (pool of candidates with cardinality constraints).
/// - **Hierarchy**: Leaf (resolves to concrete data directly) vs. Cascading (`cascade`, recursively resolving child formworks).
///
/// | Slot Kind | Leaf (Non-Cascading) | Cascading (`cascade`) |
/// | :--- | :--- | :--- |
/// | **Single** | `#[slot]` | `#[slot(cascade)]` |
/// | **Option** | `#[slot(option)]` | `#[slot(option, cascade)]` |
/// | **Collection** | `#[slot(collection)]` | `#[slot(collection, cascade)]` |
/// | **Pick** | `#[slot(pick)]` | `#[slot(pick, cascade)]` |
///
/// # Slot Kinds Reference
///
/// | Slot Attribute | Field Type | Wet Formwork Slot Type | Resolves To | Description |
/// | :--- | :--- | :--- | :--- | :--- |
/// | `#[slot]` | `T` | `SingleSlot<T, T>` | `T` | Single slot for a non-formwork type. Accepts `T` or `SingleChoice<T>`. |
/// | `#[slot(option)]` | `Option<T>` | `Option<SingleSlot<T, T>>` | `Option<T>` | Optional slot for non-formwork items. Accepts individual items, `SingleChoice<T>`, or `SingleSlot`. |
/// | `#[slot(collection)]` | `Vec<T>` | `Vec<SingleSlot<T, T>>` | `Vec<T>` | Collection slot for non-formwork items. Accepts individual items, `SingleChoice<T>`, or `SingleSlot`. |
/// | `#[slot(pick)]` | `Vec<T>` | `MultiSlot<T, T>` | `Vec<T>` | Pick slot for selecting multiple items from candidates according to cardinality. Accepts `MultiChoice<T>` or `MultiSlot`. |
/// | `#[slot(cascade)]` | `Child` | `SingleSlot<ChildFormwork, Child>` | `Child` | Cascading single slot for a nested formwork type. Resolves the child when the parent resolves. |
/// | `#[slot(option, cascade)]` | `Option<Child>` | `Option<SingleSlot<ChildFormwork, Child>>` | `Option<Child>` | Cascading optional slot for nested formwork types. Resolves the child when present. |
/// | `#[slot(collection, cascade)]` | `Vec<Child>` | `Vec<SingleSlot<ChildFormwork, Child>>` | `Vec<Child>` | Cascading collection slot for nested formwork types. Recursively resolves all children. |
/// | `#[slot(pick, cascade)]` | `Vec<Child>` | `MultiSlot<ChildFormwork, Child>` | `Vec<Child>` | Cascading pick slot selecting child formworks subject to cardinality. |
///
/// # End-to-End Walkthrough Example
///
/// ```rust
/// use formwork::{formwork, Formwork, choice::SingleChoice};
///
/// #[formwork(builder)]
/// #[derive(Clone, Debug, PartialEq)]
/// pub struct HeroClass {
///     pub name: String,
///     #[slot]
///     pub primary_weapon: String,
///     #[slot(collection)]
///     pub proficiencies: Vec<String>,
/// }
///
/// // 1. Construct a wet formwork with choices
/// let mut wet_hero: HeroClassFormwork = HeroClass::builder()
///     .name("Fighter")
///     .primary_weapon(SingleChoice::new("Choose martial weapon", vec![
///         "Longsword".to_string(),
///         "Battleaxe".to_string(),
///     ]))
///     .with_proficiencies("Shields".to_string())
///     .build();
///
/// // 2. Formwork is incomplete because a choice is pending
/// assert!(!wet_hero.is_complete());
///
/// // 3. Satisfy choices across the hierarchy using a visitor
/// wet_hero.walk_choices(&mut formwork::visitor::AutoSelectFirst);
/// assert!(wet_hero.is_complete());
///
/// // 4. Resolve into the watertight dry domain model
/// let dry_hero: HeroClass = wet_hero.resolve().unwrap();
/// assert_eq!(dry_hero.name, "Fighter");
/// assert_eq!(dry_hero.primary_weapon, "Longsword");
/// assert_eq!(dry_hero.proficiencies, vec!["Shields".to_string()]);
/// ```
#[proc_macro_attribute]
pub fn formwork(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as DeriveInput);

    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };

    let mut args = match ast::FormworkArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    for attr in &item_struct.attrs {
        if attr.path().is_ident("derive")
            && let Ok(meta) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
        {
            for path in meta {
                let path_str = quote::quote!(#path).to_string();
                if !args
                    .derives
                    .iter()
                    .any(|p| quote::quote!(#p).to_string() == path_str)
                {
                    args.derives.push(path);
                }
            }
        }
    }

    let ast_struct = match ast::FormworkStruct::parse_input(&item_struct) {
        Ok(ast) => ast,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    // Validate fields
    if let darling::ast::Data::Struct(fields) = &ast_struct.data {
        for field in &fields.fields {
            if let Err(e) = field.slot_config() {
                return TokenStream::from(e.write_errors());
            }
        }
    }

    let original_attrs = &item_struct.attrs;

    let modes = gen_mode::generate_modes(&ast_struct, &args);
    let st = gen_struct::generate_struct(&ast_struct, original_attrs, &args);
    let fw = gen_formwork::generate_formwork(&ast_struct);
    let vis = gen_visitor::generate_visitor(&ast_struct, &args);

    let expanded = quote::quote! {
        #modes
        #st
        #fw
        #vis
    };

    expanded.into()
}

/// Internal helper attribute used by `#[formwork]` to qualify mode types for typed builders.
#[doc(hidden)]
#[proc_macro_attribute]
pub fn __qualify_mode(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mode_trait = parse_macro_input!(attr as syn::Ident);
    let mut item_struct = parse_macro_input!(item as DeriveInput);
    if let syn::Data::Struct(ref mut data_struct) = item_struct.data {
        for field in &mut data_struct.fields {
            if let syn::Type::Path(ref type_path) = field.ty
                && type_path.qself.is_none()
                && type_path.path.segments.len() == 2
                && type_path.path.segments[0].ident == "M"
            {
                let assoc = &type_path.path.segments[1].ident;
                field.ty = syn::parse_quote!(<M as #mode_trait>::#assoc);
            }
        }
    }
    quote::quote!(#item_struct).into()
}
