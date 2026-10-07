use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::SchematicStruct;

pub fn generate_visitor(ast: &SchematicStruct) -> TokenStream {
    let name = &ast.ident;
    let unresolved_struct = format_ident!("{}Unresolved", name);

    let mut visitor_bounds = Vec::new();
    let mut walk_logic = Vec::new();

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            let field_name = field.ident.as_ref().unwrap();

            if field.is_slot() {
                let config = field.slot_config().unwrap().unwrap();
                let unresolved_inner_type = if config.cascade {
                    let base = field.base_type();
                    let unresolved_inner_mode = format_ident!("{}Unresolved", crate::ast::base_ident_name(base).unwrap());
                    quote! { #base<#unresolved_inner_mode> }
                } else {
                    let base = field.base_type();
                    quote! { #base }
                };

                visitor_bounds.push(quote! {
                    V: schematic::ChoiceVisitor<#unresolved_inner_type>
                });

                if config.cascade {
                    visitor_bounds.push(quote! {
                        #unresolved_inner_type: schematic::Walkable<V>
                    });
                }

                if config.collection {
                    walk_logic.push(quote! {
                        for slot in &mut self.#field_name {
                            if let schematic::Slot::Choose(choice) = slot {
                                let flow = visitor.visit_choice(choice);
                                if flow != schematic::VisitorFlow::Continue {
                                    return schematic::Level::from_flow(flow);
                                }
                            }
                        }
                    });

                    if config.cascade {
                        walk_logic.push(quote! {
                            for slot in &mut self.#field_name {
                                if let Some(child) = slot.as_fixed_mut() {
                                    let flow = schematic::Schematic::walk_choices(child, visitor);
                                    if flow == schematic::VisitorFlow::Break {
                                        return schematic::Level::from_flow(flow);
                                    }
                                }
                                if let Some(choice) = slot.as_choice_mut() {
                                    for child in choice.selected_values_mut() {
                                        let flow = schematic::Schematic::walk_choices(child, visitor);
                                        if flow == schematic::VisitorFlow::Break {
                                            return schematic::Level::from_flow(flow);
                                        }
                                    }
                                }
                            }
                        });
                    }
                } else {
                    walk_logic.push(quote! {
                        if let schematic::Slot::Choose(choice) = &mut self.#field_name {
                            let flow = visitor.visit_choice(choice);
                            if flow != schematic::VisitorFlow::Continue {
                                return schematic::Level::from_flow(flow);
                            }
                        }
                    });

                    if config.cascade {
                        walk_logic.push(quote! {
                            if let Some(child) = self.#field_name.as_fixed_mut() {
                                let flow = schematic::Schematic::walk_choices(child, visitor);
                                if flow == schematic::VisitorFlow::Break {
                                    return schematic::Level::from_flow(flow);
                                }
                            }
                            if let Some(choice) = self.#field_name.as_choice_mut() {
                                for child in choice.selected_values_mut() {
                                    let flow = schematic::Schematic::walk_choices(child, visitor);
                                    if flow == schematic::VisitorFlow::Break {
                                        return schematic::Level::from_flow(flow);
                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }

    if visitor_bounds.is_empty() {
        return quote! {
            impl<V> schematic::Walkable<V> for #name<#unresolved_struct> {
                fn walk_level(&mut self, _visitor: &mut V, _depth: usize) -> schematic::Level {
                    schematic::Level::continue_level(false)
                }
            }
        };
    }

    quote! {
        impl<V> schematic::Walkable<V> for #name<#unresolved_struct>
        where
            #(#visitor_bounds),*
        {
            fn walk_level(&mut self, visitor: &mut V, depth: usize) -> schematic::Level {
                if depth == 0 {
                    #(#walk_logic)*
                    schematic::Level::continue_level(false)
                } else {
                    schematic::Level::continue_level(false)
                }
            }
        }
    }
}
