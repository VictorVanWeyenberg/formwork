use crate::slot::{SlotField, SlotKind};
use crate::{format_draft_name, format_final_name};
use proc_macro2::Ident;
use quote::{quote, ToTokens};

pub fn generate_blueprint_impl(name: &Ident, fields: &Vec<SlotField>) -> proc_macro2::TokenStream {
    let draft_name = format_draft_name(name);
    let final_name = format_final_name(name);
    let bp = quote!(crate::blueprint);
    let field_idents = extract_field_idents(fields);
    let field_resolutions = generate_field_resolutions(fields);
    let plain_idents = extract_plain_idents(fields);
    let slot_idents = extract_slot_idents(fields);
    let slot_one_idents = extract_slot_one_idents(fields);
    let slot_many_idents = extract_slot_many_idents(fields);

    quote! {
        #[automatically_derived]
        impl #bp::Blueprint for #name<#draft_name> {
            type Final = #name<#final_name>;

            fn resolve(self) -> ::core::result::Result<Self::Final, #bp::ChoiceError> {
                let #name { #(#field_idents, )* } = self;
                ::core::result::Result::Ok(#name {
                    #(#field_resolutions,)*
                })
            }

            fn visible_choices(&self) -> ::std::vec::Vec<&dyn #bp::ChoicePrompt> {
                let mut out: ::std::vec::Vec<&dyn #bp::ChoicePrompt> = ::std::vec::Vec::new();
                #( out.extend(self.#slot_one_idents.iter().flat_map(#bp::SlotOne::visible_choices)); )*
                #( out.extend(self.#slot_many_idents.iter().flat_map(#bp::SlotMany::visible_choices)); )*
                out
            }

            fn next_open_choice_mut(&mut self) -> ::core::option::Option<&mut dyn #bp::ChoicePrompt> {
                #(
                    if let ::core::option::Option::Some(choice) =
                        self.#slot_one_idents.iter_mut().find_map(#bp::SlotOne::next_open_choice_mut)
                    {
                        return ::core::option::Option::Some(choice);
                    }
                )*
                #(
                    if let ::core::option::Option::Some(choice) =
                        self.#slot_many_idents.iter_mut().find_map(#bp::SlotMany::next_open_choice_mut)
                    {
                        return ::core::option::Option::Some(choice);
                    }
                )*
                ::core::option::Option::None
            }
        }

        #[automatically_derived]
        impl ::core::convert::From<#name<#final_name>> for #name<#draft_name> {
            fn from(value: #name<#final_name>) -> Self {
                let #name { #(#plain_idents,)* #(#slot_idents,)* .. } = value;
                #name {
                    #(#plain_idents,)*
                    #(#slot_idents: #slot_idents.into_iter().map(::core::convert::Into::into).collect(),)*
                }
            }
        }
    }
}

fn extract_field_idents(fields: &Vec<SlotField>) -> Vec<&Ident> {
    fields
        .iter()
        .filter_map(|slot| slot.field.ident.as_ref())
        .collect()
}

fn generate_field_resolutions(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields.iter().map(generate_field_resolution).collect()
}

fn generate_field_resolution(field: &SlotField) -> proc_macro2::TokenStream {
    let field_ident = field.field.ident.clone().unwrap();
    match &field.slot_kind {
        None => field_ident.into_token_stream(),
        Some(slot_kind) => match slot_kind {
            SlotKind::Slot { .. } => quote!(#field_ident: #field_ident.resolve()?),
            SlotKind::SlotTransitive { .. } => quote!(#field_ident: #field_ident.resolve()?),
            SlotKind::Slots { .. } => {
                quote!(#field_ident: #field_ident.into_iter().map(|x| x.resolve()).collect::<Result<Vec<_>, _>>()?)
            }
            SlotKind::SlotsTransitive { .. } => {
                quote!(#field_ident: #field_ident.into_iter().map(|x| x.resolve()).collect::<Result<Vec<_>, _>>()?)
            }
        },
    }
}

fn extract_plain_idents(fields: &Vec<SlotField>) -> Vec<&Ident> {
    fields
        .iter()
        .filter(|field| field.slot_kind.is_none())
        .map(|field| field.field.ident.as_ref().unwrap())
        .collect()
}

fn extract_slot_idents(fields: &Vec<SlotField>) -> Vec<&Ident> {
    fields
        .iter()
        .filter(|field| field.slot_kind.is_some())
        .map(|field| field.field.ident.as_ref().unwrap())
        .collect()
}

fn extract_slot_one_idents(fields: &Vec<SlotField>) -> Vec<&Ident> {
    fields
        .iter()
        .filter(|field| {
            field.slot_kind.clone().is_some_and(|k| {
                matches!(k, SlotKind::Slot { .. } | SlotKind::SlotTransitive { .. })
            })
        })
        .map(|field| field.field.ident.as_ref().unwrap())
        .collect()
}

fn extract_slot_many_idents(fields: &Vec<SlotField>) -> Vec<&Ident> {
    fields
        .iter()
        .filter(|field| {
            field.slot_kind.clone().is_some_and(|k| {
                matches!(k, SlotKind::Slot { .. } | SlotKind::SlotTransitive { .. })
            })
        })
        .map(|field| field.field.ident.as_ref().unwrap())
        .collect()
}
