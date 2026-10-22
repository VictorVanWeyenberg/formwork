use crate::ast::SchematicStruct;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_visitor(ast: &SchematicStruct, args: &crate::ast::SchematicArgs) -> TokenStream {
    let name = &ast.ident;
    let unresolved_struct = format_ident!("{}Unresolved", name);

    let mut visitor_bounds = Vec::new();
    let mut local_logic = Vec::new();
    let mut cascade_logic = Vec::new();
    let mut has_cascade = false;

    let is_dfs = args.traverse == crate::ast::TraversalStrategy::Dfs;

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            let field_name = field.ident.as_ref().unwrap();

            if let Some(config) = field.slot_config().unwrap() {
                let unresolved_inner_type = field.unresolved_inner_type(config.cascade);

                visitor_bounds.push(quote! {
                    V: schematic::ChoiceVisitor<#unresolved_inner_type>
                });

                if config.cascade {
                    visitor_bounds.push(quote! {
                        #unresolved_inner_type: schematic::Walkable<V>
                    });
                    has_cascade = true;
                }

                if config.collection {
                    local_logic.push(quote! {
                        for slot in &mut self.#field_name {
                            if let schematic::Slot::Choose(choice) = slot {
                                let flow = visitor.visit_single(choice);
                                if flow != schematic::VisitorFlow::Continue {
                                    return schematic::Level::from_flow(flow);
                                }
                            }
                        }
                    });

                    if config.cascade {
                        if is_dfs {
                            local_logic.push(quote! {
                                for slot in &mut self.#field_name {
                                    if let schematic::Slot::FixedUnresolved(child) = slot {
                                        let flow = schematic::Schematic::walk_choices(child, visitor);
                                        if flow == schematic::VisitorFlow::Break {
                                            return schematic::Level::from_flow(flow);
                                        }
                                    }
                                    if let Some(choice) = slot.as_choice_mut() {
                                        if let Some(child) = choice.selected_value_mut() {
                                            let flow = schematic::Schematic::walk_choices(child, visitor);
                                            if flow == schematic::VisitorFlow::Break {
                                                return schematic::Level::from_flow(flow);
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            cascade_logic.push(quote! {
                                for slot in &mut self.#field_name {
                                    if let schematic::Slot::FixedUnresolved(child) = slot {
                                        level.combine(child.walk_level(visitor, depth - 1));
                                        if level.flow == schematic::VisitorFlow::Break {
                                            return level;
                                        }
                                    }
                                    if let Some(choice) = slot.as_choice_mut() {
                                        if let Some(child) = choice.selected_value_mut() {
                                            level.combine(child.walk_level(visitor, depth - 1));
                                            if level.flow == schematic::VisitorFlow::Break {
                                                return level;
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                } else if config.pick {
                    local_logic.push(quote! {
                        if let schematic::Slot::Choose(choice) = &mut self.#field_name {
                            let flow = visitor.visit_multi(choice);
                            if flow != schematic::VisitorFlow::Continue {
                                return schematic::Level::from_flow(flow);
                            }
                        }
                    });

                    if config.cascade {
                        if is_dfs {
                            local_logic.push(quote! {
                                if let schematic::Slot::FixedUnresolved(children) = &mut self.#field_name {
                                    for child in children {
                                        let flow = schematic::Schematic::walk_choices(child, visitor);
                                        if flow == schematic::VisitorFlow::Break {
                                            return schematic::Level::from_flow(flow);
                                        }
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
                        } else {
                            cascade_logic.push(quote! {
                                if let schematic::Slot::FixedUnresolved(children) = &mut self.#field_name {
                                    for child in children {
                                        level.combine(child.walk_level(visitor, depth - 1));
                                        if level.flow == schematic::VisitorFlow::Break {
                                            return level;
                                        }
                                    }
                                }
                                if let Some(choice) = self.#field_name.as_choice_mut() {
                                    for child in choice.selected_values_mut() {
                                        level.combine(child.walk_level(visitor, depth - 1));
                                        if level.flow == schematic::VisitorFlow::Break {
                                            return level;
                                        }
                                    }
                                }
                            });
                        }
                    }
                } else {
                    local_logic.push(quote! {
                        if let schematic::Slot::Choose(choice) = &mut self.#field_name {
                            let flow = visitor.visit_single(choice);
                            if flow != schematic::VisitorFlow::Continue {
                                return schematic::Level::from_flow(flow);
                            }
                        }
                    });

                    if config.cascade {
                        if is_dfs {
                            local_logic.push(quote! {
                                if let schematic::Slot::FixedUnresolved(ref mut child) = self.#field_name {
                                    let flow = schematic::Schematic::walk_choices(child, visitor);
                                    if flow == schematic::VisitorFlow::Break {
                                        return schematic::Level::from_flow(flow);
                                    }
                                }
                                if let Some(choice) = self.#field_name.as_choice_mut() {
                                    if let Some(child) = choice.selected_value_mut() {
                                        let flow = schematic::Schematic::walk_choices(child, visitor);
                                        if flow == schematic::VisitorFlow::Break {
                                            return schematic::Level::from_flow(flow);
                                        }
                                    }
                                }
                            });
                        } else {
                            cascade_logic.push(quote! {
                                if let schematic::Slot::FixedUnresolved(ref mut child) = self.#field_name {
                                    level.combine(child.walk_level(visitor, depth - 1));
                                    if level.flow == schematic::VisitorFlow::Break {
                                        return level;
                                    }
                                }
                                if let Some(choice) = self.#field_name.as_choice_mut() {
                                    if let Some(child) = choice.selected_value_mut() {
                                        level.combine(child.walk_level(visitor, depth - 1));
                                        if level.flow == schematic::VisitorFlow::Break {
                                            return level;
                                        }
                                    }
                                }
                            });
                        }
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

    if is_dfs {
        quote! {
            impl<V> schematic::Walkable<V> for #name<#unresolved_struct>
            where
                #(#visitor_bounds),*
            {
                fn walk_level(&mut self, visitor: &mut V, depth: usize) -> schematic::Level {
                    if depth == 0 {
                        #(#local_logic)*
                        schematic::Level::continue_level(false)
                    } else {
                        schematic::Level::continue_level(false)
                    }
                }
            }
        }
    } else {
        quote! {
            impl<V> schematic::Walkable<V> for #name<#unresolved_struct>
            where
                #(#visitor_bounds),*
            {
                fn walk_level(&mut self, visitor: &mut V, depth: usize) -> schematic::Level {
                    if depth == 0 {
                        #(#local_logic)*
                        schematic::Level::continue_level(#has_cascade)
                    } else {
                        let mut level = schematic::Level::continue_level(false);
                        #(#cascade_logic)*
                        level
                    }
                }
            }
        }
    }
}
