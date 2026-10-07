use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::SchematicStruct;

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
                if config.collection || config.pick {
                    complete_checks.push(quote! { self.#field_name.iter().all(|slot| slot.is_complete()) });

                    let resolve_expr = if config.cascade {
                        quote! {
                            self.#field_name.into_iter()
                                .map(|slot| slot.resolve())
                                .collect::<Result<Vec<_>, schematic::error::SchematicError>>()?
                                .into_iter()
                                .flatten()
                                .map(|inner| inner.resolve())
                                .collect::<Result<Vec<_>, schematic::error::SchematicError>>()?
                        }
                    } else {
                        quote! {
                            self.#field_name.into_iter()
                                .map(|slot| slot.resolve())
                                .collect::<Result<Vec<_>, schematic::error::SchematicError>>()?
                                .into_iter()
                                .flatten()
                                .collect()
                        }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });

                    from_fields.push(quote! {
                        #field_name: value.#field_name.into_iter().map(schematic::Slot::FixedResolved).collect()
                    });
                } else {
                    complete_checks.push(quote! { self.#field_name.is_complete() });

                    let resolve_expr = if config.cascade {
                        quote! { self.#field_name.resolve_exact()?.resolve()? }
                    } else {
                        quote! { self.#field_name.resolve_exact()? }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });

                    from_fields.push(quote! {
                        #field_name: schematic::Slot::FixedResolved(value.#field_name)
                    });
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
