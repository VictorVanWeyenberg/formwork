use super::GeneratedField;
use crate::ast::{FormworkArgs, FormworkField};
use crate::gen_struct::context::SlotFieldContext;
use crate::gen_struct::docs::{
    extract_doc_comments, format_accepted_values_footer, format_doc_comment,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

pub(crate) fn format_field(
    attrs: &[&syn::Attribute],
    builder_attr: &TokenStream,
    vis: &syn::Visibility,
    ident: &Ident,
    ty: &TokenStream,
) -> TokenStream {
    quote! {
        #(#attrs)*
        #builder_attr
        #vis #ident: #ty,
    }
}

pub(crate) fn generate_builder_derives(
    args: &FormworkArgs,
    mode_trait: &Ident,
    mutators: &[TokenStream],
) -> TokenStream {
    if !args.builder {
        return quote! {};
    }

    if mutators.is_empty() {
        quote! {
            #[derive(typed_builder::TypedBuilder)]
            #[builder(doc)]
        }
    } else {
        quote! {
            #[formwork::__qualify_mode(#mode_trait)]
            #[derive(typed_builder::TypedBuilder)]
            #[builder(doc)]
            #[builder(mutators(
                #(#mutators)*
            ))]
        }
    }
}

pub(crate) fn generate_slot_builder_attr(
    ctx: &SlotFieldContext,
    builder_enabled: bool,
) -> TokenStream {
    if !builder_enabled {
        return quote! {};
    }

    if ctx.config.collection {
        return quote! { #[builder(default, via_mutators)] };
    }

    let field_name = ctx.field_name;
    let trait_name = &ctx.trait_name;
    let accepted = if ctx.config.option {
        "Accepts an optional item, choice, or slot."
    } else {
        "Accepts an item, choice, or slot."
    };
    let doc_str = if !ctx.doc_lines.is_empty() {
        format_doc_comment(
            &format!("Sets `{}`.", field_name),
            &ctx.doc_lines,
            Some(accepted),
        )
    } else {
        format!("Sets `{}`. {}", field_name, accepted)
    };

    if ctx.config.option {
        quote! {
            #[builder(default, setter(doc = #doc_str, transform = |item: impl #trait_name<M>| {
                item.into_slot()
            }))]
        }
    } else {
        quote! {
            #[builder(setter(doc = #doc_str, transform = |item: impl #trait_name<M>| {
                item.into_slot()
            }))]
        }
    }
}

pub(crate) fn generate_slot_collection_mutators(ctx: &SlotFieldContext) -> TokenStream {
    let field_name = ctx.field_name;
    let trait_name = &ctx.trait_name;
    let fn_name = format_ident!("with_{}", field_name);
    let wet_inner_type = &ctx.wet_inner_type;
    let wet_inner_type_str = quote!(#wet_inner_type).to_string();

    let with_footer = format_accepted_values_footer(&wet_inner_type_str, false);
    let with_fn_doc = format_doc_comment(
        &format!("Appends a single item, choice, or slot to `{}`.", field_name),
        &ctx.doc_lines,
        Some(&with_footer),
    );

    let iter_footer = format_accepted_values_footer(&wet_inner_type_str, true);
    let iter_fn_doc = format_doc_comment(
        &format!("Appends an iterator of items, choices, or slots to `{}`.", field_name),
        &ctx.doc_lines,
        Some(&iter_footer),
    );

    quote! {
        #[doc = #with_fn_doc]
        pub fn #fn_name(&mut self, item: impl #trait_name<M>) {
            self.#field_name.extend(std::iter::once(item.into_slot()));
        }

        #[doc = #iter_fn_doc]
        pub fn #field_name<I>(&mut self, items: I)
        where
            I: std::iter::IntoIterator,
            I::Item: #trait_name<M>
        {
            self.#field_name.extend(items.into_iter().map(|item| item.into_slot()));
        }
    }
}

pub(crate) fn generate_regular_field(
    field: &FormworkField,
    args: &FormworkArgs,
) -> GeneratedField {
    let field_name = field.ident.as_ref().unwrap();
    let field_attrs = field.non_slot_attrs();
    let doc_lines = extract_doc_comments(&field.attrs);
    let vis = &field.vis;
    let ty = &field.ty;

    let (builder_attr, mutator) = if let Some(base_inner_type) = field.inner_type_if_vec() {
        let builder_attr = if args.builder {
            quote! { #[builder(default, via_mutators)] }
        } else {
            quote! {}
        };
        let mutator = if args.builder {
            let fn_name = format_ident!("with_{}", field_name);
            let doc = format_doc_comment(
                &format!("Appends a single item to `{}`.", field_name),
                &doc_lines,
                None,
            );
            Some(quote! {
                #[doc = #doc]
                pub fn #fn_name(&mut self, item: impl Into<#base_inner_type>) {
                    self.#field_name.push(item.into());
                }
            })
        } else {
            None
        };
        (builder_attr, mutator)
    } else {
        let builder_attr = if args.builder {
            let doc_setter = if !doc_lines.is_empty() {
                let doc = format_doc_comment(&format!("Sets `{}`.", field_name), &doc_lines, None);
                quote! { doc = #doc, }
            } else {
                quote! {}
            };
            quote! { #[builder(setter(into, #doc_setter))] }
        } else {
            quote! {}
        };
        (builder_attr, None)
    };

    let ty_tokens = quote! { #ty };
    let field_tokens = format_field(&field_attrs, &builder_attr, vis, field_name, &ty_tokens);

    GeneratedField {
        field_tokens,
        slot_trait: None,
        mutator,
    }
}
