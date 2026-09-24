use crate::slot::{SlotField, SlotKind};
use crate::{format_final_name, format_mode_name, generate_assoc_type_name};
use quote::{quote, ToTokens};
use syn::{Attribute, DataStruct, Ident, Visibility};

pub fn generate_struct(
    attrs: &Vec<Attribute>,
    vis: &Visibility,
    name: &Ident,
    fields: &Vec<SlotField>,
) -> proc_macro2::TokenStream {
    let mode_name = format_mode_name(name);
    let final_name = format_final_name(name);
    let fields = generate_fields(&mode_name, fields);
    quote! {
        #(#attrs
        )*
        #vis struct #name<M: #mode_name = #final_name> {
            #(#fields,
            )*
        }
    }
}

fn generate_fields(mode_name: &Ident, fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| generate_field(mode_name, field))
        .collect()
}

fn generate_field(mode_name: &Ident, field: &SlotField) -> proc_macro2::TokenStream {
    let vis = &field.field.vis;
    let field_ident = &field.field.ident.clone().unwrap();
    let assoc_type_name = generate_assoc_type_name(&field.field);
    match &field.slot_kind {
        None => field.field.clone().into_token_stream(),
        Some(slot_kind) => match slot_kind {
            SlotKind::Slot { .. } | SlotKind::SlotTransitive { .. } => {
                quote!(#vis #field_ident: <M as #mode_name>::#assoc_type_name)
            }
            SlotKind::Slots { .. } | SlotKind::SlotsTransitive { .. } => {
                quote!(#vis #field_ident: Vec<<M as #mode_name>::#assoc_type_name>)
            }
        },
    }
}
