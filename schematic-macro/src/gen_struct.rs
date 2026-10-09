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
    let mut slot_traits: Vec<proc_macro2::TokenStream> = Vec::new();

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
                let config_val = config.as_ref().unwrap();
                let is_coll = config_val.collection || config_val.pick;
                let is_cascade = config_val.cascade;

                let trait_name = format_ident!("Into{}{}Slot", name, camel_name);

                let unresolved_inner_type = if is_cascade {
                    let base = field.base_type();
                    let unresolved_inner_mode = crate::ast::unresolved_mode_type(base).unwrap();
                    quote! { #base<#unresolved_inner_mode> }
                } else {
                    let base = field.base_type();
                    quote! { #base }
                };

                let resolved_inner_type = if is_cascade {
                    let base = field.base_type();
                    let resolved_inner_mode = crate::ast::resolved_mode_type(base).unwrap();
                    quote! { #base<#resolved_inner_mode> }
                } else {
                    let base = field.base_type();
                    quote! { #base }
                };

                let slot_type = if is_coll {
                    quote! { schematic::MultiSlot<#unresolved_inner_type, #resolved_inner_type> }
                } else {
                    quote! { schematic::SingleSlot<#unresolved_inner_type, #resolved_inner_type> }
                };

                let choice_type = if is_coll {
                    quote! { schematic::choice::MultiChoice<#unresolved_inner_type> }
                } else {
                    quote! { schematic::choice::SingleChoice<#unresolved_inner_type> }
                };

                let builder_attr = if args.builder {
                    if is_coll {
                        quote! { #[builder(default, via_mutators)] }
                    } else {
                        quote! { #[builder(setter(transform = |item: impl #trait_name<M>| {
                            item.into_slot()
                        }))] }
                    }
                } else {
                    quote! {}
                };

                let trait_def = if is_coll {
                    quote! {
                        pub trait #trait_name<M: #mode_trait> {
                            fn into_slot(self) -> <<M as #mode_trait>::#mode_type_name as std::iter::IntoIterator>::Item;
                        }
                    }
                } else {
                    quote! {
                        pub trait #trait_name<M: #mode_trait> {
                            fn into_slot(self) -> <M as #mode_trait>::#mode_type_name;
                        }
                    }
                };

                let mut trait_impls = Vec::new();
                let return_ty_unresolved = if is_coll {
                    quote! { #slot_type }
                } else {
                    quote! { <#unresolved_struct as #mode_trait>::#mode_type_name }
                };

                // 1. Raw Slot object
                trait_impls.push(quote! {
                    impl #trait_name<#unresolved_struct> for #return_ty_unresolved {
                        fn into_slot(self) -> #return_ty_unresolved {
                            self
                        }
                    }
                });

                // 2. Unresolved Inner
                trait_impls.push(quote! {
                    impl #trait_name<#unresolved_struct> for #unresolved_inner_type {
                        fn into_slot(self) -> #return_ty_unresolved {
                            schematic::Slot::FixedUnresolved(self)
                        }
                    }
                });

                // 3. Resolved Inner (if cascade)
                if is_cascade {
                    trait_impls.push(quote! {
                        impl #trait_name<#unresolved_struct> for #resolved_inner_type {
                            fn into_slot(self) -> #return_ty_unresolved {
                                schematic::Slot::FixedResolved(self)
                            }
                        }
                    });
                }

                // 4. Choice
                trait_impls.push(quote! {
                    impl #trait_name<#unresolved_struct> for #choice_type {
                        fn into_slot(self) -> #return_ty_unresolved {
                            schematic::Slot::Choose(self)
                        }
                    }
                });

                // 5. String / &str ergonomics
                let base = field.base_type();
                let is_string = if let syn::Type::Path(p) = base {
                    p.path.segments.last().map(|s| s.ident == "String").unwrap_or(false)
                } else { false };

                if is_string {
                    trait_impls.push(quote! {
                        impl #trait_name<#unresolved_struct> for &str {
                            fn into_slot(self) -> #return_ty_unresolved {
                                schematic::Slot::FixedUnresolved(self.to_string())
                            }
                        }
                    });
                }

                // 6. Resolved Struct Impl
                let return_ty_resolved = quote! { #resolved_inner_type };
                trait_impls.push(quote! {
                    impl #trait_name<#resolved_struct> for #resolved_inner_type {
                        fn into_slot(self) -> #return_ty_resolved {
                            self
                        }
                    }
                });

                if is_string {
                    trait_impls.push(quote! {
                        impl #trait_name<#resolved_struct> for &str {
                            fn into_slot(self) -> #return_ty_resolved {
                                self.to_string()
                            }
                        }
                    });
                }

                slot_traits.push(quote! {
                    #trait_def
                    #(#trait_impls)*
                });

                if args.builder && is_coll {
                    let fn_name = format_ident!("with_{}", field_name);
                    mutators.push(quote! {
                        pub fn #fn_name(&mut self, item: impl #trait_name<M>) {
                            self.#field_name.extend(std::iter::once(item.into_slot()));
                        }
                        
                        pub fn #field_name<I>(&mut self, items: I) 
                        where 
                            I: std::iter::IntoIterator,
                            I::Item: #trait_name<M>
                        {
                            self.#field_name.extend(items.into_iter().map(|item| item.into_slot()));
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
        #(#slot_traits)*

        pub type #schematic_type = #name<#unresolved_struct>;

        #(#struct_attrs)*
        #builder_derives
        pub struct #name<M: #mode_trait = #resolved_struct> {
            #(#struct_fields)*
        }
    }
}
