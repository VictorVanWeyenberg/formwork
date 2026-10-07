use syn::{Error, Field, Ident, ItemStruct, Meta, Type};

pub struct BlueprintStruct {
    pub ident: Ident,
    pub fields: Vec<BlueprintField>,
    pub attrs: Vec<syn::Attribute>,
}

pub struct BlueprintField {
    pub ident: Ident,
    pub ty: Type,
    pub config: Option<SlotConfig>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SlotConfig {
    pub is_vec: bool,
    pub is_choice: bool,
    pub is_nested: bool,
}

impl BlueprintStruct {
    pub fn parse(item: ItemStruct) -> Result<Self, Error> {
        if !item.generics.params.is_empty() {
            return Err(Error::new_spanned(item.generics, "#[blueprint] does not support generics"));
        }

        let fields = match item.fields {
            syn::Fields::Named(fields) => fields,
            _ => return Err(Error::new_spanned(item.ident, "#[blueprint] is only allowed on structs with named fields")),
        };

        let parsed_fields = fields
            .named
            .into_iter()
            .map(BlueprintField::parse)
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(BlueprintStruct {
            ident: item.ident,
            fields: parsed_fields,
            attrs: item.attrs,
        })
    }
}

impl BlueprintField {
    fn parse(field: Field) -> Result<Self, Error> {
        let ident = field.ident.clone().unwrap();
        let mut config = None;

        for attr in &field.attrs {
            if attr.path().is_ident("slot") {
                let mut slot_config = SlotConfig::default();
                
                if let Meta::List(meta) = &attr.meta {
                    meta.parse_nested_meta(|meta| {
                        if meta.path.is_ident("vec") {
                            slot_config.is_vec = true;
                        } else if meta.path.is_ident("choice") {
                            slot_config.is_choice = true;
                        } else if meta.path.is_ident("nested") {
                            slot_config.is_nested = true;
                        } else {
                            return Err(meta.error("unsupported slot attribute argument"));
                        }
                        Ok(())
                    })?;
                }
                config = Some(slot_config);
            }
        }

        Ok(BlueprintField {
            ident,
            ty: field.ty,
            config,
        })
    }
}

impl BlueprintField {
    pub fn base_type(&self) -> &Type {
        let _is_vec = self.config.map(|c| c.is_vec).unwrap_or(false);
        let mut is_vec_field = false;
        
        let inner_type_if_vec = if let syn::Type::Path(syn::TypePath { path, .. }) = &self.ty {
            if let Some(segment) = path.segments.last() {
                if segment.ident == "Vec" {
                    is_vec_field = true;
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            Some(inner_ty)
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        } else { None };
        
        if is_vec_field {
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
