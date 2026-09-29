//! `#[derive(Blueprint)]` skeleton (S0 item 6, ticket T7).
//!
//! Input:  `struct X<M: Mode = Final> { plain: P, .., slots: Vec<M::Kind>, .. }`
//!         (`Vec<<M as Mode>::Kind>` is accepted too — typed-builder needs that spelling)
//! Output: `impl Blueprint for X<Draft>`, `impl From<X<Final>> for X<Draft>` (lift) and
//!         `impl From<X<Final>> for Slot<X<Draft>>`.
//!
//! A *slot field* is exactly a field typed `Vec<M::Kind>`; any other use of `M` is an error.
//! Paths are resolved against `crate::blueprint`, i.e. the derive is meant to be used inside the
//! crate that defines the blueprint machinery (a `#[blueprint(crate = "...")]` attribute would
//! lift that restriction later).

use crate::alter_input::generate_struct;
use crate::assoc::generate_field_trait_assoc_types;
use crate::blueprint_impl::generate_blueprint_impl;
use crate::draft_assoc::generate_field_draft_assoc_types;
use crate::final_assoc::generate_field_final_assoc_types;
use crate::slot::SlotField;
use darling::util::PathList;
use darling::{FromAttributes, FromMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Error, Field, Fields, FieldsNamed, Ident, Visibility,
};

mod alter_input;
mod assoc;
mod blueprint_impl;
mod draft_assoc;
mod final_assoc;
mod slot;

#[derive(Debug, Default, FromMeta)]
struct BlueprintArgs {
    #[darling(default)]
    pub derive: PathList,
}

#[proc_macro_attribute]
pub fn blueprint(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = if attr.is_empty() {
        vec![]
    } else {
        let attr = parse_macro_input!(attr as darling::ast::NestedMeta);
        match BlueprintArgs::from_list(&[attr.clone()]) {
            Ok(args) => args.derive.to_vec(),
            Err(err) => return Error::new_spanned(attr, format!("Darling error: {:?}", err))
                .into_compile_error()
                .into(),
        }
    };
    expand(args, parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(
    derives: Vec<syn::Path>,
    input: DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let vis = &input.vis;
    let attrs = &input.attrs;
    let name = &input.ident;
    let fields = verify_input(&input)?;

    let altered_input = generate_struct(attrs, vis, name, &fields);
    let mode_tokens = generate_mode_tokens(derives, vis, name, &fields)?;
    let blueprint_impl = generate_blueprint_impl(name, &fields);

    Ok(quote! {
        #altered_input
        #mode_tokens
        #blueprint_impl
    })
}

fn generate_mode_tokens(
    derives: Vec<syn::Path>,
    vis: &Visibility,
    name: &Ident,
    fields: &Vec<SlotField>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mode_name = format_mode_name(&name);
    let draft_name = format_draft_name(&name);
    let final_name = format_final_name(&name);
    let blueprint_name = format_blueprint_name(&name);
    let draft_assoc_types = generate_field_draft_assoc_types(fields);
    let final_assoc_types = generate_field_final_assoc_types(fields);
    let trait_assoc_types = generate_field_trait_assoc_types(fields);
    let derives = if derives.is_empty() {
        quote! {}
    } else {
        quote! { #[derive(#(#derives), *)] }
    };
    Ok(
        quote! {
            #vis trait #mode_name {
                #(#trait_assoc_types)*
            }
            #derives
            #vis struct #draft_name;
            #derives
            #vis struct #final_name;
            impl #mode_name for #draft_name {
                #(#draft_assoc_types)*
            }
            impl #mode_name for #final_name {
                #(#final_assoc_types)*
            }
            #vis type #blueprint_name = #name<#draft_name>;
        }
    )
}

fn format_blueprint_name(name: &Ident) -> Ident {
    format_ident!("{}Blueprint", name)
}

fn format_final_name(name: &Ident) -> Ident {
    format_ident!("{}Final", name)
}

fn format_draft_name(name: &Ident) -> Ident {
    format_ident!("{}Draft", name)
}

fn format_mode_name(name: &Ident) -> Ident {
    format_ident!("{}Mode", name)
}

fn verify_input(input: &DeriveInput) -> syn::Result<Vec<SlotField>> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "Attribute `blueprint` does not support generic parameters",
        ));
    }
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(FieldsNamed { named, .. }) => named
                .iter()
                .cloned()
                .map(|field| field.try_into())
                .collect::<syn::Result<Vec<_>>>(),
            _ => Err(Error::new_spanned(
                &input.ident,
                "#[derive(Blueprint)] requires named fields",
            )),
        },
        _ => Err(Error::new_spanned(
            &input.ident,
            "#[derive(Blueprint)] only supports structs",
        )),
    }
}

fn generate_assoc_type_name(field: &Field) -> proc_macro2::TokenStream {
    let name = format_ident!(
        "{}",
        convert_case::ccase!(pascal, &field.ident.clone().unwrap().to_string())
    );
    quote! { #name }
}

fn generate_field_assoc_type_lvalue(field: &Field) -> proc_macro2::TokenStream {
    let name = generate_assoc_type_name(field);
    quote! { type #name }
}

fn generate_field_assoc_type_assignment(
    field: &Field,
    assign_to: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let lvalue = generate_field_assoc_type_lvalue(field);
    quote! { #lvalue = #assign_to; }
}
