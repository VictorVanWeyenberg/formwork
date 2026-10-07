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

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            let field_name = field.ident.as_ref().unwrap();
            let camel_name = RenameRule::PascalCase.apply_to_field(&field_name.to_string());
            let mode_type_name = format_ident!("{}", camel_name);
            let vis = &field.vis;

            let config = field.slot_config().unwrap();
            let is_slot = config.is_some();
            let is_collection_field = field.inner_type_if_vec().is_some();

            if is_slot {
                let is_coll = config.as_ref().unwrap().collection || config.as_ref().unwrap().pick;
                let builder_attr = if args.builder {
                    if is_coll {
                        quote! { #[builder(default, via_mutators)] }
                    } else {
                        quote! { #[builder(setter(into))] }
                    }
                } else {
                    quote! {}
                };

                if args.builder && is_coll {
                    let fn_name = format_ident!("with_{}", field_name);
                    mutators.push(quote! {
                        pub fn #fn_name(&mut self, item: impl Into<<<M as #mode_trait>::#mode_type_name as std::iter::IntoIterator>::Item>) {
                            self.#field_name.extend(std::iter::once(item.into()));
                        }
                    });
                }

                struct_fields.push(quote! {
                    #builder_attr
                    #vis #field_name: <M as #mode_trait>::#mode_type_name,
                });
            } else {
                let ty = &field.ty;
                if is_collection_field {
                    let base_inner_type = field.base_type();
                    let fn_name = format_ident!("with_{}", field_name);
                    
                    let builder_attr = if args.builder {
                        quote! { #[builder(default, via_mutators)] }
                    } else {
                        quote! {}
                    };

                    if args.builder {
                        mutators.push(quote! {
                            pub fn #fn_name(&mut self, item: impl Into<#base_inner_type>) {
                                self.#field_name.push(item.into());
                            }
                        });
                    }
                    
                    struct_fields.push(quote! {
                        #builder_attr
                        #vis #field_name: #ty,
                    });
                } else {
                    let builder_attr = if args.builder {
                        quote! { #[builder(setter(into))] }
                    } else {
                        quote! {}
                    };

                    struct_fields.push(quote! {
                        #builder_attr
                        #vis #field_name: #ty,
                    });
                }
            }
        }
    }

    let struct_attrs = original_attrs;
    let builder_derives = if args.builder {
        if mutators.is_empty() {
            quote! {
                #[derive(typed_builder::TypedBuilder)]
            }
        } else {
            quote! {
                #[derive(typed_builder::TypedBuilder)]
                #[builder(mutators(
                    #(#mutators)*
                ))]
            }
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
    }
}
