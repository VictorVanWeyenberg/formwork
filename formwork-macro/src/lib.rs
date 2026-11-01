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
/// Generates a dual-mode type:
/// - **Dry domain model (`StructName`)**: The watertight, cured domain struct with concrete field types.
/// - **Wet formwork (`StructNameFormwork`)**: The mutable mold implementing `Formwork`,
///   holding pending choices, defaults, and unconfirmed values in stateful slots.
///
/// # Macro Attributes
///
/// - `builder`: Generates compile-time checked builders for both wet and dry types.
/// - `traverse = "bfs" | "dfs"`: Choice traversal strategy for `.walk_choices()` (defaults to `"bfs"`).
/// - `derives(...)`: Additional derives forwarded to internal mode marker types.
///
/// # Field Attributes
///
/// Fields without attributes remain fixed in both modes. Slot fields use `#[slot(...)]`:
///
/// - `#[slot]`: Single scalar slot (`T` or `SingleChoice<T>`).
/// - `#[slot(option)]`: Optional slot (`Option<T>`).
/// - `#[slot(collection)]`: Ordered list of slot items (`Vec<T>`).
/// - `#[slot(pick)]`: Multi-item selection slot with cardinality constraints (`Vec<T>`).
/// - `cascade`: Add to any slot (e.g. `#[slot(cascade)]`, `#[slot(collection, cascade)]`)
///   to recursively traverse and resolve nested child formworks.
///
/// # Example
///
/// ```rust
/// use formwork::{formwork, Formwork, choice::SingleChoice};
///
/// #[formwork(builder)]
/// pub struct Character {
///     pub name: String,
///     #[slot]
///     pub class: String,
/// }
///
/// let mut formwork = Character::builder()
///     .name("Conan")
///     .class(SingleChoice::new("Pick Class", vec!["Barbarian".to_string(), "Fighter".to_string()]))
///     .build();
///
/// formwork.walk_choices(&mut formwork::visitor::AutoSelectFirst);
/// let hero: Character = formwork.resolve().unwrap();
/// assert_eq!(hero.class, "Barbarian");
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
