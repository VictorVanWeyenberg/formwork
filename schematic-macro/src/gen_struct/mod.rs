mod builder;
mod context;
mod docs;
mod traits;

pub(crate) use context::*;

use crate::ast::{SchematicArgs, SchematicField, SchematicStruct, SlotConfig};
use builder::{
    format_field, generate_builder_derives, generate_regular_field, generate_slot_builder_attr,
    generate_slot_collection_mutators,
};
use docs::{generate_hkd_struct_doc, generate_schematic_alias_doc};
use proc_macro2::TokenStream;
use quote::quote;
use traits::generate_slot_trait;

pub(crate) struct GeneratedField {
    pub(crate) field_tokens: TokenStream,
    pub(crate) slot_trait: Option<TokenStream>,
    pub(crate) mutator: Option<TokenStream>,
}

pub fn generate_struct(
    ast: &SchematicStruct,
    original_attrs: &[syn::Attribute],
    args: &SchematicArgs,
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
    let schematic_doc = generate_schematic_alias_doc(idents.name);
    let hkd_doc = generate_hkd_struct_doc(idents.name, &idents.schematic_type);

    let name = idents.name;
    let mode_trait = &idents.mode_trait;
    let unresolved_struct = &idents.unresolved_struct;
    let resolved_struct = &idents.resolved_struct;
    let schematic_type = &idents.schematic_type;

    quote! {
        #(#slot_traits)*

        #[doc = #schematic_doc]
        pub type #schematic_type = #name<#unresolved_struct>;

        #(#original_attrs)*
        #[doc = #hkd_doc]
        #builder_derives
        pub struct #name<M: #mode_trait = #resolved_struct> {
            #(#struct_fields)*
        }
    }
}

fn generate_field(
    field: &SchematicField,
    idents: &StructIdents,
    args: &SchematicArgs,
) -> GeneratedField {
    if let Some(config) = field.slot_config().unwrap() {
        generate_slot_field(field, &config, idents, args)
    } else {
        generate_regular_field(field, args)
    }
}

fn generate_slot_field(
    field: &SchematicField,
    config: &SlotConfig,
    idents: &StructIdents,
    args: &SchematicArgs,
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
    let ty = quote! { M::#mode_type_name };
    let field_tokens = format_field(&ctx.field_attrs, &builder_attr, ctx.vis, ctx.field_name, &ty);

    GeneratedField {
        field_tokens,
        slot_trait: Some(slot_trait),
        mutator,
    }
}
