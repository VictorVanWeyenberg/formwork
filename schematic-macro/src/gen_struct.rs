use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::{SchematicStruct, SchematicArgs};
use ident_case::RenameRule;

pub fn generate_struct(ast: &SchematicStruct, original_attrs: &[syn::Attribute], args: &SchematicArgs) -> TokenStream {
    let name = &ast.ident;
    let mode_trait = format_ident!("{}Mode", name);
    let unresolved_struct = format_ident!("{}Unresolved", name);
    let resolved_struct = format_ident!("{}Resolved", name);
    let schematic_type = format_ident!("{}Schematic", name);

    let mut struct_fields = Vec::new();
    let mut mutators = Vec::new();
    let mut unresolved_methods = Vec::new();

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            let field_name = field.ident.as_ref().unwrap();
            let camel_name = RenameRule::PascalCase.apply_to_field(&field_name.to_string());
            let mode_type_name = format_ident!("{}", camel_name);

            let config = field.slot_config().unwrap();
            let is_slot = config.is_some();
            let is_collection_slot = config.as_ref().map(|c| c.collection).unwrap_or(false);
            let is_collection_field = field.inner_type_if_vec().is_some();
            let is_collection_in_both = is_collection_slot || (is_collection_field && !is_slot);

            let builder_attr = if args.builder {
                if is_collection_in_both {
                    quote! { #[builder(default, via_mutators)] }
                } else {
                    quote! { #[builder(setter(into))] }
                }
            } else {
                quote! {}
            };

            if is_slot {
                if is_collection_in_both {
                    let fn_name = format_ident!("with_{}", field_name);
                    mutators.push(quote! {
                        pub fn #fn_name(&mut self, item: impl Into<<M as #mode_trait>::#mode_type_name>) {
                            self.#field_name.push(item.into());
                        }
                    });
                    struct_fields.push(quote! {
                        #builder_attr
                        pub #field_name: Vec<<M as #mode_trait>::#mode_type_name>,
                    });
                } else {
                    struct_fields.push(quote! {
                        #builder_attr
                        pub #field_name: <M as #mode_trait>::#mode_type_name,
                    });
                }

                let pick_fn_name = format_ident!("add_{}_pick", field_name);
                let with_pick_fn_name = format_ident!("with_{}_pick", field_name);
                
                let config = config.unwrap();
                let unresolved_inner_type = if config.cascade {
                    let base = field.base_type();
                    let unresolved_inner_mode = format_ident!("{}Unresolved", base_ident_name(base).unwrap());
                    quote! { #base<#unresolved_inner_mode> }
                } else {
                    let base = field.base_type();
                    quote! { #base }
                };

                if config.collection {
                    unresolved_methods.push(quote! {
                        pub fn #pick_fn_name(&mut self, choice: schematic::Choice<#unresolved_inner_type>) {
                            self.#field_name.push(schematic::Slot::Choose(choice));
                        }
                    });
                    if is_collection_in_both {
                        mutators.push(quote! {
                            pub fn #with_pick_fn_name(&mut self, choice: schematic::Choice<#unresolved_inner_type>) 
                            where schematic::Slot<#unresolved_inner_type>: Into<<M as #mode_trait>::#mode_type_name> 
                            {
                                self.#field_name.push(schematic::Slot::Choose(choice).into());
                            }
                        });
                    }
                } else {
                    unresolved_methods.push(quote! {
                        pub fn #pick_fn_name(&mut self, choice: schematic::Choice<#unresolved_inner_type>) {
                            self.#field_name = schematic::Slot::Choose(choice);
                        }
                    });
                    if is_collection_in_both {
                        mutators.push(quote! {
                            pub fn #with_pick_fn_name(&mut self, choice: schematic::Choice<#unresolved_inner_type>) 
                            where schematic::Slot<#unresolved_inner_type>: Into<<M as #mode_trait>::#mode_type_name> 
                            {
                                self.#field_name = schematic::Slot::Choose(choice).into();
                            }
                        });
                    }
                }
            } else {
                let ty = &field.ty;
                if is_collection_in_both {
                    let base_inner_type = field.base_type();
                    let fn_name = format_ident!("with_{}", field_name);
                    mutators.push(quote! {
                        pub fn #fn_name(&mut self, item: impl Into<#base_inner_type>) {
                            self.#field_name.push(item.into());
                        }
                    });
                }
                struct_fields.push(quote! {
                    #builder_attr
                    pub #field_name: #ty,
                });
            }
        }
    }

    let struct_attrs = original_attrs;
    let builder_derives = if args.builder {
        quote! {
            #[derive(typed_builder::TypedBuilder)]
            #[builder(mutators(
                #(#mutators)*
            ))]
        }
    } else {
        quote! {}
    };

    quote! {
        pub type #schematic_type = #name<#unresolved_struct>;

        #(#struct_attrs)*
        #builder_derives
        pub struct #name<M: #mode_trait = #resolved_struct> {
            #(#struct_fields)*
        }

        impl #name<#unresolved_struct> {
            #(#unresolved_methods)*
        }
    }
}

fn base_ident_name(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    None
}
