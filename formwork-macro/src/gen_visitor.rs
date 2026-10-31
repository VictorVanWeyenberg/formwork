use crate::ast::{FormworkArgs, FormworkField, FormworkStruct, SlotConfig, TraversalStrategy};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[derive(Default)]
struct VisitorParts {
    visitor_bounds: Vec<TokenStream>,
    has_cascade: bool,
    dfs_steps: Vec<TokenStream>,
    dfs_unmakes: Vec<TokenStream>,
    bfs_steps: Vec<TokenStream>,
    bfs_unmakes: Vec<TokenStream>,
    bfs_cascade_steps: Vec<TokenStream>,
    bfs_cascade_unmakes: Vec<TokenStream>,
}

pub fn generate_visitor(ast: &FormworkStruct, args: &FormworkArgs) -> TokenStream {
    let name = &ast.ident;
    let wet_struct = format_ident!("{}Wet", name);
    let is_dfs = args.traverse == TraversalStrategy::Dfs;
    let parts = collect_visitor_parts(ast, is_dfs);

    if parts.visitor_bounds.is_empty() {
        return generate_empty_walkable(name, &wet_struct);
    }

    generate_walkable_impl(name, &wet_struct, &parts, is_dfs)
}

fn collect_visitor_parts(ast: &FormworkStruct, is_dfs: bool) -> VisitorParts {
    let mut parts = VisitorParts::default();
    if let darling::ast::Data::Struct(fields) = &ast.data {
        let has_cascade = fields
            .fields
            .iter()
            .any(|f| f.slot_config().ok().flatten().is_some_and(|c| c.cascade));
        parts.has_cascade = has_cascade;

        for field in &fields.fields {
            process_field(field, is_dfs, has_cascade, &mut parts);
        }
    }
    parts
}

fn process_field(
    field: &FormworkField,
    is_dfs: bool,
    has_cascade: bool,
    parts: &mut VisitorParts,
) {
    let Some(config) = field.slot_config().unwrap() else {
        return;
    };
    let field_name = field.ident.as_ref().unwrap();
    let wet_inner_type = field.wet_inner_type(&config);

    parts.visitor_bounds.push(quote! {
        V: formwork::ChoiceVisitor<#wet_inner_type>
    });

    if is_dfs {
        parts
            .dfs_steps
            .push(generate_local_step_dfs(field_name, &config));
        parts
            .dfs_unmakes
            .push(generate_local_unmake_dfs(field_name, &config));
    } else {
        parts
            .bfs_steps
            .push(generate_local_step_bfs(field_name, &config, has_cascade));
        parts
            .bfs_unmakes
            .push(generate_local_unmake_bfs(field_name, &config, has_cascade));
    }

    if config.cascade {
        parts.visitor_bounds.push(quote! {
            #wet_inner_type: formwork::Walkable<V>
        });

        if is_dfs {
            parts
                .dfs_steps
                .push(generate_cascade_dfs_step(field_name, &config));
            parts
                .dfs_unmakes
                .push(generate_cascade_dfs_unmake(field_name, &config));
        } else {
            parts
                .bfs_cascade_steps
                .push(generate_cascade_bfs_step(field_name, &config));
            parts
                .bfs_cascade_unmakes
                .push(generate_cascade_bfs_unmake(field_name, &config));
        }
    }
}

fn generate_local_step_dfs(field_name: &Ident, config: &SlotConfig) -> TokenStream {
    if config.collection {
        quote! {
            for slot in &mut self.#field_name {
                if let formwork::Slot::Choose(choice) = slot {
                    if *current == target {
                        return formwork::StepOutcome::Visited(visitor.visit_single(choice));
                    }
                    *current += 1;
                }
            }
        }
    } else if config.option {
        quote! {
            if let Some(formwork::Slot::Choose(choice)) = &mut self.#field_name {
                if *current == target {
                    return formwork::StepOutcome::Visited(visitor.visit_single(choice));
                }
                *current += 1;
            }
        }
    } else {
        let visit_call = if config.pick {
            quote! { visitor.visit_multi(choice) }
        } else {
            quote! { visitor.visit_single(choice) }
        };
        quote! {
            if let formwork::Slot::Choose(choice) = &mut self.#field_name {
                if *current == target {
                    return formwork::StepOutcome::Visited(#visit_call);
                }
                *current += 1;
            }
        }
    }
}

fn generate_local_unmake_dfs(field_name: &Ident, config: &SlotConfig) -> TokenStream {
    if config.collection {
        quote! {
            for slot in &mut self.#field_name {
                if let formwork::Slot::Choose(choice) = slot {
                    if *current == target {
                        let _ = choice.unmake();
                        return true;
                    }
                    *current += 1;
                }
            }
        }
    } else if config.option {
        quote! {
            if let Some(formwork::Slot::Choose(choice)) = &mut self.#field_name {
                if *current == target {
                    let _ = choice.unmake();
                    return true;
                }
                *current += 1;
            }
        }
    } else {
        quote! {
            if let formwork::Slot::Choose(choice) = &mut self.#field_name {
                if *current == target {
                    let _ = choice.unmake();
                    return true;
                }
                *current += 1;
            }
        }
    }
}

fn generate_local_step_bfs(
    field_name: &Ident,
    config: &SlotConfig,
    has_cascade: bool,
) -> TokenStream {
    if config.collection {
        quote! {
            for slot in &mut self.#field_name {
                if let formwork::Slot::Choose(choice) = slot {
                    if *current == target {
                        return (formwork::StepOutcome::Visited(visitor.visit_single(choice)), #has_cascade);
                    }
                    *current += 1;
                }
            }
        }
    } else if config.option {
        quote! {
            if let Some(formwork::Slot::Choose(choice)) = &mut self.#field_name {
                if *current == target {
                    return (formwork::StepOutcome::Visited(visitor.visit_single(choice)), #has_cascade);
                }
                *current += 1;
            }
        }
    } else {
        let visit_call = if config.pick {
            quote! { visitor.visit_multi(choice) }
        } else {
            quote! { visitor.visit_single(choice) }
        };
        quote! {
            if let formwork::Slot::Choose(choice) = &mut self.#field_name {
                if *current == target {
                    return (formwork::StepOutcome::Visited(#visit_call), #has_cascade);
                }
                *current += 1;
            }
        }
    }
}

fn generate_local_unmake_bfs(
    field_name: &Ident,
    config: &SlotConfig,
    has_cascade: bool,
) -> TokenStream {
    if config.collection {
        quote! {
            for slot in &mut self.#field_name {
                if let formwork::Slot::Choose(choice) = slot {
                    if *current == target {
                        let _ = choice.unmake();
                        return (true, #has_cascade);
                    }
                    *current += 1;
                }
            }
        }
    } else if config.option {
        quote! {
            if let Some(formwork::Slot::Choose(choice)) = &mut self.#field_name {
                if *current == target {
                    let _ = choice.unmake();
                    return (true, #has_cascade);
                }
                *current += 1;
            }
        }
    } else {
        quote! {
            if let formwork::Slot::Choose(choice) = &mut self.#field_name {
                if *current == target {
                    let _ = choice.unmake();
                    return (true, #has_cascade);
                }
                *current += 1;
            }
        }
    }
}

fn wrap_cascade_target(field_name: &Ident, config: &SlotConfig, body: TokenStream) -> TokenStream {
    if config.collection {
        quote! {
            for slot in &mut self.#field_name {
                if let formwork::Slot::FixedWet(child) = slot {
                    #body
                }
                if let Some(choice) = slot.as_choice_mut() {
                    if let Some(child) = choice.selected_value_mut() {
                        #body
                    }
                }
            }
        }
    } else if config.option {
        quote! {
            if let Some(slot) = &mut self.#field_name {
                if let formwork::Slot::FixedWet(child) = slot {
                    #body
                }
                if let Some(choice) = slot.as_choice_mut() {
                    if let Some(child) = choice.selected_value_mut() {
                        #body
                    }
                }
            }
        }
    } else if config.pick {
        quote! {
            if let formwork::Slot::FixedWet(children) = &mut self.#field_name {
                for child in children {
                    #body
                }
            }
            if let Some(choice) = self.#field_name.as_choice_mut() {
                for child in choice.selected_values_mut() {
                    #body
                }
            }
        }
    } else {
        quote! {
            if let formwork::Slot::FixedWet(child) = &mut self.#field_name {
                #body
            }
            if let Some(choice) = self.#field_name.as_choice_mut() {
                if let Some(child) = choice.selected_value_mut() {
                    #body
                }
            }
        }
    }
}

fn generate_cascade_dfs_step(field_name: &Ident, config: &SlotConfig) -> TokenStream {
    wrap_cascade_target(
        field_name,
        config,
        quote! {
            let outcome = formwork::Walkable::<V>::walk_step(child, visitor, target, current);
            if let formwork::StepOutcome::Visited(_) = outcome {
                return outcome;
            }
        },
    )
}

fn generate_cascade_dfs_unmake(field_name: &Ident, config: &SlotConfig) -> TokenStream {
    wrap_cascade_target(
        field_name,
        config,
        quote! {
            if formwork::Walkable::<V>::unmake_at_index(child, target, current) {
                return true;
            }
        },
    )
}

fn generate_cascade_bfs_step(field_name: &Ident, config: &SlotConfig) -> TokenStream {
    wrap_cascade_target(
        field_name,
        config,
        quote! {
            let (outcome, child_deeper) =
                formwork::Walkable::<V>::walk_step_level(child, visitor, depth - 1, target, current);
            if child_deeper {
                deeper = true;
            }
            if let formwork::StepOutcome::Visited(_) = outcome {
                return (outcome, true);
            }
        },
    )
}

fn generate_cascade_bfs_unmake(field_name: &Ident, config: &SlotConfig) -> TokenStream {
    wrap_cascade_target(
        field_name,
        config,
        quote! {
            let (unmade, child_deeper) =
                formwork::Walkable::<V>::unmake_step_level(child, depth - 1, target, current);
            if child_deeper {
                deeper = true;
            }
            if unmade {
                return (true, true);
            }
        },
    )
}

fn generate_empty_walkable(name: &Ident, wet_struct: &Ident) -> TokenStream {
    quote! {
        impl<V> formwork::Walkable<V> for #name<#wet_struct> {
            fn walk_step(&mut self, _visitor: &mut V, _target: usize, _current: &mut usize) -> formwork::StepOutcome {
                formwork::StepOutcome::Exhausted
            }

            fn unmake_at_index(&mut self, _target: usize, _current: &mut usize) -> bool {
                false
            }
        }
    }
}

fn generate_walkable_impl(
    name: &Ident,
    wet_struct: &Ident,
    parts: &VisitorParts,
    is_dfs: bool,
) -> TokenStream {
    let visitor_bounds = &parts.visitor_bounds;

    if is_dfs {
        let dfs_steps = &parts.dfs_steps;
        let dfs_unmakes = &parts.dfs_unmakes;
        quote! {
            impl<V> formwork::Walkable<V> for #name<#wet_struct>
            where
                #(#visitor_bounds),*
            {
                fn walk_step(&mut self, visitor: &mut V, target: usize, current: &mut usize) -> formwork::StepOutcome {
                    #(#dfs_steps)*
                    formwork::StepOutcome::Exhausted
                }

                fn unmake_at_index(&mut self, target: usize, current: &mut usize) -> bool {
                    #(#dfs_unmakes)*
                    false
                }
            }
        }
    } else {
        let bfs_steps = &parts.bfs_steps;
        let bfs_unmakes = &parts.bfs_unmakes;
        let bfs_cascade_steps = &parts.bfs_cascade_steps;
        let bfs_cascade_unmakes = &parts.bfs_cascade_unmakes;
        let has_cascade = parts.has_cascade;

        let (depth_n_step_body, depth_n_unmake_body) = if has_cascade {
            (
                quote! {
                    let mut deeper = false;
                    #(#bfs_cascade_steps)*
                    (formwork::StepOutcome::Exhausted, deeper)
                },
                quote! {
                    let mut deeper = false;
                    #(#bfs_cascade_unmakes)*
                    (false, deeper)
                },
            )
        } else {
            (
                quote! { (formwork::StepOutcome::Exhausted, false) },
                quote! { (false, false) },
            )
        };

        quote! {
            impl<V> formwork::Walkable<V> for #name<#wet_struct>
            where
                #(#visitor_bounds),*
            {
                fn walk_step(&mut self, visitor: &mut V, target: usize, current: &mut usize) -> formwork::StepOutcome {
                    let mut depth = 0;
                    loop {
                        let (outcome, deeper) =
                            formwork::Walkable::<V>::walk_step_level(self, visitor, depth, target, current);
                        if let formwork::StepOutcome::Visited(_) = outcome {
                            return outcome;
                        }
                        if !deeper {
                            return formwork::StepOutcome::Exhausted;
                        }
                        depth += 1;
                    }
                }

                fn unmake_at_index(&mut self, target: usize, current: &mut usize) -> bool {
                    let mut depth = 0;
                    loop {
                        let (unmade, deeper) =
                            formwork::Walkable::<V>::unmake_step_level(self, depth, target, current);
                        if unmade {
                            return true;
                        }
                        if !deeper {
                            return false;
                        }
                        depth += 1;
                    }
                }

                fn walk_step_level(
                    &mut self,
                    visitor: &mut V,
                    depth: usize,
                    target: usize,
                    current: &mut usize,
                ) -> (formwork::StepOutcome, bool) {
                    if depth == 0 {
                        #(#bfs_steps)*
                        (formwork::StepOutcome::Exhausted, #has_cascade)
                    } else {
                        #depth_n_step_body
                    }
                }

                fn unmake_step_level(
                    &mut self,
                    depth: usize,
                    target: usize,
                    current: &mut usize,
                ) -> (bool, bool) {
                    if depth == 0 {
                        #(#bfs_unmakes)*
                        (false, #has_cascade)
                    } else {
                        #depth_n_unmake_body
                    }
                }
            }
        }
    }
}
