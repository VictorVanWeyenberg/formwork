#![warn(missing_docs)]

//! Procedural macros for the Higher-Kinded Data (HKD) formwork architecture.
//!
//! Provides the primary [`#[formwork]`](macro@formwork) attribute macro to transform struct
//! and enum definitions into dual-mode Higher-Kinded Data structures.

use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod ast;
mod gen_convert;
mod gen_enum;
mod gen_formwork;
mod gen_mode;
mod gen_struct;
mod gen_visitor;

/// Transforms a struct or enum into a Higher-Kinded Data (HKD) formwork type.
///
/// Generates a dual-mode type:
/// - **Dry domain model (`TypeName`)**: The watertight, cured domain struct or enum with concrete types.
/// - **Wet formwork (`TypeNameFormwork`)**: The mutable mold implementing `Formwork`,
///   holding pending choices, defaults, and unconfirmed values in stateful slots or nested formworks.
///
/// # Macro Attributes
///
/// - `builder` *(structs only)*: Generates compile-time checked builders for both wet and dry types.
/// - `traverse = "bfs" | "dfs"` *(structs only)*: Choice traversal strategy for `.walk_choices()` (defaults to `"bfs"`).
/// - `derives(...)`: Additional derives forwarded to internal mode marker types.
///
/// # Struct Field Attributes
///
/// Fields without attributes remain fixed in both modes. Slot fields use `#[slot(...)]`:
///
/// - `#[slot]`: Single scalar slot (`T` or `SingleChoice<T>`).
/// - `#[slot(option)]`: Optional slot (`Option<T>`).
/// - `#[slot(collection)]`: Ordered list of slot items (`Vec<T>`).
/// - `#[slot(pick)]`: Multi-item selection slot with cardinality constraints (`Vec<T>`).
/// - `#[slot(freeform)]`: Freeform slot allowing arbitrary constructed values or defaults (`T` or `Freeform<T>`).
/// - `#[slot(freeform, option)]`: Optional freeform slot (`Option<T>`).
/// - `#[slot(freeform, collection)]`: Ordered collection of freeform slot items (`Vec<T>`).
/// - `cascade`: Add to non-freeform slots (e.g. `#[slot(cascade)]`, `#[slot(collection, cascade)]`)
///   to recursively traverse and resolve nested child formworks.
///
/// # Enums
///
/// `#[formwork]` also supports non-empty enums whose variants are exclusively single-field tuple
/// variants wrapping distinct formwork types (e.g. `Variant(InnerFormworkType)`).
/// Enum formworks transparently delegate `Formwork` and `Walkable` to their active variant, and
/// generate `From` and builder slot-conversion implementations so inner variant types can be
/// passed directly into parent `#[formwork(builder)]` methods.
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
    let item_input = parse_macro_input!(item as DeriveInput);

    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };

    let mut args = match ast::FormworkArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    for attr in &item_input.attrs {
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

    match &item_input.data {
        syn::Data::Struct(_) => {
            let ast_struct = match ast::FormworkStruct::parse_input(&item_input) {
                Ok(ast) => ast,
                Err(e) => return TokenStream::from(e.write_errors()),
            };

            if let darling::ast::Data::Struct(fields) = &ast_struct.data {
                for field in &fields.fields {
                    if let Err(e) = field.slot_config() {
                        return TokenStream::from(e.write_errors());
                    }
                }
            }

            let original_attrs = &item_input.attrs;

            let modes = gen_mode::generate_modes(&ast_struct, &args);
            let st = gen_struct::generate_struct(&ast_struct, original_attrs, &args);
            let fw = gen_formwork::generate_formwork(&ast_struct);
            let vis = gen_visitor::generate_visitor(&ast_struct, &args);

            quote::quote! {
                #modes
                #st
                #fw
                #vis
            }
            .into()
        }
        syn::Data::Enum(_) => {
            let ast_enum = match ast::FormworkEnum::parse_input(&item_input, &args) {
                Ok(ast) => ast,
                Err(e) => return TokenStream::from(e.to_compile_error()),
            };

            gen_enum::generate_enum(&ast_enum, &item_input.attrs, &args).into()
        }
        syn::Data::Union(_) => TokenStream::from(
            darling::Error::custom("`#[formwork]` only supports structs and enums, not unions")
                .with_span(&item_input.ident)
                .write_errors(),
        ),
    }
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
            } else if let syn::Type::Path(ref type_path) = field.ty
                && type_path.qself.is_none()
                && type_path.path.segments.len() == 1
                && type_path.path.segments[0].ident == "Option"
                && let syn::PathArguments::AngleBracketed(ref args) =
                    type_path.path.segments[0].arguments
                && args.args.len() == 1
                && let syn::GenericArgument::Type(syn::Type::Path(inner_path)) =
                    &args.args[0]
                && inner_path.qself.is_none()
                && inner_path.path.segments.len() == 2
                && inner_path.path.segments[0].ident == "M"
            {
                let assoc = &inner_path.path.segments[1].ident;
                field.ty = syn::parse_quote!(Option<<M as #mode_trait>::#assoc>);
            }
        }
    }
    quote::quote!(#item_struct).into()
}
