use crate::ast::{FormworkField, SlotConfig};
use crate::gen_struct::docs::extract_doc_comments;
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[derive(Debug)]
pub(crate) struct StructIdents<'a> {
    pub(crate) name: &'a Ident,
    pub(crate) mode_trait: Ident,
    pub(crate) wet_struct: Ident,
    pub(crate) dry_struct: Ident,
    pub(crate) formwork_type: Ident,
}

impl<'a> StructIdents<'a> {
    pub(crate) fn new(name: &'a Ident) -> Self {
        Self {
            name,
            mode_trait: format_ident!("{}Mode", name),
            wet_struct: format_ident!("{}Wet", name),
            dry_struct: format_ident!("{}Dry", name),
            formwork_type: format_ident!("{}Formwork", name),
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
    pub(crate) wet_inner_type: TokenStream,
    pub(crate) dry_inner_type: TokenStream,
    pub(crate) slot_type: TokenStream,
    pub(crate) choice_type: TokenStream,
}

impl<'a> SlotFieldContext<'a> {
    pub(crate) fn new(
        field: &'a FormworkField,
        config: &'a SlotConfig,
        struct_idents: &'a StructIdents<'a>,
    ) -> Self {
        let field_name = field.ident.as_ref().unwrap();
        let camel_name = RenameRule::PascalCase.apply_to_field(field_name.to_string());
        let mode_type_name = format_ident!("{}", camel_name);
        let trait_name = format_ident!("Into{}{}Slot", struct_idents.name, camel_name);

        let wet_inner_type = field.wet_inner_type(config);
        let dry_inner_type = field.dry_inner_type(config);

        let slot_type = if config.pick {
            quote! { formwork::MultiSlot<#wet_inner_type, #dry_inner_type> }
        } else {
            quote! { formwork::SingleSlot<#wet_inner_type, #dry_inner_type> }
        };

        let choice_type = if config.pick {
            quote! { formwork::choice::MultiChoice<#wet_inner_type> }
        } else {
            quote! { formwork::choice::SingleChoice<#wet_inner_type> }
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
            wet_inner_type,
            dry_inner_type,
            slot_type,
            choice_type,
        }
    }
}
