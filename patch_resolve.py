import re

with open("blueprint-macro/src/core.rs", "r") as f:
    content = f.read()

# I will replace the match slot_type blocks to append .resolve()? when is_nested is true.

old_vec = """                "vec" => {
                    draft_fields.push(quote! { type #mode_type_name = Vec<blueprint::Slot<#draft_inner_type>>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.iter().all(|s| s.is_complete()) });
                    resolve_fields.push(quote! { 
                        #field_name: self.#field_name.into_iter()
                            .map(|s| s.resolve())
                            .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                            .into_iter()
                            .flatten()
                            .collect()
                    });
                },"""

new_vec = """                "vec" => {
                    draft_fields.push(quote! { type #mode_type_name = Vec<blueprint::Slot<#draft_inner_type>>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.iter().all(|s| s.is_complete()) });
                    
                    let resolve_expr = if is_nested {
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
                },"""

old_choice = """                "choice" => {
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    resolve_fields.push(quote! { #field_name: self.#field_name.resolve()? });
                },"""

new_choice = """                "choice" => {
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    let resolve_expr = if is_nested {
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
                },"""

old_single = """                _ => {
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    resolve_fields.push(quote! { #field_name: self.#field_name.resolve_exact()? });
                }"""

new_single = """                _ => {
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    let resolve_expr = if is_nested {
                        quote! { self.#field_name.resolve_exact()?.resolve()? }
                    } else {
                        quote! { self.#field_name.resolve_exact()? }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });
                }"""

content = content.replace(old_vec, new_vec)
content = content.replace(old_choice, new_choice)
content = content.replace(old_single, new_single)

with open("blueprint-macro/src/core.rs", "w") as f:
    f.write(content)
