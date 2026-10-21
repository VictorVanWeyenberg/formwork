use crate::ast::SchematicStruct;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub fn generate_schematic(ast: &SchematicStruct) -> TokenStream {
    let name = &ast.ident;
    let resolved_struct = format_ident!("{}Resolved", name);
    let schematic_type = format_ident!("{}Schematic", name);

    let mut complete_checks = Vec::new();
    let mut resolve_fields = Vec::new();
    let mut from_fields = Vec::new();

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            let field_name = field.ident.as_ref().unwrap();

            if field.is_slot() {
                let config = field.slot_config().unwrap().unwrap();
                match (config.collection, config.pick, config.cascade) {
                    (true, false, true) => {
                        complete_checks
                            .push(generate_cascade_collection_complete_check(field_name));
                        resolve_fields
                            .push(generate_cascade_collection_resolve_expression(field_name));
                        from_fields.push(generate_general_collection_conversion(field_name));
                    }
                    (true, false, false) => {
                        complete_checks.push(generate_collection_complete_check(field_name));
                        resolve_fields.push(generate_collection_resolve_expression(field_name));
                        from_fields.push(generate_general_collection_conversion(field_name));
                    }
                    (false, true, true) => {
                        complete_checks.push(generate_cascade_pick_complete_check(field_name));
                        resolve_fields.push(generate_cascade_pick_resolve_expression(field_name));
                        from_fields.push(generate_general_pick_conversion(field_name));
                    }
                    (false, true, false) => {
                        complete_checks.push(generate_pick_complete_check(field_name));
                        resolve_fields.push(generate_pick_resolve_expression(field_name));
                        from_fields.push(generate_general_pick_conversion(field_name));
                    }
                    (false, false, true) => {
                        complete_checks.push(generate_cascade_slot_complete_check(field_name));
                        resolve_fields.push(generate_cascade_slot_resolve_expression(field_name));
                        from_fields.push(generate_general_slot_conversion(field_name));
                    }
                    (false, false, false) => {
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
        impl schematic::Schematic for #schematic_type {
            type Resolved = #name<#resolved_struct>;

            fn is_complete(&self) -> bool {
                #is_complete_body
            }

            fn resolve(self) -> Result<Self::Resolved, schematic::error::SchematicError> {
                Ok(#name {
                    #(#resolve_fields),*
                })
            }
        }

        impl From<#name<#resolved_struct>> for #schematic_type {
            fn from(value: #name<#resolved_struct>) -> Self {
                Self {
                    #(#from_fields),*
                }
            }
        }
    }
}

fn generate_general_slot_conversion(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: schematic::Slot::FixedResolved(value.#field_name)
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
            schematic::Slot::FixedResolved(resolved) => resolved,
            schematic::Slot::FixedUnresolved(unresolved) => unresolved.resolve()?,
            schematic::Slot::Choose(choice) => choice.resolve_exact()?.resolve()?,
        }
    }
}

fn generate_cascade_slot_complete_check(field_name: &Ident) -> TokenStream {
    quote! {
        (match &self.#field_name {
            schematic::Slot::FixedResolved(_) => true,
            schematic::Slot::FixedUnresolved(inner) => inner.is_complete(),
            schematic::Slot::Choose(choice) => choice.is_made() && choice.selected_value().is_some_and(|inner| inner.is_complete()),
        })
    }
}

fn generate_general_pick_conversion(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: schematic::Slot::FixedResolved(value.#field_name)
    }
}

fn generate_pick_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! { #field_name: self.#field_name.resolve()? }
}

fn generate_pick_complete_check(field_name: &Ident) -> TokenStream {
    quote! { self.#field_name.is_complete() }
}

fn generate_cascade_pick_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: match self.#field_name {
            schematic::Slot::FixedResolved(r) => Ok(r),
            schematic::Slot::FixedUnresolved(u) => u.into_iter().map(|item| item.resolve()).collect::<Result<Vec<_>, schematic::error::SchematicError>>(),
            schematic::Slot::Choose(c) => c.resolve()?.into_iter().map(|item| item.resolve()).collect::<Result<Vec<_>, schematic::error::SchematicError>>(),
        }?
    }
}

fn generate_cascade_pick_complete_check(field_name: &Ident) -> TokenStream {
    quote! {
        (match &self.#field_name {
            schematic::Slot::FixedResolved(_) => true,
            schematic::Slot::FixedUnresolved(inners) => inners.iter().all(|i| i.is_complete()),
            schematic::Slot::Choose(choice) => choice.is_made() && choice.selected_values().iter().all(|i| i.is_complete()),
        })
    }
}

fn generate_general_collection_conversion(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: value.#field_name.into_iter().map(schematic::Slot::FixedResolved).collect()
    }
}

fn generate_collection_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: self.#field_name.into_iter()
            .map(schematic::SingleSlot::resolve_exact)
            .collect::<Result<Vec<_>, schematic::error::SchematicError>>()?
    }
}

fn generate_collection_complete_check(field_name: &Ident) -> TokenStream {
    quote! { self.#field_name.iter().all(|slot| slot.is_complete()) }
}

fn generate_cascade_collection_resolve_expression(field_name: &Ident) -> TokenStream {
    quote! {
        #field_name: self.#field_name.into_iter()
            .map(|slot| match slot {
                schematic::Slot::FixedResolved(r) => Ok(r),
                schematic::Slot::FixedUnresolved(u) => u.resolve(),
                schematic::Slot::Choose(c) => c.resolve_exact()?.resolve(),
            })
            .collect::<Result<Vec<_>, schematic::error::SchematicError>>()?
    }
}

fn generate_cascade_collection_complete_check(field_name: &Ident) -> TokenStream {
    quote! {
        self.#field_name.iter().all(|slot| match slot {
            schematic::Slot::FixedResolved(_) => true,
            schematic::Slot::FixedUnresolved(inner) => inner.is_complete(),
            schematic::Slot::Choose(choice) => choice.is_made() && choice.selected_value().is_some_and(|inner| inner.is_complete()),
        })
    }
}
