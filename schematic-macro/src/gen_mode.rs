use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::SchematicStruct;

pub fn generate_modes(ast: &SchematicStruct) -> TokenStream {
    let name = &ast.ident;
    let mode_trait = format_ident!("{}Mode", name);
    let unresolved_struct = format_ident!("{}Unresolved", name);
    let resolved_struct = format_ident!("{}Resolved", name);

    let mut mode_types = Vec::new();
    let mut unresolved_fields = Vec::new();
    let mut resolved_fields = Vec::new();

    for field in &ast.fields {
        let field_name = &field.ident;
        let mode_type_name = format_ident!("{}", snake_to_camel(&field_name.to_string()));
        let field_type = &field.ty;

        mode_types.push(quote! { type #mode_type_name; });

        if let Some(config) = &field.config {
            let unresolved_inner_type = if config.is_cascade {
                let base = field.base_type();
                let unresolved_inner_mode = format_ident!("{}Unresolved", base_ident_name(base).unwrap());
                quote! { #base<#unresolved_inner_mode> }
            } else {
                let base = field.base_type();
                quote! { #base }
            };

            let _is_collection_field = field.inner_type_if_vec().is_some();

            match (config.is_collection, config.is_pick) {
                (true, _) => {
                    // slot_type == "vec"
                    let base_inner_type = field.base_type();
                    unresolved_fields.push(quote! { type #mode_type_name = schematic::Slot<#unresolved_inner_type>; });
                    resolved_fields.push(quote! { type #mode_type_name = #base_inner_type; });
                },
                (_, true) => {
                    // slot_type == "choice"
                    unresolved_fields.push(quote! { type #mode_type_name = schematic::Slot<#unresolved_inner_type>; });
                    resolved_fields.push(quote! { type #mode_type_name = #field_type; });
                },
                _ => {
                    // slot_type == "scalar"
                    unresolved_fields.push(quote! { type #mode_type_name = schematic::Slot<#unresolved_inner_type>; });
                    resolved_fields.push(quote! { type #mode_type_name = #field_type; });
                }
            }
        } else {
            let is_collection_field = field.inner_type_if_vec().is_some();
            let base_inner_type = field.base_type();
            
            if is_collection_field {
                unresolved_fields.push(quote! { type #mode_type_name = #base_inner_type; });
                resolved_fields.push(quote! { type #mode_type_name = #base_inner_type; });
            } else {
                unresolved_fields.push(quote! { type #mode_type_name = #field_type; });
                resolved_fields.push(quote! { type #mode_type_name = #field_type; });
            }
        }
    }

    quote! {
        pub trait #mode_trait {
            #(#mode_types)*
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub struct #unresolved_struct;
        #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub struct #resolved_struct;

        impl #mode_trait for #unresolved_struct {
            #(#unresolved_fields)*
        }

        impl #mode_trait for #resolved_struct {
            #(#resolved_fields)*
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
