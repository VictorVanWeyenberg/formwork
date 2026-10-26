use crate::ast::{SchematicArgs, SchematicStruct};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

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
            if let Some(config) = field.slot_config().unwrap() {
                let field_name = field.ident.as_ref().unwrap();
                let camel_name = RenameRule::PascalCase.apply_to_field(field_name.to_string());
                let mode_type_name = format_ident!("{}", camel_name);

                let bounds_tokens = if config.collection {
                    quote! {
                        : Default
                            + std::iter::IntoIterator
                            + Extend<<Self::#mode_type_name as std::iter::IntoIterator>::Item>
                    }
                } else {
                    quote! {}
                };

                mode_types.push(quote! { type #mode_type_name #bounds_tokens; });

                let unresolved_inner_type = field.unresolved_inner_type(&config);
                let resolved_inner_type = field.resolved_inner_type(&config);

                if config.collection {
                    unresolved_fields.push(quote! { type #mode_type_name = Vec<schematic::SingleSlot<#unresolved_inner_type, #resolved_inner_type>>; });
                    resolved_fields
                        .push(quote! { type #mode_type_name = Vec<#resolved_inner_type>; });
                } else if config.pick {
                    unresolved_fields.push(quote! { type #mode_type_name = schematic::MultiSlot<#unresolved_inner_type, #resolved_inner_type>; });
                    resolved_fields
                        .push(quote! { type #mode_type_name = Vec<#resolved_inner_type>; });
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
        #[doc(hidden)]
        pub trait #mode_trait {
            #(#mode_types)*
        }

        #[doc(hidden)]
        #derives
        pub struct #unresolved_struct;

        #[doc(hidden)]
        #derives
        pub struct #resolved_struct;

        #[doc(hidden)]
        impl #mode_trait for #unresolved_struct {
            #(#unresolved_fields)*
        }

        #[doc(hidden)]
        impl #mode_trait for #resolved_struct {
            #(#resolved_fields)*
        }
    }
}
