//! `#[derive(Blueprint)]` skeleton (S0 item 6, ticket T7).
//!
//! Input:  `struct X<M: Mode = Final> { plain: P, .., slots: Vec<M::Kind>, .. }`
//!         (`Vec<<M as Mode>::Kind>` is accepted too — typed-builder needs that spelling)
//! Output: `impl Blueprint for X<Draft>`, `impl From<X<Final>> for X<Draft>` (lift) and
//!         `impl From<X<Final>> for Slot<X<Draft>>`.
//!
//! A *slot field* is exactly a field typed `Vec<M::Kind>`; any other use of `M` is an error.
//! Paths are resolved against `crate::blueprint`, i.e. the derive is meant to be used inside the
//! crate that defines the blueprint machinery (a `#[blueprint(crate = "...")]` attribute would
//! lift that restriction later).

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DeriveInput, Error, Fields, GenericArgument, GenericParam, Ident,
    PathArguments, Type,
};

#[proc_macro_attribute]
pub fn blueprint(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let mode = mode_param(&input)?;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => return Err(Error::new_spanned(name, "#[derive(Blueprint)] requires named fields")),
        },
        _ => return Err(Error::new_spanned(name, "#[derive(Blueprint)] only supports structs")),
    };

    let mut plain = Vec::new();
    let mut slots = Vec::new();
    for field in fields {
        let ident = field.ident.as_ref().expect("named field");
        if is_slot(&field.ty, mode) {
            slots.push(ident);
        } else if mentions(&field.ty, mode) {
            return Err(Error::new_spanned(
                &field.ty,
                format!("unsupported slot shape: expected `Vec<{mode}::Kind>`"),
            ));
        } else {
            plain.push(ident);
        }
    }

    let bp = quote!(crate::blueprint);
    Ok(quote! {
        #input
        
        #[automatically_derived]
        impl #bp::Blueprint for #name<#bp::Draft> {
            type Final = #name<#bp::Final>;

            fn resolve(self) -> ::core::result::Result<Self::Final, #bp::ChoiceError> {
                let #name { #(#plain,)* #(#slots,)* } = self;
                ::core::result::Result::Ok(#name {
                    #(#plain,)*
                    #(#slots: #bp::resolve_slots(#slots)?,)*
                })
            }

            fn visible_choices(&self) -> ::std::vec::Vec<&dyn #bp::ChoicePrompt> {
                let mut out: ::std::vec::Vec<&dyn #bp::ChoicePrompt> = ::std::vec::Vec::new();
                #( out.extend(self.#slots.iter().flat_map(#bp::Slot::visible_choices)); )*
                out
            }

            fn next_open_choice_mut(&mut self) -> ::core::option::Option<&mut dyn #bp::ChoicePrompt> {
                #(
                    if let ::core::option::Option::Some(choice) =
                        self.#slots.iter_mut().find_map(#bp::Slot::next_open_choice_mut)
                    {
                        return ::core::option::Option::Some(choice);
                    }
                )*
                ::core::option::Option::None
            }
        }

        #[automatically_derived]
        impl ::core::convert::From<#name<#bp::Final>> for #name<#bp::Draft> {
            fn from(value: #name<#bp::Final>) -> Self {
                let #name { #(#plain,)* #(#slots,)* } = value;
                #name {
                    #(#plain,)*
                    #(#slots: #slots.into_iter().map(::core::convert::Into::into).collect(),)*
                }
            }
        }

        #[automatically_derived]
        impl ::core::convert::From<#name<#bp::Final>> for #bp::Slot<#name<#bp::Draft>> {
            fn from(value: #name<#bp::Final>) -> Self {
                #bp::Slot::Fixed(::core::convert::Into::into(value))
            }
        }
    })
}

/// The single type parameter of the struct: its mode.
fn mode_param(input: &DeriveInput) -> syn::Result<&Ident> {
    let generics = &input.generics;
    let mut types = generics.params.iter().filter_map(|param| match param {
        GenericParam::Type(ty) => Some(&ty.ident),
        _ => None,
    });
    let mode = types.next();
    let others = generics.params.iter().any(|p| !matches!(p, GenericParam::Type(_)));
    match (mode, types.next(), others) {
        (Some(mode), None, false) => Ok(mode),
        _ => Err(Error::new_spanned(
            generics,
            "#[derive(Blueprint)] expects exactly one generic parameter: the mode `M: Mode`",
        )),
    }
}

/// `Vec<M::Kind>` or `Vec<<M as Trait>::Kind>` and nothing else.
fn is_slot(ty: &Type, mode: &Ident) -> bool {
    let Type::Path(outer) = ty else { return false };
    if outer.qself.is_some() || outer.path.segments.len() != 1 {
        return false;
    }
    let vec = &outer.path.segments[0];
    if vec.ident != "Vec" {
        return false;
    }
    let PathArguments::AngleBracketed(args) = &vec.arguments else { return false };
    if args.args.len() != 1 {
        return false;
    }
    let Some(GenericArgument::Type(Type::Path(inner))) = args.args.first() else { return false };
    if !inner.path.segments.iter().all(|s| s.arguments.is_none()) {
        return false;
    }
    match &inner.qself {
        // `M::Kind`
        None => inner.path.segments.len() == 2 && inner.path.segments[0].ident == *mode,
        // `<M as Mode>::Kind`
        Some(qself) => {
            let Type::Path(self_ty) = &*qself.ty else { return false };
            self_ty.qself.is_none()
                && self_ty.path.is_ident(mode)
                && qself.position + 1 == inner.path.segments.len()
        }
    }
}

fn mentions(ty: &Type, mode: &Ident) -> bool {
    fn walk(stream: proc_macro2::TokenStream, mode: &Ident) -> bool {
        stream.into_iter().any(|tree| match tree {
            TokenTree::Ident(ident) => ident == *mode,
            TokenTree::Group(group) => walk(group.stream(), mode),
            _ => false,
        })
    }
    walk(ty.to_token_stream(), mode)
}
