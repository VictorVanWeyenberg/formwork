use crate::slot::{expect_none_generic_generic_argument, SlotField, SlotKind};
use crate::{format_draft_name, format_final_name};
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Field, GenericArgument};

pub fn generate_blueprint_impl(name: &Ident, fields: &Vec<SlotField>) -> proc_macro2::TokenStream {
    let draft_name = format_draft_name(name);
    let final_name = format_final_name(name);
    let bp = quote!(crate::blueprint);
    let field_idents = extract_field_idents(fields);
    let field_resolutions = generate_field_resolutions(fields);
    let visible_choice_extends = generate_visible_choice_extends(fields);
    let next_open_choice_checks = generate_next_open_choice_checks(fields);
    let final_to_draft_fields = generate_final_to_draft_fields(fields);
    let transitive_slot_conversions = generate_transitive_slot_conversions(fields);

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
                #(#visible_choice_extends)*
                out
            }

            fn next_open_choice_mut(&mut self) -> ::core::option::Option<&mut dyn #bp::ChoicePrompt> {
                #(#next_open_choice_checks)*
                ::core::option::Option::None
            }
        }

        #[automatically_derived]
        impl ::core::convert::From<#name<#final_name>> for #name<#draft_name> {
            fn from(value: #name<#final_name>) -> Self {
                let #name { #(#field_idents, )* } = value;
                #name {
                    #(#final_to_draft_fields,)*
                }
            }
        }

        #(#transitive_slot_conversions)*
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

fn generate_visible_choice_extends(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter_map(some_slot_kind_unpack)
        .map(|(slot_kind, field)| {
            let field_name = field.ident.as_ref().unwrap();
            match slot_kind {
                SlotKind::Slot { .. } | SlotKind::SlotTransitive { .. } => quote! {
                    out.extend(self.#field_name.visible_choices());
                },
                SlotKind::Slots { .. } | SlotKind::SlotsTransitive { .. } => quote! {
                    out.extend(self.#field_name.iter().flat_map(|x| x.visible_choices()));
                },
            }
        })
        .collect()
}

fn some_slot_kind_unpack(slot_field: &SlotField) -> Option<(&SlotKind, &Field)> {
    if slot_field.slot_kind.is_some() {
        Some((slot_field.slot_kind.as_ref().unwrap(), &slot_field.field))
    } else {
        None
    }
}

fn generate_next_open_choice_checks(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter_map(some_slot_kind_unpack)
        .map(|(slot_kind, field)| {
            let field_name = field.ident.as_ref().unwrap();
            match slot_kind {
                SlotKind::Slot { .. } | SlotKind::SlotTransitive { .. } => {
                    quote! {
                        if let ::core::option::Option::Some(choice) =
                            self.#field_name.next_open_choice_mut()
                        {
                            return ::core::option::Option::Some(choice);
                        }
                    }
                }
                SlotKind::Slots { .. } | SlotKind::SlotsTransitive { .. } => {
                    quote! {
                        if let ::core::option::Option::Some(choice) =
                            self.#field_name.iter_mut().find_map(|x| x.next_open_choice_mut())
                        {
                            return ::core::option::Option::Some(choice);
                        }
                    }
                }
            }
        })
        .collect()
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

fn generate_final_to_draft_fields(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(generate_final_to_draft_field)
        .collect()
}

fn generate_final_to_draft_field(
    slot_field: &SlotField
) -> proc_macro2::TokenStream {
    let ident = &slot_field.field.ident;
    if let Some(slot_kind) = &slot_field.slot_kind {
        match slot_kind {
            SlotKind::Slot { .. } | SlotKind::SlotTransitive { .. } => {
                quote! { #ident: #ident.into() }
            }
            SlotKind::Slots { .. } | SlotKind::SlotsTransitive { .. } => {
                quote! { #ident: #ident.into_iter().map(|x| x.into()).collect() }
            }
        }
    } else {
        quote! { #ident }
    }
}

fn generate_transitive_slot_conversions(fields: &Vec<SlotField>) -> Vec<proc_macro2::TokenStream> {
    fields.iter()
        .filter_map(some_slot_kind_unpack)
        .map(generate_transitive_slot_conversion)
        .collect()
}

fn generate_transitive_slot_conversion((slot_kind, _field): (&SlotKind, &Field)) -> proc_macro2::TokenStream {
    let bp = quote!(crate::blueprint);
    match slot_kind {
        SlotKind::Slot { .. } | SlotKind::Slots { .. } => quote! {},
        SlotKind::SlotTransitive { generic_type_name, draft_name, .. } | SlotKind::SlotsTransitive { generic_type_name, draft_name } => quote! {
            impl From<#generic_type_name> for #bp::SlotOneTransitive<#generic_type_name<#draft_name>> {
                fn from(value: #generic_type_name) -> Self {
                    #bp::SlotOneTransitive::Fixed(value.into())
                }
            }
            impl From<#bp::Choice<#generic_type_name>> for #bp::SlotOneTransitive<#generic_type_name<#draft_name>> {
                fn from(value: #bp::Choice<#generic_type_name>) -> Self {
                    #bp::SlotOneTransitive::Choose(value.into())
                }
            }
            impl From<#bp::Choice<#generic_type_name>> for #bp::Choice<#generic_type_name<#draft_name>> {
                fn from(value: #bp::Choice<#generic_type_name>) -> Self {
                    let #bp::Choice { cardinality, prompt, options, .. } = value;
                    let options = options.into_iter()
                        .map(|o| o.into())
                        .collect::<Vec<#bp::ChoiceOption<#generic_type_name<#draft_name>>>>();
                    #bp::Choice::new(cardinality, prompt, options)
                }
            }
            impl From<#bp::ChoiceOption<#generic_type_name>> for #bp::ChoiceOption<#generic_type_name<#draft_name>> {
                fn from(value: #bp::ChoiceOption<#generic_type_name>) -> Self {
                    let #bp::ChoiceOption { label, value } = value;
                    #bp::ChoiceOption::<#generic_type_name<#draft_name>>::new(label, value)
                }
            }
        },
    }
}
