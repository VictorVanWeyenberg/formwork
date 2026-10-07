use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::ast::BlueprintStruct;
use crate::gen_mode::snake_to_camel;

pub fn generate_struct(ast: &BlueprintStruct) -> TokenStream {
    let name = &ast.ident;
    let mode_trait = format_ident!("{}Mode", name);
    let draft_struct = format_ident!("{}Draft", name);
    let final_struct = format_ident!("{}Final", name);
    let blueprint_type = format_ident!("{}Blueprint", name);

    let mut struct_fields = Vec::new();
    let mut mutators = Vec::new();
    let mut draft_methods = Vec::new();

    for field in &ast.fields {
        let field_name = &field.ident;
        let mode_type_name = format_ident!("{}", snake_to_camel(&field_name.to_string()));

        let is_slot = field.config.is_some();
        let is_vec_slot = field.config.map(|c| c.is_vec).unwrap_or(false);
        let is_vec_field = field.inner_type_if_vec().is_some();
        let is_vec_in_both = is_vec_slot || (is_vec_field && !is_slot);

        if is_vec_in_both {
            let fn_name = format_ident!("with_{}", field_name);
            mutators.push(quote! {
                pub fn #fn_name(&mut self, item: impl Into<<M as #mode_trait>::#mode_type_name>) {
                    self.#field_name.push(item.into());
                }
            });
            struct_fields.push(quote! {
                #[cfg_attr(feature = "builders", builder(default, via_mutators))]
                pub #field_name: Vec<<M as #mode_trait>::#mode_type_name>,
            });
        } else {
            struct_fields.push(quote! {
                #[cfg_attr(feature = "builders", builder(setter(into)))]
                pub #field_name: <M as #mode_trait>::#mode_type_name,
            });
        }

        if let Some(config) = &field.config {
            let choice_fn_name = format_ident!("add_{}_choice", field_name);
            
            let draft_inner_type = if config.is_nested {
                let base = field.base_type();
                let draft_inner_mode = format_ident!("{}Draft", base_ident_name(base).unwrap());
                quote! { #base<#draft_inner_mode> }
            } else {
                let base = field.base_type();
                quote! { #base }
            };

            if config.is_vec {
                draft_methods.push(quote! {
                    pub fn #choice_fn_name(&mut self, choice: blueprint::Choice<#draft_inner_type>) {
                        self.#field_name.push(blueprint::Slot::Choose(choice));
                    }
                });
            } else {
                draft_methods.push(quote! {
                    pub fn #choice_fn_name(&mut self, choice: blueprint::Choice<#draft_inner_type>) {
                        self.#field_name = blueprint::Slot::Choose(choice);
                    }
                });
            }
        }
    }

    let struct_attrs = &ast.attrs;

    quote! {
        pub type #blueprint_type = #name<#draft_struct>;

        #(#struct_attrs)*
        #[cfg_attr(feature = "builders", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "builders", builder(mutators(
            #(#mutators)*
        )))]
        pub struct #name<M: #mode_trait = #final_struct> {
            #(#struct_fields)*
        }

        impl #name<#draft_struct> {
            #(#draft_methods)*
        }
    }
}

fn base_ident_name(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    None
}
