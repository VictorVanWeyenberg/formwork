use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::{SchematicStruct, SchematicArgs};
use ident_case::RenameRule;

pub fn generate_modes(ast: &SchematicStruct, args: &SchematicArgs) -> TokenStream {
    let name = &ast.ident;
    let mode_trait = format_ident!("{}Mode", name);
    let unresolved_struct = format_ident!("{}Unresolved", name);
    let resolved_struct = format_ident!("{}Resolved", name);

    let mut mode_types = Vec::new();
    let mut unresolved_fields = Vec::new();
    let mut resolved_fields = Vec::new();

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            if field.is_slot() {
                let field_name = field.ident.as_ref().unwrap();
                let camel_name = RenameRule::PascalCase.apply_to_field(&field_name.to_string());
                let mode_type_name = format_ident!("{}", camel_name);

                let mut mode_bounds = Vec::new();
                for path in &*args.derives {
                    if let Some(segment) = path.segments.last() {
                        if segment.ident == "Deserialize" {
                            mode_bounds.push(quote! { for<'schematic_a> #path<'schematic_a> });
                            continue;
                        }
                    }
                    mode_bounds.push(quote! { #path });
                }
                
                let config = field.slot_config().unwrap().unwrap();

                if config.collection || config.pick {
                    mode_bounds.push(quote! { Default });
                    mode_bounds.push(quote! { std::iter::IntoIterator });
                    mode_bounds.push(quote! { Extend<<Self::#mode_type_name as std::iter::IntoIterator>::Item> });
                }

                let bounds_tokens = if mode_bounds.is_empty() {
                    quote! {}
                } else {
                    quote! { : #(#mode_bounds)+* }
                };

                mode_types.push(quote! { type #mode_type_name #bounds_tokens; });

                let unresolved_inner_type = if config.cascade {
                    let base = field.base_type();
                    let unresolved_inner_mode = crate::ast::unresolved_mode_type(base).unwrap();
                    quote! { #base<#unresolved_inner_mode> }
                } else {
                    let base = field.base_type();
                    quote! { #base }
                };

                let resolved_inner_type = if config.cascade {
                    let base = field.base_type();
                    let resolved_inner_mode = crate::ast::resolved_mode_type(base).unwrap();
                    quote! { #base<#resolved_inner_mode> }
                } else {
                    let base = field.base_type();
                    quote! { #base }
                };

                if config.collection || config.pick {
                    unresolved_fields.push(quote! { type #mode_type_name = Vec<schematic::MultiSlot<#unresolved_inner_type, #resolved_inner_type>>; });
                    resolved_fields.push(quote! { type #mode_type_name = Vec<#resolved_inner_type>; });
                } else {
                    unresolved_fields.push(quote! { type #mode_type_name = schematic::SingleSlot<#unresolved_inner_type, #resolved_inner_type>; });
                    resolved_fields.push(quote! { type #mode_type_name = #resolved_inner_type; });
                }
            }
        }
    }

    let derives = if args.derives.is_empty() {
        quote! {}
    } else {
        let paths = &args.derives;
        quote! { #[derive(#(#paths),*)] }
    };

    quote! {
        pub trait #mode_trait {
            #(#mode_types)*
        }

        #derives
        pub struct #unresolved_struct;
        
        #derives
        pub struct #resolved_struct;

        impl #mode_trait for #unresolved_struct {
            #(#unresolved_fields)*
        }

        impl #mode_trait for #resolved_struct {
            #(#resolved_fields)*
        }
    }
}
