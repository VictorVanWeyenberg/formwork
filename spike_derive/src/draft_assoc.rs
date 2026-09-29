use crate::generate_field_assoc_type_assignment;
use crate::slot::{SlotField, SlotKind};
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{GenericArgument, Type};

pub fn generate_field_draft_assoc_types(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter_map(generate_field_draft_assoc_type)
        .collect()
}

fn generate_field_draft_assoc_type(slot_field: &SlotField) -> Option<proc_macro2::TokenStream> {
    match &slot_field.slot_kind {
        None => None,
        Some(slot_kind) => match slot_kind {
            SlotKind::Slot { ty, many } => Some(generate_field_assoc_type_assignment(
                &slot_field.field,
                generate_slot_field_draft_assoc_type(ty, many),
            )),
            SlotKind::SlotTransitive {
                generic_type_name,
                draft_name,
                many,
            } => Some(generate_field_assoc_type_assignment(
                &slot_field.field,
                generate_slot_transitive_field_draft_assoc_type(
                    generic_type_name,
                    draft_name,
                    many,
                ),
            )),
            SlotKind::Slots { generic_argument } => Some(generate_field_assoc_type_assignment(
                &slot_field.field,
                generate_slots_field_draft_assoc_type(generic_argument),
            )),
            SlotKind::SlotsTransitive {
                generic_type_name,
                draft_name,
            } => Some(generate_field_assoc_type_assignment(
                &slot_field.field,
                generate_slots_transitive_draft_assoc_type(generic_type_name, draft_name),
            )),
        },
    }
}

fn generate_slot_field_draft_assoc_type(ty: &Type, many: &bool) -> proc_macro2::TokenStream {
    if *many {
        quote! {
            crate::blueprint::SlotMany<#ty>
        }
    } else {
        quote! {
            crate::blueprint::SlotOne<#ty>
        }
    }
}

fn generate_slots_field_draft_assoc_type(arg: &GenericArgument) -> proc_macro2::TokenStream {
    quote! {
        crate::blueprint::SlotOne<#arg>
    }
}

fn generate_slot_transitive_field_draft_assoc_type(
    name: &Ident,
    draft_name: &Ident,
    many: &bool,
) -> proc_macro2::TokenStream {
    if *many {
        quote! {
            crate::blueprint::SlotManyTransitive<#name<#draft_name>>
        }
    } else {
        quote! {
            crate::blueprint::SlotOneTransitive<#name<#draft_name>>
        }
    }
}

fn generate_slots_transitive_draft_assoc_type(
    name: &Ident,
    draft_name: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        crate::blueprint::SlotOneTransitive<#name<#draft_name>>
    }
}
