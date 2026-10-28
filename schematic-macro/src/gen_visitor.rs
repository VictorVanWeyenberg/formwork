use crate::ast::{SchematicArgs, SchematicField, SchematicStruct, SlotConfig, TraversalStrategy};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[derive(Default)]
struct VisitorParts {
    visitor_bounds: Vec<TokenStream>,
    local_logic: Vec<TokenStream>,
    cascade_logic: Vec<TokenStream>,
    has_cascade: bool,
}

pub fn generate_visitor(ast: &SchematicStruct, args: &SchematicArgs) -> TokenStream {
    let name = &ast.ident;
    let unresolved_struct = format_ident!("{}Unresolved", name);
    let is_dfs = args.traverse == TraversalStrategy::Dfs;
    let parts = collect_visitor_parts(ast, is_dfs);

    if parts.visitor_bounds.is_empty() {
        return generate_empty_walkable(name, &unresolved_struct);
    }

    generate_walkable_impl(name, &unresolved_struct, &parts, is_dfs)
}

fn collect_visitor_parts(ast: &SchematicStruct, is_dfs: bool) -> VisitorParts {
    let mut parts = VisitorParts::default();
    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            process_field(field, is_dfs, &mut parts);
        }
    }
    parts
}

fn process_field(field: &SchematicField, is_dfs: bool, parts: &mut VisitorParts) {
    let Some(config) = field.slot_config().unwrap() else {
        return;
    };
    let field_name = field.ident.as_ref().unwrap();
    let unresolved_inner_type = field.unresolved_inner_type(&config);

    parts.visitor_bounds.push(quote! {
        V: schematic::ChoiceVisitor<#unresolved_inner_type>
    });

    parts.local_logic.push(generate_local_visit(field_name, &config));

    if config.cascade {
        parts.visitor_bounds.push(quote! {
            #unresolved_inner_type: schematic::Walkable<V>
        });
        parts.has_cascade = true;

        let cascade = generate_cascade(field_name, &config, is_dfs);
        if is_dfs {
            parts.local_logic.push(cascade);
        } else {
            parts.cascade_logic.push(cascade);
        }
    }
}

fn generate_local_visit(field_name: &Ident, config: &SlotConfig) -> TokenStream {
    if config.collection {
        quote! {
            for slot in &mut self.#field_name {
                if let schematic::Slot::Choose(choice) = slot {
                    let flow = visitor.visit_single(choice);
                    if flow != schematic::VisitorFlow::Continue {
                        return schematic::Level::from_flow(flow);
                    }
                }
            }
        }
    } else {
        let visit_call = if config.pick {
            quote! { visitor.visit_multi(choice) }
        } else {
            quote! { visitor.visit_single(choice) }
        };
        quote! {
            if let schematic::Slot::Choose(choice) = &mut self.#field_name {
                let flow = #visit_call;
                if flow != schematic::VisitorFlow::Continue {
                    return schematic::Level::from_flow(flow);
                }
            }
        }
    }
}

fn child_cascade_step(is_dfs: bool, child: &TokenStream) -> TokenStream {
    if is_dfs {
        quote! {
            let flow = schematic::Schematic::walk_choices(#child, visitor);
            if flow == schematic::VisitorFlow::Break {
                return schematic::Level::from_flow(flow);
            }
        }
    } else {
        quote! {
            level.combine(#child.walk_level(visitor, depth - 1));
            if level.flow == schematic::VisitorFlow::Break {
                return level;
            }
        }
    }
}

fn generate_cascade(field_name: &Ident, config: &SlotConfig, is_dfs: bool) -> TokenStream {
    if config.collection {
        generate_collection_cascade(field_name, is_dfs)
    } else if config.pick {
        generate_pick_cascade(field_name, is_dfs)
    } else {
        generate_single_cascade(field_name, is_dfs)
    }
}

fn generate_single_cascade(field_name: &Ident, is_dfs: bool) -> TokenStream {
    let step = child_cascade_step(is_dfs, &quote!(child));
    quote! {
        if let schematic::Slot::FixedUnresolved(ref mut child) = self.#field_name {
            #step
        }
        if let Some(choice) = self.#field_name.as_choice_mut() {
            if let Some(child) = choice.selected_value_mut() {
                #step
            }
        }
    }
}

fn generate_collection_cascade(field_name: &Ident, is_dfs: bool) -> TokenStream {
    let step = child_cascade_step(is_dfs, &quote!(child));
    quote! {
        for slot in &mut self.#field_name {
            if let schematic::Slot::FixedUnresolved(child) = slot {
                #step
            }
            if let Some(choice) = slot.as_choice_mut() {
                if let Some(child) = choice.selected_value_mut() {
                    #step
                }
            }
        }
    }
}

fn generate_pick_cascade(field_name: &Ident, is_dfs: bool) -> TokenStream {
    let step = child_cascade_step(is_dfs, &quote!(child));
    quote! {
        if let schematic::Slot::FixedUnresolved(children) = &mut self.#field_name {
            for child in children {
                #step
            }
        }
        if let Some(choice) = self.#field_name.as_choice_mut() {
            for child in choice.selected_values_mut() {
                #step
            }
        }
    }
}

fn generate_empty_walkable(name: &Ident, unresolved_struct: &Ident) -> TokenStream {
    quote! {
        impl<V> schematic::Walkable<V> for #name<#unresolved_struct> {
            fn walk_level(&mut self, _visitor: &mut V, _depth: usize) -> schematic::Level {
                schematic::Level::continue_level(false)
            }
        }
    }
}

fn strategy_walk_branches(
    is_dfs: bool,
    has_cascade: bool,
    cascade_logic: &[TokenStream],
) -> (TokenStream, TokenStream) {
    if is_dfs {
        (
            quote! { false },
            quote! { schematic::Level::continue_level(false) },
        )
    } else {
        (
            quote! { #has_cascade },
            quote! {
                let mut level = schematic::Level::continue_level(false);
                #(#cascade_logic)*
                level
            },
        )
    }
}

fn generate_walkable_impl(
    name: &Ident,
    unresolved_struct: &Ident,
    parts: &VisitorParts,
    is_dfs: bool,
) -> TokenStream {
    let visitor_bounds = &parts.visitor_bounds;
    let local_logic = &parts.local_logic;
    let (depth_zero_continuation, depth_n_body) =
        strategy_walk_branches(is_dfs, parts.has_cascade, &parts.cascade_logic);

    quote! {
        impl<V> schematic::Walkable<V> for #name<#unresolved_struct>
        where
            #(#visitor_bounds),*
        {
            fn walk_level(&mut self, visitor: &mut V, depth: usize) -> schematic::Level {
                if depth == 0 {
                    #(#local_logic)*
                    schematic::Level::continue_level(#depth_zero_continuation)
                } else {
                    #depth_n_body
                }
            }
        }
    }
}
