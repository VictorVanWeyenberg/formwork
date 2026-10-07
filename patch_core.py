import re

with open("blueprint-macro/src/core.rs", "r") as f:
    content = f.read()

# We need to extract the logic that parses nested
# and replace the draft type generation.

content = content.replace(
    'let mut slot_type = "single";',
    'let mut slot_type = "single";\n        let mut is_nested = false;'
)

content = content.replace(
    'if tokens_str.contains("choice") {',
    'if tokens_str.contains("choice") {\n                        slot_type = "choice";\n                    }\n                    if tokens_str.contains("nested") {\n                        is_nested = true;\n                    }'
)

replacement = """
        if is_slot {
            let base_inner_type = extract_vec_inner_type(field_type).unwrap_or(field_type);
            
            let draft_inner_type = if is_nested {
                // If nested, we append 'Blueprint' to the type ident
                // We assume base_inner_type is a simple Path
                let mut path = match base_inner_type {
                    syn::Type::Path(p) => p.clone(),
                    _ => panic!("Nested blueprint type must be a path"),
                };
                let last_segment = path.path.segments.last_mut().unwrap();
                last_segment.ident = format_ident!("{}Blueprint", last_segment.ident);
                quote! { #path }
            } else {
                quote! { #base_inner_type }
            };

            match slot_type {
                "vec" => {
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
                },
                "choice" => {
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    resolve_fields.push(quote! { #field_name: self.#field_name.resolve()? });
                },
                _ => {
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    resolve_fields.push(quote! { #field_name: self.#field_name.resolve_exact()? });
                }
            }
        } else {
"""

# I need to match everything from "if is_slot {" to "        } else {"
start_idx = content.find("if is_slot {")
end_idx = content.find("        } else {")

new_content = content[:start_idx] + replacement.strip() + "\n        } else {" + content[end_idx + 16:]

with open("blueprint-macro/src/core.rs", "w") as f:
    f.write(new_content)
