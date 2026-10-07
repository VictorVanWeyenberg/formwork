use darling::{FromDeriveInput, FromField, FromMeta};
use syn::{Attribute, DeriveInput, Ident, Meta, Type};

#[derive(Debug, FromMeta, Default)]
pub struct SchematicArgs {
    #[darling(default)]
    pub builder: bool,
    #[darling(default)]
    pub derives: darling::util::PathList,
}

#[derive(Debug, FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
pub struct SchematicStruct {
    pub ident: Ident,
    pub data: darling::ast::Data<(), SchematicField>,
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(slot, allow, doc, cfg))]
pub struct SchematicField {
    pub ident: Option<Ident>,
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
        let mut config = SlotConfig { collection: false, pick: false, cascade: false };

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
                return Err(darling::Error::custom("`collection` and `pick` cannot be used together in #[slot]").with_span(&self.ident));
            }
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub fn is_slot(&self) -> bool {
        self.slot_config().unwrap_or(None).is_some()
    }

    pub fn base_type(&self) -> &Type {
        let is_collection_field = self.inner_type_if_vec().is_some();
        if is_collection_field {
            self.inner_type_if_vec().unwrap()
        } else {
            &self.ty
        }
    }

    pub fn inner_type_if_vec(&self) -> Option<&Type> {
        if let syn::Type::Path(syn::TypePath { path, .. }) = &self.ty {
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
}

pub fn base_ident_name(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    None
}
