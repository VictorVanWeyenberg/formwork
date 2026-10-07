use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::BlueprintStruct;
use std::collections::HashSet;

pub fn generate_visitor(ast: &BlueprintStruct) -> TokenStream {
    let name = &ast.ident;
    let blueprint_type = format_ident!("{}Blueprint", name);

    let mut choice_types = HashSet::new();
    let mut nested_types = HashSet::new();

    let mut depth_0_visits = Vec::new();
    let mut depth_0_has_children = Vec::new();
    let mut depth_n_visits = Vec::new();

    for field in &ast.fields {
        let field_name = &field.ident;

        if let Some(config) = &field.config {
            let draft_inner_type = if config.is_nested {
                let base = field.base_type();
                let draft_inner_mode = format_ident!("{}Draft", crate::ast::base_ident_name(base).unwrap());
                let inner = quote! { #base<#draft_inner_mode> };
                nested_types.insert(inner.to_string());
                inner
            } else {
                let base = field.base_type();
                quote! { #base }
            };

            if config.is_vec || config.is_choice {
                choice_types.insert(draft_inner_type.to_string());
            }

            // Depth 0: check choices
            if config.is_vec {
                depth_0_visits.push(quote! {
                    for slot in &mut self.#field_name {
                        if let blueprint::Slot::Choose(choice) = slot {
                            let flow = visitor.visit_choice(choice);
                            if flow != blueprint::visitor::VisitorFlow::Continue {
                                return blueprint::Level::from_flow(flow);
                            }
                        }
                    }
                });
            } else if config.is_choice {
                depth_0_visits.push(quote! {
                    if let blueprint::Slot::Choose(choice) = &mut self.#field_name {
                        let flow = visitor.visit_choice(choice);
                        if flow != blueprint::visitor::VisitorFlow::Continue {
                            return blueprint::Level::from_flow(flow);
                        }
                    }
                });
            }

            // Has children checks and Depth N recursion
            if config.is_nested {
                if config.is_vec {
                    depth_0_has_children.push(quote! {
                        self.#field_name.iter().any(|slot| !slot.selected_values().is_empty())
                    });

                    depth_n_visits.push(quote! {
                        for slot in &mut self.#field_name {
                            for child in slot.selected_values_mut() {
                                level.combine(blueprint::Walkable::walk_level(child, visitor, depth - 1));
                                if level.flow == blueprint::visitor::VisitorFlow::Break {
                                    return level;
                                }
                            }
                        }
                    });
                } else if config.is_choice {
                    depth_0_has_children.push(quote! {
                        !self.#field_name.selected_values().is_empty()
                    });

                    depth_n_visits.push(quote! {
                        for child in self.#field_name.selected_values_mut() {
                            level.combine(blueprint::Walkable::walk_level(child, visitor, depth - 1));
                            if level.flow == blueprint::visitor::VisitorFlow::Break {
                                return level;
                            }
                        }
                    });
                } else {
                    depth_0_has_children.push(quote! {
                        !self.#field_name.selected_values().is_empty()
                    });

                    depth_n_visits.push(quote! {
                        for child in self.#field_name.selected_values_mut() {
                            level.combine(blueprint::Walkable::walk_level(child, visitor, depth - 1));
                            if level.flow == blueprint::visitor::VisitorFlow::Break {
                                return level;
                            }
                        }
                    });
                }
            }
        }
    }

    let choice_bounds: Vec<_> = choice_types.iter()
        .map(|s| {
            let ty: syn::Type = syn::parse_str(s).unwrap();
            quote! { V: blueprint::visitor::ChoiceVisitor<#ty> }
        }).collect();

    let nested_bounds: Vec<_> = nested_types.iter()
        .map(|s| {
            let ty: syn::Type = syn::parse_str(s).unwrap();
            quote! { #ty: blueprint::Walkable<V> }
        }).collect();

    let where_clause = if choice_bounds.is_empty() && nested_bounds.is_empty() {
        quote! {}
    } else {
        let all_bounds = choice_bounds.into_iter().chain(nested_bounds);
        quote! { where #(#all_bounds),* }
    };

    let has_children_expr = if depth_0_has_children.is_empty() {
        quote! { false }
    } else {
        quote! { #(#depth_0_has_children)||* }
    };

    quote! {
        impl<V> blueprint::Walkable<V> for #blueprint_type 
        #where_clause
        {
            fn walk_level(&mut self, visitor: &mut V, depth: usize) -> blueprint::Level {
                if depth == 0 {
                    #(#depth_0_visits)*
                    blueprint::Level::continue_level(#has_children_expr)
                } else {
                    let mut level = blueprint::Level::continue_level(false);
                    #(#depth_n_visits)*
                    level
                }
            }
        }
    }
}
