use quote::{quote, ToTokens};
use crate::{format_draft_name, format_final_name};
use proc_macro2::Ident;
use crate::slot::{SlotField, SlotKind};

pub fn generate_blueprint_impl(name: &Ident, fields: &Vec<SlotField>) -> proc_macro2::TokenStream {
    let draft_name = format_draft_name(name);
    let final_name = format_final_name(name);
    let bp = quote!(crate::blueprint);
    let field_idents = extract_field_idents(fields);
    let field_resolutions = generate_field_resolutions(fields);
    quote! {
        #[automatically_derived]
        impl #bp::Blueprint for #name<#draft_name> {
            type Final = #name<#final_name>;

            fn resolve(self) -> ::core::result::Result<Self::Final, #bp::ChoiceError> {
                let #name { #(#field_idents, )* } = self;
                ::core::result::Result::Ok(#name {
                    #(#plain,)*
                    #(#slots: #bp::resolve_slots(#slots)?,)*
                })
            }

            fn visible_choices(&self) -> ::std::vec::Vec<&dyn #bp::ChoicePrompt> {
                let mut out: ::std::vec::Vec<&dyn #bp::ChoicePrompt> = ::std::vec::Vec::new();
                #( out.extend(self.#slots.iter().flat_map(#bp::Slot::visible_choices)); )*
                out
            }

            fn next_open_choice_mut(&mut self) -> ::core::option::Option<&mut dyn #bp::ChoicePrompt> {
                #(
                    if let ::core::option::Option::Some(choice) =
                        self.#slots.iter_mut().find_map(#bp::Slot::next_open_choice_mut)
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
                let #name { #(#plain,)* #(#slots,)* .. } = value;
                #name {
                    #(#plain,)*
                    #(#slots: #slots.into_iter().map(::core::convert::Into::into).collect(),)*
                }
            }
        }

        #[automatically_derived]
        impl ::core::convert::From<#name<#final_name>> for #bp::Slot<#name<#draft_name>> {
            fn from(value: #name<#final_name>) -> Self {
                #bp::Slot::Fixed(::core::convert::Into::into(value))
            }
        }
    }
}

fn extract_field_idents(fields: &Vec<SlotField>) -> Vec<&Ident> {
    fields.iter()
        .filter_map(|slot| slot.field.ident.as_ref())
        .collect()
}

fn generate_field_resolutions(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields.iter()
        .map(generate_field_resolution)
        .collect()
}

fn generate_field_resolution(field: &SlotField) -> proc_macro2::TokenStream {
    let field_ident = field.field.ident.clone().unwrap();
    match &field.slot_kind {
        None => field_ident.into_token_stream(),
        Some(slot_kind) => match slot_kind {
            SlotKind::Slot { .. } => quote!(#field_ident: #field_ident.resolve()?),
            SlotKind::SlotTransitive { .. } => {}
            SlotKind::Slots { .. } => {}
            SlotKind::SlotsTransitive { .. } => {}
        }
    }
}