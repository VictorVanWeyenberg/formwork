use crate::ast::{FormworkArgs, FormworkStruct};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_modes(ast: &FormworkStruct, args: &FormworkArgs) -> TokenStream {
    let name = &ast.ident;
    let mode_trait = format_ident!("{}Mode", name);
    let wet_struct = format_ident!("{}Wet", name);
    let dry_struct = format_ident!("{}Dry", name);

    let mut mode_types = Vec::new();
    let mut wet_fields = Vec::new();
    let mut dry_fields = Vec::new();

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
                } else if config.option {
                    quote! { : Default }
                } else {
                    quote! {}
                };

                mode_types.push(quote! { type #mode_type_name #bounds_tokens; });

                let wet_inner_type = field.wet_inner_type(&config);
                let dry_inner_type = field.dry_inner_type(&config);

                if config.collection {
                    wet_fields.push(quote! { type #mode_type_name = Vec<formwork::SingleSlot<#wet_inner_type, #dry_inner_type>>; });
                    dry_fields
                        .push(quote! { type #mode_type_name = Vec<#dry_inner_type>; });
                } else if config.pick {
                    wet_fields.push(quote! { type #mode_type_name = formwork::MultiSlot<#wet_inner_type, #dry_inner_type>; });
                    dry_fields
                        .push(quote! { type #mode_type_name = Vec<#dry_inner_type>; });
                } else if config.option {
                    wet_fields.push(quote! { type #mode_type_name = Option<formwork::SingleSlot<#wet_inner_type, #dry_inner_type>>; });
                    dry_fields
                        .push(quote! { type #mode_type_name = Option<#dry_inner_type>; });
                } else {
                    wet_fields.push(quote! { type #mode_type_name = formwork::SingleSlot<#wet_inner_type, #dry_inner_type>; });
                    dry_fields.push(quote! { type #mode_type_name = #dry_inner_type; });
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

    let wet_doc = format!("Wet mode marker for [`{}`].", name);
    let dry_doc = format!("Dry mode marker for [`{}`].", name);

    quote! {
        #[doc(hidden)]
        pub trait #mode_trait {
            #(#mode_types)*
        }

        #[doc = #wet_doc]
        #derives
        pub struct #wet_struct;

        #[doc = #dry_doc]
        #derives
        pub struct #dry_struct;

        #[doc(hidden)]
        impl #mode_trait for #wet_struct {
            #(#wet_fields)*
        }

        #[doc(hidden)]
        impl #mode_trait for #dry_struct {
            #(#dry_fields)*
        }
    }
}
