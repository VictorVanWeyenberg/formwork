use crate::generate_field_assoc_type_lvalue;
use crate::slot::{get_vec_generic_type, SlotField, SlotKind};
use quote::{quote, ToTokens};
use syn::GenericArgument;

pub fn generate_field_trait_assoc_types(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter_map(generate_field_trait_assoc_type)
        .collect()
}

fn generate_field_trait_assoc_type(field: &SlotField) -> Option<proc_macro2::TokenStream> {
    if let Some(slot_kind) = &field.slot_kind {
        let lvalue = generate_field_assoc_type_lvalue(&field.field);
        let field_type = match slot_kind {
            SlotKind::Slot { ty, .. } => ty.to_token_stream(),
            SlotKind::SlotTransitive { generic_type_name, .. } => field.field.ty.to_token_stream(),
            SlotKind::Slots { generic_argument } => generic_argument.to_token_stream(),
            SlotKind::SlotsTransitive { generic_type_name, .. } => generic_type_name.to_token_stream(),
        };
        Some(quote! {
            #lvalue: crate::blueprint::Node;
        })
    } else {
        None
    }
}
