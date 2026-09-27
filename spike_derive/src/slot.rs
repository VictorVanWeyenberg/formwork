use crate::format_draft_name;
use proc_macro2::Ident;
use std::ops::Not;
use syn::{
    AngleBracketedGenericArguments, Attribute, Error, Field, GenericArgument, PathArguments,
    PathSegment, Type, TypePath,
};

pub struct SlotField {
    pub field: Field,
    pub slot_kind: Option<SlotKind>,
}

impl TryFrom<Field> for SlotField {
    type Error = syn::Error;
    fn try_from(field: Field) -> Result<Self, Self::Error> {
        let (slots, plain): (Vec<&Attribute>, Vec<&Attribute>) =
            field.attrs.iter().partition(attribute_is_of_slot_kind);
        let slots: Vec<_> = slots
            .into_iter()
            .map(|attr| attribute_as_slot_kind(attr, &field))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter_map(std::convert::identity)
            .collect();
        let field = Field {
            ident: field.ident,
            attrs: plain.into_iter().cloned().collect(),
            vis: field.vis,
            colon_token: Default::default(),
            default: field.default,
            modifiers: field.modifiers,
            ty: field.ty,
        };
        if slots.len() == 0 {
            Ok(Self {
                field,
                slot_kind: None,
            })
        } else if slots.len() > 1 {
            Err(syn::Error::new_spanned(
                field,
                "Discrepant field attributes",
            ))
        } else {
            Ok(Self {
                field,
                slot_kind: slots.first().cloned(),
            })
        }
    }
}

fn attribute_is_of_slot_kind(attr: &&Attribute) -> bool {
    let path = attr.path();
    path.is_ident("slot")
        || path.is_ident("slots")
        || path.is_ident("slot_transitive")
        || path.is_ident("slots_transitive")
}

fn attribute_as_slot_kind(attr: &Attribute, field: &Field) -> syn::Result<Option<SlotKind>> {
    if attr.path().is_ident("slot") {
        if let Some(GenericArgument::Type(ty)) = get_vec_generic_type(&field.ty) {
            Ok(Some(SlotKind::Slot {
                ty: ty.clone(),
                many: true,
            }))
        } else {
            Ok(Some(SlotKind::Slot {
                ty: field.ty.clone(),
                many: false,
            }))
        }
    } else if attr.path().is_ident("slots") {
        Ok(Some(SlotKind::Slots {
            generic_argument: expect_vec(&field.ty)?.clone(),
        }))
    } else if attr.path().is_ident("slot_transitive") {
        if let Some(GenericArgument::Type(ty)) = get_vec_generic_type(&field.ty) {
            let name = expect_none_generic_path(ty)?;
            let draft_name = format_draft_name(name);
            Ok(Some(SlotKind::SlotTransitive {
                generic_type_name: name.clone(),
                draft_name,
                many: true,
            }))
        } else {
            let name = expect_none_generic_path(&field.ty)?;
            let draft_name = format_draft_name(name);
            Ok(Some(SlotKind::SlotTransitive {
                generic_type_name: name.clone(),
                draft_name,
                many: false,
            }))
        }
    } else if attr.path().is_ident("slots_transitive") {
        let name = expect_vec_generic_type_to_be_none_generic(&field.ty)?;
        let draft_name = format_draft_name(name);
        Ok(Some(SlotKind::SlotsTransitive {
            generic_type_name: name.clone(),
            draft_name,
        }))
    } else {
        return Ok(None);
    }
}

#[derive(Clone, PartialEq)]
pub enum SlotKind {
    Slot {
        ty: Type,
        many: bool,
    },
    SlotTransitive {
        generic_type_name: Ident,
        draft_name: Ident,
        many: bool,
    },
    Slots {
        generic_argument: GenericArgument,
    },
    SlotsTransitive {
        generic_type_name: Ident,
        draft_name: Ident,
    },
}

fn expect_none_generic_path(ty: &Type) -> syn::Result<&Ident> {
    match ty {
        Type::Path(TypePath {
            path: syn::Path { segments, .. },
            ..
        }) if segments.len() == 1 => segments
            .first()
            .unwrap()
            .arguments
            .is_empty()
            .not()
            .then_some(&segments.first().unwrap().ident)
            .ok_or_else(|| Error::new_spanned(ty, "Generic types are not supported here")),
        _ => Err(Error::new_spanned(ty, "Type not supported")),
    }
}

fn expect_vec(ty: &Type) -> syn::Result<&GenericArgument> {
    get_vec_generic_type(ty).ok_or(Error::new_spanned(
        ty,
        "Attribute `slots` is only supported on a Vec<> field.",
    ))
}

fn expect_vec_generic_type_to_be_none_generic(ty: &Type) -> syn::Result<&Ident> {
    expect_vec(ty).and_then(expect_none_generic_generic_argument)
}

fn expect_none_generic_generic_argument(arg: &GenericArgument) -> syn::Result<&Ident> {
    match arg {
        GenericArgument::Type(Type::Path(TypePath {
            path: syn::Path { segments, .. },
            ..
        })) => {
            if segments.len() == 1 {
                let PathSegment { ident, arguments } = segments.first().unwrap();
                if arguments.is_empty() {
                    Ok(ident)
                } else {
                    Err(Error::new_spanned(ident, "Expected none generic argument"))
                }
            } else {
                Err(Error::new_spanned(arg, "Path form not supported"))
            }
        }
        _ => Err(Error::new_spanned(arg, "Unsupported generic argument")),
    }
}

/// If type is Vec, returns the generic argument, else none.
pub fn get_vec_generic_type(ty: &Type) -> Option<&GenericArgument> {
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    if path.segments.len() != 1 {
        return None;
    }
    let segment = path.segments.first().unwrap();
    if segment.ident != "Vec" {
        return None;
    }
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
    else {
        unreachable!("Invalid vec")
    };
    Some(
        args.first()
            .expect("Vec ought to have one generic argument"),
    )
}
