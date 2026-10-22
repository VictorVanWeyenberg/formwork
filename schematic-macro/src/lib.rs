use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod ast;
mod gen_mode;
mod gen_schematic;
mod gen_struct;
mod gen_visitor;

/// Transforms a struct into a Higher-Kinded Data (HKD) schematic type.
///
/// `#[schematic]` generates an unresolved blueprint alongside a watertight resolved domain
/// model from a single struct definition.
///
/// # Higher-Kinded Data (HKD) Architecture
///
/// In domain systems like RPG character creators, order configurators, form builders, and
/// game engines, data starts out incomplete: users or game rules must make decisions (such
/// as selecting one or more proficiencies, tools, feats, or subraces).
///
/// Traditional solutions suffer from major drawbacks:
/// - **Excessive Option Wrapping**: Wrapping fields in `Option<T>` forces the final domain model
///   to permanently carry `Option` types, requiring runtime unwraps and leaving the door open to invalid states.
/// - **Dual Model Duplication**: Defining twin structs (e.g. `CharacterBlueprint` and `Character`)
///   leads to thousands of lines of fragile manual mapping and maintenance boilerplate.
///
/// `#[schematic]` solves this by generating a single struct parameterized over a mode `M`:
///
/// ```rust,ignore
/// pub struct Character<M: CharacterMode = CharacterResolved> {
///     // ...
/// }
/// ```
///
/// - **Resolved Domain Model (`Character`)**: Unparameterized, `Character` defaults to resolved mode.
///   All fields contain their watertight, concrete types (`T`, `Vec<T>`). No choices or unresolved slots exist.
/// - **Unresolved Blueprint (`CharacterSchematic`)**: A type alias for `Character<CharacterUnresolved>`.
///   Slot fields wrap values in stateful slots capable of holding pending choices, made choices with history,
///   or fixed values.
/// - **Resolution**: Blueprints implement the `Schematic` trait.
///   Calling `resolve()` verifies that all choices across the entire hierarchy
///   are satisfied and produces the watertight domain model.
///
/// # Configuration Attributes
///
/// Pass options directly into `#[schematic(...)]`:
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
/// When designing a schematic struct, consider whether each field should be a slot:
///
/// - **Normal Field (no `#[slot]`)**: The value is known unconditionally upfront at construction time.
///   It cannot be deferred, picked from choices, or altered by visitors (e.g. an ID, a static title, metadata).
/// - **Slot Field (`#[slot]`)**: The field may involve choices, deferred selection, or hierarchical resolution.
///
/// # 2x3 Orthogonal Slot Matrix
///
/// Slot attributes are designed along two orthogonal dimensions:
/// - **Cardinality / Multiplicity**: Single (scalar value), Collection (arbitrary items or choices), or Pick (pool of candidates with cardinality constraints).
/// - **Hierarchy**: Leaf (resolves to concrete data directly) vs. Cascading (`cascade`, recursively resolving child schematics).
///
/// | Slot Kind | Leaf (Non-Cascading) | Cascading (`cascade`) |
/// | :--- | :--- | :--- |
/// | **Single** | `#[slot]` | `#[slot(cascade)]` |
/// | **Collection** | `#[slot(collection)]` | `#[slot(collection, cascade)]` |
/// | **Pick** | `#[slot(pick)]` | `#[slot(pick, cascade)]` |
///
/// # Slot Kinds Reference
///
/// | Slot Attribute | Field Type | Unresolved Blueprint Slot Type | Resolves To | Description |
/// | :--- | :--- | :--- | :--- | :--- |
/// | `#[slot]` | `T` | `SingleSlot<T, T>` | `T` | Single slot for a non-schematic type. Accepts `T` or `SingleChoice<T>`. |
/// | `#[slot(collection)]` | `Vec<T>` | `Vec<SingleSlot<T, T>>` | `Vec<T>` | Collection slot for non-schematic items. Accepts individual items, `SingleChoice<T>`, or `SingleSlot`. |
/// | `#[slot(pick)]` | `Vec<T>` | `MultiSlot<T, T>` | `Vec<T>` | Pick slot for selecting multiple items from candidates according to cardinality. Accepts `MultiChoice<T>` or `MultiSlot`. |
/// | `#[slot(cascade)]` | `Child` | `SingleSlot<ChildSchematic, Child>` | `Child` | Cascading single slot for a nested schematic type. Resolves the child when the parent resolves. |
/// | `#[slot(collection, cascade)]` | `Vec<Child>` | `Vec<SingleSlot<ChildSchematic, Child>>` | `Vec<Child>` | Cascading collection slot for nested schematic types. Recursively resolves all children. |
/// | `#[slot(pick, cascade)]` | `Vec<Child>` | `MultiSlot<ChildSchematic, Child>` | `Vec<Child>` | Cascading pick slot selecting child schematics subject to cardinality. |
///
/// # End-to-End Walkthrough Example
///
/// ```rust
/// use schematic::{schematic, Schematic, choice::SingleChoice};
///
/// #[schematic(builder)]
/// #[derive(Clone, Debug, PartialEq)]
/// pub struct HeroClass {
///     pub name: String,
///     #[slot]
///     pub primary_weapon: String,
///     #[slot(collection)]
///     pub proficiencies: Vec<String>,
/// }
///
/// // 1. Construct an unresolved blueprint with choices
/// let mut blueprint: HeroClassSchematic = HeroClass::builder()
///     .name("Fighter")
///     .primary_weapon(SingleChoice::new("Choose martial weapon", vec![
///         "Longsword".to_string(),
///         "Battleaxe".to_string(),
///     ]))
///     .with_proficiencies("Shields".to_string())
///     .build();
///
/// // 2. Blueprint is incomplete because a choice is pending
/// assert!(!blueprint.is_complete());
///
/// // 3. Satisfy choices across the hierarchy using a visitor
/// blueprint.walk_choices(&mut schematic::visitor::AutoSelectFirst);
/// assert!(blueprint.is_complete());
///
/// // 4. Resolve into the watertight domain model
/// let resolved: HeroClass = blueprint.resolve().unwrap();
/// assert_eq!(resolved.name, "Fighter");
/// assert_eq!(resolved.primary_weapon, "Longsword");
/// assert_eq!(resolved.proficiencies, vec!["Shields".to_string()]);
/// ```
#[proc_macro_attribute]
pub fn schematic(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as DeriveInput);

    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };

    let mut args = match ast::SchematicArgs::from_list(&attr_args) {
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

    let ast_struct = match ast::SchematicStruct::parse_input(&item_struct) {
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
    let bp = gen_schematic::generate_schematic(&ast_struct);
    let vis = gen_visitor::generate_visitor(&ast_struct, &args);

    let expanded = quote::quote! {
        #modes
        #st
        #bp
        #vis
    };

    expanded.into()
}

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

