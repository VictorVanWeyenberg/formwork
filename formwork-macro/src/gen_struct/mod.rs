mod builder;
mod context;
mod docs;
mod traits;

pub(crate) use context::*;

use crate::ast::{FormworkArgs, FormworkField, FormworkStruct, SlotConfig};
use builder::{
    format_field, generate_builder_derives, generate_regular_field, generate_slot_builder_attr,
    generate_slot_collection_mutators,
};
use docs::{generate_formwork_alias_doc, generate_hkd_struct_doc};
use proc_macro2::TokenStream;
use quote::quote;
use traits::generate_slot_trait;

pub(crate) struct GeneratedField {
    pub(crate) field_tokens: TokenStream,
    pub(crate) slot_trait: Option<TokenStream>,
    pub(crate) mutator: Option<TokenStream>,
}

pub fn generate_struct(
    ast: &FormworkStruct,
    original_attrs: &[syn::Attribute],
    args: &FormworkArgs,
) -> TokenStream {
    let idents = StructIdents::new(&ast.ident);
    let mut struct_fields = Vec::new();
    let mut mutators = Vec::new();
    let mut slot_traits = Vec::new();

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            let generated = generate_field(field, &idents, args);
            struct_fields.push(generated.field_tokens);
            if let Some(slot_trait) = generated.slot_trait {
                slot_traits.push(slot_trait);
            }
            if let Some(mutator) = generated.mutator {
                mutators.push(mutator);
            }
        }
    }

    let builder_derives = generate_builder_derives(args, &idents.mode_trait, &mutators);
    let formwork_doc = generate_formwork_alias_doc(idents.name);
    let hkd_doc = generate_hkd_struct_doc(idents.name, &idents.formwork_type);

    let name = idents.name;
    let mode_trait = &idents.mode_trait;
    let wet_struct = &idents.wet_struct;
    let dry_struct = &idents.dry_struct;
    let formwork_type = &idents.formwork_type;

    quote! {
        #(#slot_traits)*

        #[doc = #formwork_doc]
        pub type #formwork_type = #name<#wet_struct>;

        #(#original_attrs)*
        #[doc = #hkd_doc]
        #builder_derives
        pub struct #name<M: #mode_trait = #dry_struct> {
            #(#struct_fields)*
        }
    }
}

fn generate_field(
    field: &FormworkField,
    idents: &StructIdents,
    args: &FormworkArgs,
) -> GeneratedField {
    if let Some(config) = field.slot_config().unwrap() {
        generate_slot_field(field, &config, idents, args)
    } else {
        generate_regular_field(field, args)
    }
}

fn generate_slot_field(
    field: &FormworkField,
    config: &SlotConfig,
    idents: &StructIdents,
    args: &FormworkArgs,
) -> GeneratedField {
    let ctx = SlotFieldContext::new(field, config, idents);
    let slot_trait = generate_slot_trait(&ctx);
    let builder_attr = generate_slot_builder_attr(&ctx, args.builder);
    let mutator = if args.builder && config.collection {
        Some(generate_slot_collection_mutators(&ctx))
    } else {
        None
    };

    let mode_type_name = &ctx.mode_type_name;
    let (ty, serde_bound_attr) = if config.option {
        let has_serialize = args.derives.iter().any(|p| {
            p.segments.last().is_some_and(|s| s.ident == "Serialize")
        });
        let has_deserialize = args.derives.iter().any(|p| {
            p.segments.last().is_some_and(|s| s.ident == "Deserialize")
        });
        let user_has_serde_bound = ctx.field_attrs.iter().any(|attr| {
            if attr.path().is_ident("serde") {
                quote!(#attr).to_string().contains("bound")
            } else {
                false
            }
        });

        let bound_attr = if !user_has_serde_bound {
            match (has_serialize, has_deserialize) {
                (true, true) => {
                    let ser_bound = format!("M::{}: ::formwork::serde::Serialize", mode_type_name);
                    let de_bound = format!("M::{}: ::formwork::serde::Deserialize<'de>", mode_type_name);
                    quote! { #[serde(bound(serialize = #ser_bound, deserialize = #de_bound))] }
                }
                (true, false) => {
                    let ser_bound = format!("M::{}: ::formwork::serde::Serialize", mode_type_name);
                    quote! { #[serde(bound(serialize = #ser_bound))] }
                }
                (false, true) => {
                    let de_bound = format!("M::{}: ::formwork::serde::Deserialize<'de>", mode_type_name);
                    quote! { #[serde(bound(deserialize = #de_bound))] }
                }
                (false, false) => quote! {},
            }
        } else {
            quote! {}
        };

        (quote! { Option<M::#mode_type_name> }, bound_attr)
    } else {
        (quote! { M::#mode_type_name }, quote! {})
    };
    let field_tokens = format_field(&ctx.field_attrs, &builder_attr, &serde_bound_attr, ctx.vis, ctx.field_name, &ty);

    GeneratedField {
        field_tokens,
        slot_trait: Some(slot_trait),
        mutator,
    }
}
