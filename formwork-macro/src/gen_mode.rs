use crate::ast::{FormworkArgs, FormworkStruct};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_modes(ast: &FormworkStruct, args: &FormworkArgs) -> TokenStream {
    let name = &ast.ident;
    let mode_trait = format_ident!("{}Mode", name);
    let wet_struct = format_ident!("{}Wet", name);
    let dry_struct = format_ident!("{}Dry", name);
    let formwork_type = format_ident!("{}Formwork", name);

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

                let slot_kind_desc = if config.pick {
                    "multi-item selection slot with cardinality constraints"
                } else if config.collection {
                    "ordered collection of slot items"
                } else if config.option {
                    "optional slot item"
                } else {
                    "single slot item"
                };

                let assoc_doc = format!(
                    "Representation type for the `{field_name}` slot field in mode `Self` ({slot_kind_desc}).\n\n\
                     - In wet mode ([`{wet_struct}`]): holds reactive choice-bearing slot storage.\n\
                     - In dry mode ([`{dry_struct}`]): holds the cured, watertight domain type.",
                    field_name = field_name,
                    slot_kind_desc = slot_kind_desc,
                    wet_struct = wet_struct,
                    dry_struct = dry_struct,
                );

                mode_types.push(quote! {
                    #[doc = #assoc_doc]
                    type #mode_type_name #bounds_tokens;
                });

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

    let mode_trait_doc = format!(
        "Mode marker trait defining the Higher-Kinded Data (HKD) slot representations for [`{name}`].\n\n\
         This trait governs how slot fields are represented between the fluid wet formwork mold \
         ([`{wet_struct}`] / [`{formwork_type}`]) and the cured watertight domain model \
         ([`{dry_struct}`] / [`{name}`]).",
        name = name,
        wet_struct = wet_struct,
        dry_struct = dry_struct,
        formwork_type = formwork_type,
    );

    let wet_doc = format!(
        "Wet mode marker for [`{name}`].\n\n\
         Configures [`{name}<{wet_struct}>`] (aliased as [`{formwork_type}`]) where slot fields \
         are wrapped in reactive formwork choice slots.\n\n\
         Implements [`{mode_trait}`].",
        name = name,
        wet_struct = wet_struct,
        formwork_type = formwork_type,
        mode_trait = mode_trait,
    );

    let dry_doc = format!(
        "Dry mode marker for [`{name}`].\n\n\
         Configures [`{name}<{dry_struct}>`] (aliased as [`{name}`]) where slot fields \
         hold resolved concrete domain values.\n\n\
         Implements [`{mode_trait}`].",
        name = name,
        dry_struct = dry_struct,
        mode_trait = mode_trait,
    );

    let wet_impl_doc = format!(
        "Configures slot fields of [`{name}`] as reactive, choice-bearing wet slots ([`{formwork_type}`]).",
        name = name,
        formwork_type = formwork_type,
    );

    let dry_impl_doc = format!(
        "Configures slot fields of [`{name}`] as cured, watertight domain types.",
        name = name,
    );

    quote! {
        #[doc = #mode_trait_doc]
        pub trait #mode_trait {
            #(#mode_types)*
        }

        #[doc = #wet_doc]
        #derives
        pub struct #wet_struct;

        #[doc = #dry_doc]
        #derives
        pub struct #dry_struct;

        #[doc = #wet_impl_doc]
        impl #mode_trait for #wet_struct {
            #(#wet_fields)*
        }

        #[doc = #dry_impl_doc]
        impl #mode_trait for #dry_struct {
            #(#dry_fields)*
        }
    }
}
