use crate::ast::{SchematicField, SlotConfig};
use crate::gen_struct::docs::extract_doc_comments;
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[derive(Debug)]
pub(crate) struct StructIdents<'a> {
    pub(crate) name: &'a Ident,
    pub(crate) mode_trait: Ident,
    pub(crate) unresolved_struct: Ident,
    pub(crate) resolved_struct: Ident,
    pub(crate) schematic_type: Ident,
}

impl<'a> StructIdents<'a> {
    pub(crate) fn new(name: &'a Ident) -> Self {
        Self {
            name,
            mode_trait: format_ident!("{}Mode", name),
            unresolved_struct: format_ident!("{}Unresolved", name),
            resolved_struct: format_ident!("{}Resolved", name),
            schematic_type: format_ident!("{}Schematic", name),
        }
    }
}

pub(crate) struct SlotFieldContext<'a> {
    pub(crate) struct_idents: &'a StructIdents<'a>,
    pub(crate) field_name: &'a Ident,
    pub(crate) vis: &'a syn::Visibility,
    pub(crate) field_attrs: Vec<&'a syn::Attribute>,
    pub(crate) doc_lines: Vec<String>,
    pub(crate) config: &'a SlotConfig,
    pub(crate) mode_type_name: Ident,
    pub(crate) trait_name: Ident,
    pub(crate) unresolved_inner_type: TokenStream,
    pub(crate) resolved_inner_type: TokenStream,
    pub(crate) slot_type: TokenStream,
    pub(crate) choice_type: TokenStream,
}

impl<'a> SlotFieldContext<'a> {
    pub(crate) fn new(
        field: &'a SchematicField,
        config: &'a SlotConfig,
        struct_idents: &'a StructIdents<'a>,
    ) -> Self {
        let field_name = field.ident.as_ref().unwrap();
        let camel_name = RenameRule::PascalCase.apply_to_field(field_name.to_string());
        let mode_type_name = format_ident!("{}", camel_name);
        let trait_name = format_ident!("Into{}{}Slot", struct_idents.name, camel_name);

        let unresolved_inner_type = field.unresolved_inner_type(config);
        let resolved_inner_type = field.resolved_inner_type(config);

        let slot_type = if config.pick {
            quote! { schematic::MultiSlot<#unresolved_inner_type, #resolved_inner_type> }
        } else {
            quote! { schematic::SingleSlot<#unresolved_inner_type, #resolved_inner_type> }
        };

        let choice_type = if config.pick {
            quote! { schematic::choice::MultiChoice<#unresolved_inner_type> }
        } else {
            quote! { schematic::choice::SingleChoice<#unresolved_inner_type> }
        };

        let field_attrs = field.non_slot_attrs();

        Self {
            struct_idents,
            field_name,
            vis: &field.vis,
            field_attrs,
            doc_lines: extract_doc_comments(&field.attrs),
            config,
            mode_type_name,
            trait_name,
            unresolved_inner_type,
            resolved_inner_type,
            slot_type,
            choice_type,
        }
    }
}
