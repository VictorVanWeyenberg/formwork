use darling::{FromDeriveInput, FromField, FromMeta};
use syn::{Attribute, DeriveInput, Ident, Meta, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromMeta)]
#[darling(rename_all = "lowercase")]
pub enum TraversalStrategy {
    #[default]
    Bfs,
    Dfs,
}

#[derive(Debug, FromMeta, Default)]
pub struct SchematicArgs {
    #[darling(default)]
    pub builder: bool,
    #[darling(default)]
    pub traverse: TraversalStrategy,
    #[darling(default)]
    pub derives: Vec<syn::Path>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
pub struct SchematicStruct {
    pub ident: Ident,
    pub data: darling::ast::Data<(), SchematicField>,
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(slot, allow, doc, cfg, serde, serde_as, deny, warn))]
pub struct SchematicField {
    pub ident: Option<Ident>,
    pub vis: syn::Visibility,
    pub ty: Type,
    pub attrs: Vec<Attribute>,
}

pub struct SlotConfig {
    pub collection: bool,
    pub pick: bool,
    pub cascade: bool,
}

impl SchematicStruct {
    pub fn parse_input(item: &DeriveInput) -> Result<Self, darling::Error> {
        Self::from_derive_input(item)
    }
}

impl SchematicField {
    pub fn slot_config(&self) -> Result<Option<SlotConfig>, darling::Error> {
        let mut has_slot = false;
        let mut config = SlotConfig {
            collection: false,
            pick: false,
            cascade: false,
        };

        for attr in &self.attrs {
            if attr.path().is_ident("slot") {
                has_slot = true;
                if let Meta::List(meta) = &attr.meta {
                    let res = meta.parse_nested_meta(|meta| {
                        if meta.path.is_ident("collection") {
                            config.collection = true;
                        } else if meta.path.is_ident("pick") {
                            config.pick = true;
                        } else if meta.path.is_ident("cascade") {
                            config.cascade = true;
                        } else {
                            return Err(meta.error("unsupported slot attribute argument"));
                        }
                        Ok(())
                    });
                    if let Err(e) = res {
                        return Err(darling::Error::from(e));
                    }
                }
            }
        }

        if has_slot {
            if config.collection && config.pick {
                return Err(darling::Error::custom(
                    "`collection` and `pick` cannot be used together in #[slot]",
                )
                .with_span(&self.ident));
            }
            if config.collection && self.inner_type_if_vec().is_none() {
                return Err(darling::Error::custom(
                    "`#[slot(collection)]` requires a `Vec<T>` field type",
                )
                .with_span(&self.ty));
            }
            if config.pick && self.inner_type_if_vec().is_none() {
                return Err(
                    darling::Error::custom("`#[slot(pick)]` requires a `Vec<T>` field type")
                        .with_span(&self.ty),
                );
            }
            if config.cascade
                && !config.collection
                && !config.pick
                && self.inner_type_if_vec().is_some()
            {
                return Err(darling::Error::custom(
                    "`#[slot(cascade)]` cannot be used on `Vec<T>`; did you mean `#[slot(collection, cascade)]` or `#[slot(pick, cascade)]`?",
                )
                .with_span(&self.ty));
            }
            if config.cascade {
                let item = self.slot_item_type(&config);
                if unresolved_mode_type(item).is_none() || resolved_mode_type(item).is_none() {
                    return Err(darling::Error::custom(
                        "`#[slot(cascade)]` requires a type path that can support HKD mode types",
                    )
                    .with_span(item));
                }
            }
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub fn is_slot(&self) -> bool {
        self.slot_config().unwrap_or(None).is_some()
    }

    pub fn slot_item_type<'a>(&'a self, config: &SlotConfig) -> &'a Type {
        if config.collection || config.pick {
            self.inner_type_if_vec().unwrap_or(&self.ty)
        } else {
            &self.ty
        }
    }

    pub fn unresolved_inner_type(&self, config: &SlotConfig) -> proc_macro2::TokenStream {
        let item = self.slot_item_type(config);
        if config.cascade {
            let mode = unresolved_mode_type(item).unwrap();
            quote::quote! { #item<#mode> }
        } else {
            quote::quote! { #item }
        }
    }

    pub fn resolved_inner_type(&self, config: &SlotConfig) -> proc_macro2::TokenStream {
        let item = self.slot_item_type(config);
        if config.cascade {
            let mode = resolved_mode_type(item).unwrap();
            quote::quote! { #item<#mode> }
        } else {
            quote::quote! { #item }
        }
    }

    pub fn is_string_slot(&self, config: &SlotConfig) -> bool {
        matches!(
            self.slot_item_type(config),
            Type::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "String")
        )
    }

    pub fn inner_type_if_vec(&self) -> Option<&Type> {
        if let syn::Type::Path(syn::TypePath { path, .. }) = &self.ty
            && let Some(segment) = path.segments.last()
            && segment.ident == "Vec"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
        {
            Some(inner_ty)
        } else {
            None
        }
    }
}

pub fn unresolved_mode_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        let mut new_type_path = type_path.clone();
        if let Some(last_segment) = new_type_path.path.segments.last_mut() {
            let ident_str = last_segment.ident.to_string();
            last_segment.ident = syn::Ident::new(
                &format!("{}Unresolved", ident_str),
                last_segment.ident.span(),
            );
            last_segment.arguments = syn::PathArguments::None;
        }
        return Some(syn::Type::Path(new_type_path));
    }
    None
}

pub fn resolved_mode_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        let mut new_type_path = type_path.clone();
        if let Some(last_segment) = new_type_path.path.segments.last_mut() {
            let ident_str = last_segment.ident.to_string();
            last_segment.ident =
                syn::Ident::new(&format!("{}Resolved", ident_str), last_segment.ident.span());
            last_segment.arguments = syn::PathArguments::None;
        }
        return Some(syn::Type::Path(new_type_path));
    }
    None
}
