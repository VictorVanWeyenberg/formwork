import re

with open("blueprint-macro/src/core.rs", "r") as f:
    content = f.read()

# We need to insert mutators and draft_methods lists, and populate them.
content = content.replace(
    'let mut resolve_fields = Vec::new();',
    'let mut resolve_fields = Vec::new();\n    let mut mutators = Vec::new();\n    let mut draft_methods = Vec::new();'
)

# We will replace the struct_fields.push to include builder attributes
# and we will populate mutators and draft_methods inside the loop.

# We find `struct_fields.push(quote! { pub #field_name: M::#mode_type_name, });`
# and replace it.
old_struct_field = "struct_fields.push(quote! { pub #field_name: M::#mode_type_name, });"
new_struct_field = """
        let is_vec = extract_vec_inner_type(field_type).is_some();
        if is_vec || slot_type == "vec" {
            let fn_name = format_ident!("with_{}", field_name);
            mutators.push(quote! {
                pub fn #fn_name(&mut self, item: impl Into<<M as #mode_trait>::#mode_type_name>) {
                    // TypedBuilder mutators apply to the builder struct where fields are `Option<T>` or `T`.
                    // Wait, if it's via_mutators, the field in the builder is initialized to default (which is empty vec).
                    // So we can just push.
                    self.#field_name.push(item.into());
                }
            });
            struct_fields.push(quote! {
                #[cfg_attr(feature = "builders", builder(default, via_mutators))]
                pub #field_name: M::#mode_type_name,
            });
        } else {
            struct_fields.push(quote! {
                #[cfg_attr(feature = "builders", builder(setter(into)))]
                pub #field_name: M::#mode_type_name,
            });
        }

        if is_slot {
            let choice_fn_name = format_ident!("add_{}_choice", field_name);
            if slot_type == "vec" {
                draft_methods.push(quote! {
                    pub fn #choice_fn_name(&mut self, choice: blueprint::Choice<#draft_inner_type>) {
                        self.#field_name.push(blueprint::Slot::Choose(choice));
                    }
                });
            } else {
                draft_methods.push(quote! {
                    pub fn #choice_fn_name(&mut self, choice: blueprint::Choice<#draft_inner_type>) {
                        self.#field_name = blueprint::Slot::Choose(choice);
                    }
                });
            }
        }
"""
content = content.replace(old_struct_field, new_struct_field)

# Now we need to insert the TypedBuilder derive and mutators block into `expanded`, and `impl #draft_struct`
old_expanded = "pub struct #name<M: #mode_trait = #final_struct> {"
new_expanded = """
        #[cfg_attr(feature = "builders", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "builders", builder(mutators(
            #(#mutators)*
        )))]
        pub struct #name<M: #mode_trait = #final_struct> {"""
content = content.replace(old_expanded, new_expanded)

# And insert the draft methods block
old_impl_blueprint = "impl blueprint::Blueprint for #blueprint_type {"
new_impl_blueprint = """
        impl #name<#draft_struct> {
            #(#draft_methods)*
        }

        impl blueprint::Blueprint for #blueprint_type {"""
content = content.replace(old_impl_blueprint, new_impl_blueprint)

with open("blueprint-macro/src/core.rs", "w") as f:
    f.write(content)
