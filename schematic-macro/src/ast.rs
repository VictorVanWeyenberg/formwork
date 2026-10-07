use syn::{Error, Field, Ident, ItemStruct, Meta, Type};

pub struct SchematicStruct {
    pub ident: Ident,
    pub fields: Vec<SchematicField>,
    pub attrs: Vec<syn::Attribute>,
    pub generate_builder: bool,
}

pub struct SchematicField {
    pub ident: Ident,
    pub ty: Type,
    pub config: Option<SlotConfig>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SlotConfig {
    pub is_collection: bool,
    pub is_pick: bool,
    pub is_cascade: bool,
}

impl SchematicStruct {
    pub fn parse(item: ItemStruct) -> Result<Self, Error> {
        if !item.generics.params.is_empty() {
            return Err(Error::new_spanned(item.generics, "#[schematic] does not support generics"));
        }

        let fields = match item.fields {
            syn::Fields::Named(fields) => fields,
            _ => return Err(Error::new_spanned(item.ident, "#[schematic] is only allowed on structs with named fields")),
        };

        let parsed_fields = fields
            .named
            .into_iter()
            .map(SchematicField::parse)
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(SchematicStruct {
            ident: item.ident,
            fields: parsed_fields,
            attrs: item.attrs,
            generate_builder: false, // Set this in lib.rs
        })
    }
}

impl SchematicField {
    fn parse(field: Field) -> Result<Self, Error> {
        let ident = field.ident.clone().unwrap();
        let mut config = None;

        for attr in &field.attrs {
            if attr.path().is_ident("slot") {
                let mut slot_config = SlotConfig::default();
                
                if let Meta::List(meta) = &attr.meta {
                    meta.parse_nested_meta(|meta| {
                        if meta.path.is_ident("collection") {
                            slot_config.is_collection = true;
                        } else if meta.path.is_ident("pick") {
                            slot_config.is_pick = true;
                        } else if meta.path.is_ident("cascade") {
                            slot_config.is_cascade = true;
                        } else {
                            return Err(meta.error("unsupported slot attribute argument"));
                        }
                        Ok(())
                    })?;
                }
                config = Some(slot_config);
            }
        }

        Ok(SchematicField {
            ident,
            ty: field.ty,
            config,
        })
    }
}

impl SchematicField {
    pub fn base_type(&self) -> &Type {
        let _is_collection = self.config.map(|c| c.is_collection).unwrap_or(false);
        let mut is_collection_field = false;
        
        let inner_type_if_vec = if let syn::Type::Path(syn::TypePath { path, .. }) = &self.ty {
            if let Some(segment) = path.segments.last() {
                if segment.ident == "Vec" {
                    is_collection_field = true;
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            Some(inner_ty)
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        } else { None };
        
        if is_collection_field {
            inner_type_if_vec.unwrap_or(&self.ty)
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
