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
use crate::draft_assoc::generate_field_draft_assoc_types;
use crate::final_assoc::generate_field_final_assoc_types;
use crate::blueprint_impl::generate_blueprint_impl;
use crate::slot::{SlotField, SlotKind};
use darling::{FromDeriveInput, FromField};
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{format_ident, quote, ToTokens};
use std::ops::Not;
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Attribute, Data, DataStruct, DeriveInput,
    Error, Field, Fields, FieldsNamed, GenericArgument, GenericParam, Generics, Ident, Meta,
    PathArguments, PathSegment, Type, TypePath, Visibility,
};

mod alter_input;
mod assoc;
mod blueprint_impl;
mod draft_assoc;
mod final_assoc;
mod slot;

struct AlteredInput {
    tokens: proc_macro2::TokenStream,
    plain: Vec<Ident>,
    slots: Vec<Ident>,
}

struct BlueprintStruct {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    fields: Vec<SlotField>,
}

#[proc_macro_attribute]
pub fn blueprint(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let vis = &input.vis;
    let attrs = &input.attrs;
    let name = &input.ident;
    let fields = verify_input(&input)?;
    let draft_name = format_draft_name(name);
    let final_name = format_final_name(name);

    let altered_input = generate_struct(attrs, vis, name, &fields);
    let mode_tokens = generate_mode_tokens(vis, name, &fields)?;
    let blueprint_impl = generate_blueprint_impl(name, &fields);

    Ok(quote! {
        #altered_input
        #mode_tokens
        #blueprint_impl
    })
}

fn generate_mode_tokens(
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
    Ok(quote! {
        #vis trait #mode_name {
            #(#trait_assoc_types
            )*
        }
        #vis struct #draft_name;
        #vis struct #final_name;
        impl #mode_name for #draft_name {
            #(#draft_assoc_types
            )*
        }
        impl #mode_name for #final_name {
            #(#final_assoc_types
            )*
        }
        #vis type #blueprint_name = #name<#draft_name>;
    })
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
            _ => {
                return Err(Error::new_spanned(
                    &input.ident,
                    "#[derive(Blueprint)] requires named fields",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input.ident,
                "#[derive(Blueprint)] only supports structs",
            ))
        }
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
