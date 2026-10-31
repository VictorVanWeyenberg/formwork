use crate::ast::FormworkStruct;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub fn generate_formwork(ast: &FormworkStruct) -> TokenStream {
    let name = &ast.ident;
    let dry_struct = format_ident!("{}Dry", name);
    let formwork_type = format_ident!("{}Formwork", name);

    let mut complete_checks = Vec::new();
    let mut resolve_fields = Vec::new();
    let mut from_fields = Vec::new();

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            let field_name = field.ident.as_ref().unwrap();

            if field.is_slot() {
                let config = field.slot_config().unwrap().unwrap();
                match (config.collection, config.pick, config.option, config.cascade) {
                    (true, false, false, true) => {
                        complete_checks
                            .push(generate_cascade_collection_complete_check(field_name));
                        resolve_fields
                            .push(generate_cascade_collection_resolve_expression(field_name));
                        from_fields.push(generate_general_collection_conversion(field_name));
                    }
                    (true, false, false, false) => {
                        complete_checks.push(generate_collection_complete_check(field_name));
                        resolve_fields.push(generate_collection_resolve_expression(field_name));
                        from_fields.push(generate_general_collection_conversion(field_name));
                    }
                    (false, true, false, true) => {
                        complete_checks.push(generate_cascade_pick_complete_check(field_name));
                        resolve_fields.push(generate_cascade_pick_resolve_expression(field_name));
                        from_fields.push(generate_general_slot_conversion(field_name));
                    }
                    (false, true, false, false) => {
                        complete_checks.push(generate_slot_complete_check(field_name));
                        resolve_fields.push(generate_pick_resolve_expression(field_name));
                        from_fields.push(generate_general_slot_conversion(field_name));
                    }
                    (false, false, true, true) => {
                        complete_checks
                            .push(generate_cascade_option_complete_check(field_name));
                        resolve_fields
                            .push(generate_cascade_option_resolve_expression(field_name));
                        from_fields.push(generate_option_slot_conversion(field_name));
                    }
                    (false, false, true, false) => {
                        complete_checks.push(generate_option_complete_check(field_name));
                        resolve_fields.push(generate_option_resolve_expression(field_name));
                        from_fields.push(generate_option_slot_conversion(field_name));
                    }
                    (false, false, false, true) => {
                        complete_checks.push(generate_cascade_slot_complete_check(field_name));
                        resolve_fields.push(generate_cascade_slot_resolve_expression(field_name));
                        from_fields.push(generate_general_slot_conversion(field_name));
                    }
                    (false, false, false, false) => {
                        complete_checks.push(generate_slot_complete_check(field_name));
                        resolve_fields.push(generate_slot_resolve_expression(field_name));
                        from_fields.push(generate_general_slot_conversion(field_name));
                    }
                    _ => {}
                }
            } else {
                resolve_fields.push(quote! { #field_name: self.#field_name });
                from_fields.push(quote! { #field_name: value.#field_name });
            }
        }
    }

    let is_complete_body = if complete_checks.is_empty() {
        quote! { true }
    } else {
        quote! { #(#complete_checks)&&* }
    };

    quote! {
        impl formwork::Formwork for #formwork_type {
            type Dry = #name<#dry_struct>;

            fn is_complete(&self) -> bool {
                #is_complete_body
            }

            fn resolve(self) -> Result<Self::Dry, formwork::error::FormworkError> {
                Ok(#name {
                    #(#resolve_fields),*
                })
            }
        }

        impl From<#name<#dry_struct>> for #formwork_type {
            fn from(value: #name<#dry_struct>) -> Self {
                Self {
                    #(#from_fields),*
                }
            }
        }
    }
}

fn generate_general_slot_conversion(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: formwork::Slot::FixedDry(value.#field_name)
    }
}

fn generate_option_slot_conversion(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: value.#field_name.map(formwork::Slot::FixedDry)
    }
}

fn generate_option_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: self.#field_name.map(formwork::SingleSlot::resolve_exact).transpose()?
    }
}

fn generate_option_complete_check(field_name: &Ident) -> TokenStream {
    quote! {
        self.#field_name.as_ref().map_or(true, |slot| slot.is_complete())
    }
}

fn generate_cascade_option_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: self.#field_name.map(|slot| match slot {
            formwork::Slot::FixedDry(d) => Ok(d),
            formwork::Slot::FixedWet(w) => w.resolve(),
            formwork::Slot::Choose(c) => c.resolve_exact()?.resolve(),
        }).transpose()?
    }
}

fn generate_cascade_option_complete_check(field_name: &Ident) -> TokenStream {
    quote! {
        self.#field_name.as_ref().map_or(true, |slot| match slot {
            formwork::Slot::FixedDry(_) => true,
            formwork::Slot::FixedWet(inner) => inner.is_complete(),
            formwork::Slot::Choose(choice) => choice.is_made() && choice.selected_value().is_some_and(|inner| inner.is_complete()),
        })
    }
}

fn generate_slot_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! { #field_name: self.#field_name.resolve_exact()? }
}

fn generate_slot_complete_check(field_name: &Ident) -> TokenStream {
    quote! { self.#field_name.is_complete() }
}

fn generate_cascade_slot_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: match self.#field_name {
            formwork::Slot::FixedDry(dry) => dry,
            formwork::Slot::FixedWet(wet) => wet.resolve()?,
            formwork::Slot::Choose(choice) => choice.resolve_exact()?.resolve()?,
        }
    }
}

fn generate_cascade_slot_complete_check(field_name: &Ident) -> TokenStream {
    quote! {
        (match &self.#field_name {
            formwork::Slot::FixedDry(_) => true,
            formwork::Slot::FixedWet(inner) => inner.is_complete(),
            formwork::Slot::Choose(choice) => choice.is_made() && choice.selected_value().is_some_and(|inner| inner.is_complete()),
        })
    }
}

fn generate_pick_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! { #field_name: self.#field_name.resolve()? }
}

fn generate_cascade_pick_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: match self.#field_name {
            formwork::Slot::FixedDry(d) => Ok(d),
            formwork::Slot::FixedWet(w) => w.into_iter().map(|item| item.resolve()).collect::<Result<Vec<_>, formwork::error::FormworkError>>(),
            formwork::Slot::Choose(c) => c.resolve()?.into_iter().map(|item| item.resolve()).collect::<Result<Vec<_>, formwork::error::FormworkError>>(),
        }?
    }
}

fn generate_cascade_pick_complete_check(field_name: &Ident) -> TokenStream {
    quote! {
        (match &self.#field_name {
            formwork::Slot::FixedDry(_) => true,
            formwork::Slot::FixedWet(inners) => inners.iter().all(|i| i.is_complete()),
            formwork::Slot::Choose(choice) => choice.is_made() && choice.selected_values().iter().all(|i| i.is_complete()),
        })
    }
}

fn generate_general_collection_conversion(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: value.#field_name.into_iter().map(formwork::Slot::FixedDry).collect()
    }
}

fn generate_collection_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: self.#field_name.into_iter()
            .map(formwork::SingleSlot::resolve_exact)
            .collect::<Result<Vec<_>, formwork::error::FormworkError>>()?
    }
}

fn generate_collection_complete_check(field_name: &Ident) -> TokenStream {
    quote! { self.#field_name.iter().all(|slot| slot.is_complete()) }
}

fn generate_cascade_collection_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: self.#field_name.into_iter()
            .map(|slot| match slot {
                formwork::Slot::FixedDry(d) => Ok(d),
                formwork::Slot::FixedWet(w) => w.resolve(),
                formwork::Slot::Choose(c) => c.resolve_exact()?.resolve(),
            })
            .collect::<Result<Vec<_>, formwork::error::FormworkError>>()?
    }
}

fn generate_cascade_collection_complete_check(field_name: &Ident) -> TokenStream {
    quote! {
        self.#field_name.iter().all(|slot| match slot {
            formwork::Slot::FixedDry(_) => true,
            formwork::Slot::FixedWet(inner) => inner.is_complete(),
            formwork::Slot::Choose(choice) => choice.is_made() && choice.selected_value().is_some_and(|inner| inner.is_complete()),
        })
    }
}
