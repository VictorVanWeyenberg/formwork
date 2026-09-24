use crate::generate_field_assoc_type_assignment;
use crate::slot::{SlotField, SlotKind};
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Field, GenericArgument, Type};

pub fn generate_field_final_assoc_types(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields.iter().map(generate_field_final_assoc_type).collect()
}

fn generate_field_final_assoc_type(slot_field: &SlotField) -> proc_macro2::TokenStream {
    match &slot_field.slot_kind {
        None => slot_field.field.clone().into_token_stream(),
        Some(slot_kind) => match slot_kind {
            SlotKind::Slot { ty } => generate_field_assoc_type_assignment(
                &slot_field.field,
                generate_slot_field_final_assoc_type(ty),
            ),
            SlotKind::SlotTransitive {
                generic_type_name,
                draft_name,
            } => generate_field_assoc_type_assignment(
                &slot_field.field,
                generate_slot_transitive_field_final_assoc_type(generic_type_name, draft_name),
            ),
            SlotKind::Slots { generic_argument } => generate_field_assoc_type_assignment(
                &slot_field.field,
                generate_slots_field_final_assoc_type(generic_argument),
            ),
            SlotKind::SlotsTransitive {
                generic_type_name,
                draft_name,
            } => generate_field_assoc_type_assignment(
                &slot_field.field,
                generate_slots_transitive_final_assoc_type(generic_type_name, draft_name),
            ),
        },
    }
}

fn generate_slot_field_final_assoc_type(ty: &Type) -> proc_macro2::TokenStream {
    quote! {
        #ty,
    }
}

fn generate_slots_field_final_assoc_type(arg: &GenericArgument) -> proc_macro2::TokenStream {
    quote! {
        #arg,
    }
}

fn generate_slot_transitive_field_final_assoc_type(
    name: &Ident,
    draft_name: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        #name,
    }
}

fn generate_slots_transitive_final_assoc_type(
    name: &Ident,
    draft_name: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        Vec<#name>,
    }
}
