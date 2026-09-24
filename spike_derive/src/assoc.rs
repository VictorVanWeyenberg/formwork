use convert_case::ccase;
use quote::{format_ident, quote};
use crate::generate_field_assoc_type_lvalue;
use crate::slot::SlotField;

pub fn generate_field_trait_assoc_types(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields.iter()
        .filter_map(generate_field_trait_assoc_type)
        .collect()
}

fn generate_field_trait_assoc_type(field: &SlotField) -> Option<proc_macro2::TokenStream> {
    if field.slot_kind == None {
        return None;
    }
    let lvalue = generate_field_assoc_type_lvalue(&field.field);
    let field_type = &field.field.ty;
    Some(quote! {
        #lvalue: crate::blueprint::Node + From<#field_type>;
    })
}