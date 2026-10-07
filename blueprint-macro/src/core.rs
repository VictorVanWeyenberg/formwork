use quote::{format_ident, quote};
use syn::{Error, ItemStruct};

fn extract_vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

pub fn generate_blueprint(item: ItemStruct) -> Result<proc_macro2::TokenStream, Error> {
    let name = &item.ident;
    
    // We only support named fields
    let fields = match &item.fields {
        syn::Fields::Named(named) => &named.named,
        _ => {
            return Err(Error::new_spanned(
                &item.ident,
                "The blueprint attribute is only supported on structs with named fields",
            ))
        }
    };

    let mut draft_fields = Vec::new();
    let mut final_fields = Vec::new();
    let mut struct_fields = Vec::new();
    let mut complete_checks = Vec::new();
    let mut resolve_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        
        let mut is_slot = false;
        let mut slot_type = "single";
        let mut is_nested = false;
        
        for attr in &field.attrs {
            if attr.path().is_ident("slot") {
                is_slot = true;
                if let syn::Meta::List(list) = &attr.meta {
                    let tokens_str = list.tokens.to_string();
                    if tokens_str.contains("vec") {
                        slot_type = "vec";
                    }
                    if tokens_str.contains("choice") {
                        slot_type = "choice";
                    }
                    if tokens_str.contains("nested") {
                        is_nested = true;
                    }
                }
            }
        }
        
        let mode_type_name = format_ident!("{}", snake_to_camel(&field_name.to_string()));

        if is_slot {
            let base_inner_type = extract_vec_inner_type(field_type).unwrap_or(field_type);
            
            let draft_inner_type = if is_nested {
                let mut path = match base_inner_type {
                    syn::Type::Path(p) => p.clone(),
                    _ => return Err(Error::new_spanned(base_inner_type, "Nested blueprint type must be a path")),
                };
                let last_segment = path.path.segments.last_mut().unwrap();
                last_segment.ident = format_ident!("{}Blueprint", last_segment.ident);
                quote! { #path }
            } else {
                quote! { #base_inner_type }
            };

            match slot_type {
                "vec" => {
                    draft_fields.push(quote! { type #mode_type_name = Vec<blueprint::Slot<#draft_inner_type>>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.iter().all(|s| s.is_complete()) });
                    
                    let resolve_expr = if is_nested {
                        quote! {
                            self.#field_name.into_iter()
                                .map(|s| s.resolve())
                                .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                                .into_iter()
                                .flatten()
                                .map(|inner| inner.resolve())
                                .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                        }
                    } else {
                        quote! {
                            self.#field_name.into_iter()
                                .map(|s| s.resolve())
                                .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                                .into_iter()
                                .flatten()
                                .collect()
                        }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });
                },
                "choice" => {
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    let resolve_expr = if is_nested {
                        quote! {
                            self.#field_name.resolve()?
                                .into_iter()
                                .map(|inner| inner.resolve())
                                .collect::<Result<Vec<_>, blueprint::error::BlueprintError>>()?
                        }
                    } else {
                        quote! { self.#field_name.resolve()? }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });
                },
                _ => {
                    draft_fields.push(quote! { type #mode_type_name = blueprint::Slot<#draft_inner_type>; });
                    final_fields.push(quote! { type #mode_type_name = #field_type; });
                    
                    complete_checks.push(quote! { self.#field_name.is_complete() });
                    let resolve_expr = if is_nested {
                        quote! { self.#field_name.resolve_exact()?.resolve()? }
                    } else {
                        quote! { self.#field_name.resolve_exact()? }
                    };
                    resolve_fields.push(quote! { #field_name: #resolve_expr });
                }
            }
        } else {
            // Not a slot, so both modes use the base type
            draft_fields.push(quote! { type #mode_type_name = #field_type; });
            final_fields.push(quote! { type #mode_type_name = #field_type; });
            
            resolve_fields.push(quote! { #field_name: self.#field_name });
        }

        struct_fields.push(quote! { pub #field_name: M::#mode_type_name, });
    }

    let is_complete_body = if complete_checks.is_empty() {
        quote! { true }
    } else {
        quote! { #(#complete_checks)&&* }
    };

    let mode_trait = format_ident!("{}Mode", name);
    let draft_struct = format_ident!("{}Draft", name);
    let final_struct = format_ident!("{}Final", name);
    let blueprint_type = format_ident!("{}Blueprint", name);

    let mode_types: Vec<_> = fields.iter().map(|f| {
        let n = format_ident!("{}", snake_to_camel(&f.ident.as_ref().unwrap().to_string()));
        quote! { type #n; }
    }).collect();

    let struct_attrs = &item.attrs;
    
    let expanded = quote! {
        pub trait #mode_trait {
            #(#mode_types)*
        }

        pub struct #draft_struct;
        pub struct #final_struct;

        impl #mode_trait for #draft_struct {
            #(#draft_fields)*
        }

        impl #mode_trait for #final_struct {
            #(#final_fields)*
        }

        pub type #blueprint_type = #name<#draft_struct>;

        #(#struct_attrs)*
        pub struct #name<M: #mode_trait = #final_struct> {
            #(#struct_fields)*
        }

        impl blueprint::Blueprint for #blueprint_type {
            type Final = #name<#final_struct>;

            fn is_complete(&self) -> bool {
                #is_complete_body
            }

            fn resolve(self) -> Result<Self::Final, blueprint::error::BlueprintError> {
                Ok(#name {
                    #(#resolve_fields),*
                })
            }
            
            fn walk_choices<V>(&mut self, _visitor: &mut V) -> blueprint::visitor::VisitorFlow
            where
                Self: blueprint::Walkable<V>,
            {
                blueprint::visitor::VisitorFlow::Continue
            }
            
            fn unmake_all(&mut self)
            where
                Self: blueprint::Walkable<blueprint::visitor::UnmakeAll>,
            {}
            
            fn confirm_all(&mut self)
            where
                Self: blueprint::Walkable<blueprint::visitor::ConfirmAll>,
            {}
        }
        
        // Dummy implementation of Walkable for any visitor until we implement actual walking
        impl<V> blueprint::Walkable<V> for #blueprint_type {
            fn walk_level(&mut self, _visitor: &mut V, _depth: usize) -> blueprint::Level {
                blueprint::Level::continue_level(false)
            }
        }
    };

    Ok(expanded)
}

fn snake_to_camel(s: &str) -> String {
    let mut camel = String::new();
    let mut next_cap = true;
    for c in s.chars() {
        if c == '_' {
            next_cap = true;
        } else if next_cap {
            camel.push(c.to_ascii_uppercase());
            next_cap = false;
        } else {
            camel.push(c);
        }
    }
    camel
}
