use darling::util::SpannedValue;
use darling::{FromDeriveInput, FromField, FromMeta};
use std::collections::HashSet;
use syn::{Attribute, DeriveInput, Ident, Meta, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromMeta)]
#[darling(rename_all = "lowercase")]
pub enum TraversalStrategy {
    #[default]
    Bfs,
    Dfs,
}

#[derive(Debug, FromMeta, Default)]
pub struct FormworkArgs {
    #[darling(default)]
    pub builder: Option<SpannedValue<bool>>,
    #[darling(default)]
    pub traverse: Option<SpannedValue<TraversalStrategy>>,
    #[darling(default)]
    pub derives: Vec<syn::Path>,
}

impl FormworkArgs {
    pub fn builder_enabled(&self) -> bool {
        self.builder.as_ref().is_some_and(|b| **b)
    }

    pub fn traverse_strategy(&self) -> TraversalStrategy {
        self.traverse.as_ref().map(|t| **t).unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct FormworkEnum {
    pub vis: syn::Visibility,
    pub ident: Ident,
    pub variants: Vec<FormworkVariant>,
}

#[derive(Debug)]
pub struct FormworkVariant {
    pub ident: Ident,
    pub attrs: Vec<Attribute>,
    pub field_attrs: Vec<Attribute>,
    pub inner_ty: Type,
    pub discriminant: Option<(syn::token::Eq, syn::Expr)>,
}

impl FormworkEnum {
    pub fn parse_input(item: &DeriveInput, args: &FormworkArgs) -> Result<Self, darling::Error> {
        if let Some(builder) = &args.builder {
            return Err(darling::Error::custom(
                "`#[formwork(builder)]` is not supported on enums",
            )
            .with_span(&builder.span()));
        }

        if let Some(traverse) = &args.traverse {
            return Err(darling::Error::custom(
                "`#[formwork(traverse = \"...\")]` is not supported on enums; enums transparently delegate traversal to their active inner formwork",
            )
            .with_span(&traverse.span()));
        }

        if !item.generics.params.is_empty() {
            return Err(darling::Error::custom(
                "`#[formwork]` does not support generic parameters or lifetimes on enums",
            )
            .with_span(&item.generics));
        }

        if let Some(where_clause) = &item.generics.where_clause {
            return Err(darling::Error::custom(
                "`#[formwork]` does not support `where` clauses on enums",
            )
            .with_span(where_clause));
        }

        let syn::Data::Enum(data_enum) = &item.data else {
            return Err(
                darling::Error::custom("expected an enum").with_span(&item.ident)
            );
        };

        if data_enum.variants.is_empty() {
            return Err(
                darling::Error::custom("`#[formwork]` does not support empty enums")
                    .with_span(&item.ident),
            );
        }

        let mut variants = Vec::with_capacity(data_enum.variants.len());
        let mut seen_inner_types: HashSet<(bool, Vec<String>)> = HashSet::new();

        for variant in &data_enum.variants {
            if let Some(slot_attr) = variant.attrs.iter().find(|a| a.path().is_ident("slot")) {
                return Err(darling::Error::custom(
                    "`#[slot]` attributes are not allowed on enum variants",
                )
                .with_span(slot_attr));
            }

            for field in &variant.fields {
                if let Some(slot_attr) = field.attrs.iter().find(|a| a.path().is_ident("slot")) {
                    return Err(darling::Error::custom(
                        "`#[slot]` attributes are not allowed on enum variant fields",
                    )
                    .with_span(slot_attr));
                }
            }

            let field = match &variant.fields {
                syn::Fields::Unit => {
                    return Err(darling::Error::custom(
                        "`#[formwork]` enums only support single-field tuple variants (e.g. `Variant(InnerType)`), not unit variants",
                    )
                    .with_span(&variant.ident));
                }
                syn::Fields::Named(named) => {
                    return Err(darling::Error::custom(
                        "`#[formwork]` enums only support single-field tuple variants (e.g. `Variant(InnerType)`), not struct variants",
                    )
                    .with_span(named));
                }
                syn::Fields::Unnamed(unnamed) => {
                    if unnamed.unnamed.is_empty() {
                        return Err(darling::Error::custom(
                            "`#[formwork]` enums only support single-field tuple variants (e.g. `Variant(InnerType)`), not zero-field tuple variants",
                        )
                        .with_span(unnamed));
                    }
                    if unnamed.unnamed.len() > 1 {
                        return Err(darling::Error::custom(
                            "`#[formwork]` enums only support single-field tuple variants (e.g. `Variant(InnerType)`), not multi-field tuple variants",
                        )
                        .with_span(unnamed));
                    }
                    &unnamed.unnamed[0]
                }
            };

            if !is_unparameterized_type_path(&field.ty) {
                return Err(darling::Error::custom(
                    "`#[formwork]` enum variant inner type must be an unparameterized type path referring to a formwork type",
                )
                .with_span(&field.ty));
            }

            if let Some(key) = normalized_type_path_key(&field.ty)
                && !seen_inner_types.insert(key)
            {
                return Err(darling::Error::custom(
                    "duplicate inner type in `#[formwork]` enum; each variant must wrap a distinct formwork type",
                )
                .with_span(&field.ty));
            }

            variants.push(FormworkVariant {
                ident: variant.ident.clone(),
                attrs: variant.attrs.clone(),
                field_attrs: field.attrs.clone(),
                inner_ty: field.ty.clone(),
                discriminant: variant.discriminant.clone(),
            });
        }

        Ok(Self {
            vis: item.vis.clone(),
            ident: item.ident.clone(),
            variants,
        })
    }
}

pub fn is_unparameterized_type_path(ty: &syn::Type) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };
    type_path.qself.is_none()
        && !type_path.path.segments.is_empty()
        && type_path
            .path
            .segments
            .iter()
            .all(|seg| seg.arguments.is_empty())
}

fn normalized_type_path_key(ty: &syn::Type) -> Option<(bool, Vec<String>)> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    Some((
        type_path.path.leading_colon.is_some(),
        type_path
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect(),
    ))
}

#[derive(Debug, FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
pub struct FormworkStruct {
    pub ident: Ident,
    pub data: darling::ast::Data<(), FormworkField>,
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(slot, allow, doc, cfg, serde, serde_as, deny, warn, builder))]
pub struct FormworkField {
    pub ident: Option<Ident>,
    pub vis: syn::Visibility,
    pub ty: Type,
    pub attrs: Vec<Attribute>,
}

pub struct SlotConfig {
    pub collection: bool,
    pub pick: bool,
    pub cascade: bool,
    pub option: bool,
    pub freeform: bool,
}

impl FormworkStruct {
    pub fn parse_input(item: &DeriveInput) -> Result<Self, darling::Error> {
        Self::from_derive_input(item)
    }
}

impl FormworkField {
    pub fn slot_config(&self) -> Result<Option<SlotConfig>, darling::Error> {
        let mut has_slot = false;
        let mut config = SlotConfig {
            collection: false,
            pick: false,
            cascade: false,
            option: false,
            freeform: false,
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
                        } else if meta.path.is_ident("option") {
                            config.option = true;
                        } else if meta.path.is_ident("cascade") {
                            config.cascade = true;
                        } else if meta.path.is_ident("freeform") {
                            config.freeform = true;
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
            if config.freeform && config.pick {
                return Err(darling::Error::custom(
                    "`freeform` and `pick` cannot be used together in #[slot]",
                )
                .with_span(&self.ident));
            }
            if config.freeform && config.cascade {
                return Err(darling::Error::custom(
                    "`freeform` cannot be combined with `cascade` in #[slot]",
                )
                .with_span(&self.ident));
            }
            if config.collection && config.pick {
                return Err(darling::Error::custom(
                    "`collection` and `pick` cannot be used together in #[slot]",
                )
                .with_span(&self.ident));
            }
            if config.option && (config.collection || config.pick) {
                return Err(darling::Error::custom(
                    "`option` cannot be combined with `collection` or `pick` in #[slot]",
                )
                .with_span(&self.ident));
            }
            if config.option && self.inner_type_if_option().is_none() {
                return Err(darling::Error::custom(
                    "`#[slot(option)]` requires an `Option<T>` field type",
                )
                .with_span(&self.ty));
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
            if config.cascade
                && !config.option
                && self.inner_type_if_option().is_some()
            {
                return Err(darling::Error::custom(
                    "`#[slot(cascade)]` cannot be used on `Option<T>`; did you mean `#[slot(option, cascade)]`?",
                )
                .with_span(&self.ty));
            }
            if config.cascade {
                let item = self.slot_item_type(&config);
                if wet_mode_type(item).is_none() || dry_mode_type(item).is_none() {
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

    pub fn non_slot_attrs(&self, builder_enabled: bool) -> Vec<&Attribute> {
        self.attrs
            .iter()
            .filter(|attr| {
                if attr.path().is_ident("slot") {
                    return false;
                }
                if !builder_enabled && attr.path().is_ident("builder") {
                    return false;
                }
                true
            })
            .collect()
    }

    pub fn slot_item_type<'a>(&'a self, config: &SlotConfig) -> &'a Type {
        if config.collection || config.pick {
            self.inner_type_if_vec().unwrap_or(&self.ty)
        } else if config.option {
            self.inner_type_if_option().unwrap_or(&self.ty)
        } else {
            &self.ty
        }
    }

    pub fn wet_inner_type(&self, config: &SlotConfig) -> proc_macro2::TokenStream {
        let item = self.slot_item_type(config);
        if config.cascade {
            let mode = wet_mode_type(item).unwrap();
            quote::quote! { #item<#mode> }
        } else {
            quote::quote! { #item }
        }
    }

    pub fn dry_inner_type(&self, config: &SlotConfig) -> proc_macro2::TokenStream {
        let item = self.slot_item_type(config);
        if config.cascade {
            let mode = dry_mode_type(item).unwrap();
            quote::quote! { #item<#mode> }
        } else {
            quote::quote! { #item }
        }
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

    pub fn inner_type_if_option(&self) -> Option<&Type> {
        if let syn::Type::Path(syn::TypePath { path, .. }) = &self.ty
            && let Some(segment) = path.segments.last()
            && segment.ident == "Option"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
        {
            Some(inner_ty)
        } else {
            None
        }
    }
}

pub fn wet_mode_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        let mut new_type_path = type_path.clone();
        if let Some(last_segment) = new_type_path.path.segments.last_mut() {
            let ident_str = last_segment.ident.to_string();
            last_segment.ident = syn::Ident::new(
                &format!("{}Wet", ident_str),
                last_segment.ident.span(),
            );
            last_segment.arguments = syn::PathArguments::None;
        }
        return Some(syn::Type::Path(new_type_path));
    }
    None
}

pub fn dry_mode_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        let mut new_type_path = type_path.clone();
        if let Some(last_segment) = new_type_path.path.segments.last_mut() {
            let ident_str = last_segment.ident.to_string();
            last_segment.ident =
                syn::Ident::new(&format!("{}Dry", ident_str), last_segment.ident.span());
            last_segment.arguments = syn::PathArguments::None;
        }
        return Some(syn::Type::Path(new_type_path));
    }
    None
}
