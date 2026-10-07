use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::BlueprintStruct;

pub fn generate_modes(ast: &BlueprintStruct) -> TokenStream {
    let name = &ast.ident;
    let mode_trait = format_ident!("{}Mode", name);
    let draft_struct = format_ident!("{}Draft", name);
    let final_struct = format_ident!("{}Final", name);

    let mut mode_types = Vec::new();
    let mut draft_fields = Vec::new();
    let mut final_fields = Vec::new();

    for field in &ast.fields {
        let field_name = &field.ident;
        let mode_type_name = format_ident!("{}", snake_to_camel(&field_name.to_string()));
        let field_type = &field.ty;

        mode_types.push(quote! { type #mode_type_name; });

        if let Some(config) = &field.config {
            let draft_inner_type = if config.is_nested {
                let base = field.base_type();
                let draft_inner_mode = format_ident!("{}Draft", base_ident_name(base).unwrap());
                quote! { #base<#draft_inner_mode> }
            } else {
                let base = field.base_type();
                quote! { #base }
            };

            let _is_vec_field = field.inner_type_if_vec().is_some();

            match (config.is_vec, config.is_choice) {
                (true, _) => {
                    // slot_type == "vec"
                    let base_inner_type = field.base_type();
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #base_inner_type; });
                },
                (_, true) => {
                    // slot_type == "choice"
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                },
                _ => {
                    // slot_type == "scalar"
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                }
            }
        } else {
            let is_vec_field = field.inner_type_if_vec().is_some();
            let base_inner_type = field.base_type();
            
            if is_vec_field {
                draft_fields.push(quote! { type #mode_type_name = #base_inner_type; });
                final_fields.push(quote! { type #mode_type_name = #base_inner_type; });
            } else {
                draft_fields.push(quote! { type #mode_type_name = #field_type; });
                final_fields.push(quote! { type #mode_type_name = #field_type; });
            }
        }
    }

    quote! {
        pub trait #mode_trait {
            #(#mode_types)*
        }

        pub struct #draft_struct;
        pub struct #final_struct;

        impl #mode_trait for #draft_struct {
            #(#draft_fields)*
        }

        impl #mode_trait for #final_struct {
            #(#final_fields)*
        }
    }
}

pub fn snake_to_camel(s: &str) -> String {
    let mut camel = String::new();
    let mut next_cap = true;
    for c in s.chars() {
        if c == '_' {
            next_cap = true;
        } else if next_cap {
            camel.push(c.to_ascii_uppercase());
            next_cap = false;
        } else {
            camel.push(c);
        }
    }
    camel
}

fn base_ident_name(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    None
}
