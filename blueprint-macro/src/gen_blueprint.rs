use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::BlueprintStruct;

pub fn generate_blueprint(ast: &BlueprintStruct) -> TokenStream {
    let name = &ast.ident;
    let final_struct = format_ident!("{}Final", name);
    let blueprint_type = format_ident!("{}Blueprint", name);

    let mut complete_checks = Vec::new();
    let mut resolve_fields = Vec::new();

    for field in &ast.fields {
        let field_name = &field.ident;

        if let Some(config) = &field.config {
            match (config.is_vec, config.is_choice) {
                (true, _) => {
                    complete_checks.push(quote! {
                        self.#field_name.iter().all(|s| s.is_complete())
                    });

                    let resolve_expr = if config.is_nested {
                        quote! {
                            self.#field_name.into_iter()
                                .map(|s| s.resolve())
                                .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                                .into_iter()
                                .flatten()
                                .map(|inner| inner.resolve())
                                .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                        }
                    } else {
                        quote! {
                            self.#field_name.into_iter()
                                .map(|s| s.resolve())
                                .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                                .into_iter()
                                .flatten()
                                .collect()
                        }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });
                },
                (_, true) => {
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    let resolve_expr = if config.is_nested {
                        quote! {
                            self.#field_name.resolve()?
                                .into_iter()
                                .map(|inner| inner.resolve())
                                .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                        }
                    } else {
                        quote! { self.#field_name.resolve()? }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });
                },
                _ => {
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    let resolve_expr = if config.is_nested {
                        quote! { self.#field_name.resolve_exact()?.resolve()? }
                    } else {
                        quote! { self.#field_name.resolve_exact()? }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });
                }
            }
        } else {
            resolve_fields.push(quote! { #field_name: self.#field_name });
        }
    }

    let is_complete_body = if complete_checks.is_empty() {
        quote! { true }
    } else {
        quote! { #(#complete_checks)&&* }
    };

    let mut from_fields = Vec::new();
    for field in &ast.fields {
        let field_name = &field.ident;
        if let Some(config) = &field.config {
            if config.is_vec {
                from_fields.push(quote! {
                    #field_name: value.#field_name.into_iter().map(|item| blueprint::Slot::Fixed(item.into())).collect()
                });
            } else if config.is_choice && field.inner_type_if_vec().is_some() {
                from_fields.push(quote! {
                    #field_name: {
                        let mut choice = blueprint::Choice::new("Lifted", value.#field_name.into_iter().map(Into::into).collect::<Vec<_>>());
                        let len = choice.options().len();
                        choice = choice.with_cardinality(blueprint::choice::Cardinality::Exact(len));
                        for i in 0..len {
                            choice.select(i).unwrap();
                        }
                        blueprint::Slot::Choose(choice)
                    }
                });
            } else {
                from_fields.push(quote! {
                    #field_name: blueprint::Slot::Fixed(value.#field_name.into())
                });
            }
        } else {
            from_fields.push(quote! {
                #field_name: value.#field_name
            });
        }
    }

    quote! {
        impl blueprint::Blueprint for #blueprint_type {
            type Final = #name<#final_struct>;

            fn is_complete(&self) -> bool {
                #is_complete_body
            }

            fn resolve(self) -> Result<Self::Final, blueprint::error::BlueprintError> {
                Ok(#name {
                    #(#resolve_fields),*
                })
            }
        }

        impl From<#name<#final_struct>> for #blueprint_type {
            fn from(value: #name<#final_struct>) -> Self {
                Self {
                    #(#from_fields),*
                }
            }
        }
    }
}
