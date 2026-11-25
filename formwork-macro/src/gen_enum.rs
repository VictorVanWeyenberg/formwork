use crate::ast::{dry_mode_type, wet_mode_type, FormworkArgs, FormworkEnum, FormworkVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

struct EnumIdents<'a> {
    name: &'a Ident,
    vis: &'a syn::Visibility,
    mode_trait: Ident,
    wet_struct: Ident,
    dry_struct: Ident,
    formwork_type: Ident,
}

impl<'a> EnumIdents<'a> {
    fn new(ast: &'a FormworkEnum) -> Self {
        let name = &ast.ident;
        Self {
            name,
            vis: &ast.vis,
            mode_trait: format_ident!("{}Mode", name),
            wet_struct: format_ident!("{}Wet", name),
            dry_struct: format_ident!("{}Dry", name),
            formwork_type: format_ident!("{}Formwork", name),
        }
    }
}

pub fn generate_enum(
    ast: &FormworkEnum,
    original_attrs: &[syn::Attribute],
    args: &FormworkArgs,
) -> TokenStream {
    let idents = EnumIdents::new(ast);
    let modes = generate_enum_modes(ast, &idents, args);
    let def = generate_enum_def(ast, &idents, original_attrs);
    let fw = generate_enum_formwork(ast, &idents);
    let walkable = generate_enum_walkable(ast, &idents);
    let conversions = generate_enum_conversions(ast, &idents);

    quote! {
        #modes
        #def
        #fw
        #walkable
        #conversions
    }
}

fn variant_cfg_attrs(variant: &FormworkVariant) -> Vec<&syn::Attribute> {
    variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .collect()
}

fn variant_mode_types(inner_ty: &syn::Type) -> (TokenStream, TokenStream) {
    let wet_mode = wet_mode_type(inner_ty).expect("validated unparameterized type path");
    let dry_mode = dry_mode_type(inner_ty).expect("validated unparameterized type path");
    (quote! { #inner_ty<#wet_mode> }, quote! { #inner_ty<#dry_mode> })
}

fn generate_enum_modes(
    ast: &FormworkEnum,
    idents: &EnumIdents<'_>,
    args: &FormworkArgs,
) -> TokenStream {
    let name = idents.name;
    let vis = idents.vis;
    let mode_trait = &idents.mode_trait;
    let wet_struct = &idents.wet_struct;
    let dry_struct = &idents.dry_struct;
    let formwork_type = &idents.formwork_type;

    let mut mode_types = Vec::with_capacity(ast.variants.len());
    let mut wet_types = Vec::with_capacity(ast.variants.len());
    let mut dry_types = Vec::with_capacity(ast.variants.len());

    for variant in &ast.variants {
        let v_ident = &variant.ident;
        let cfg_attrs = variant_cfg_attrs(variant);
        let (inner_wet, inner_dry) = variant_mode_types(&variant.inner_ty);

        let assoc_doc = format!(
            "Representation type for the `{v_ident}` variant in mode `Self`.\n\n\
             - In wet mode ([`{wet_struct}`]): holds the wet formwork representation of the variant's inner type.\n\
             - In dry mode ([`{dry_struct}`]): holds the cured, watertight dry domain type.",
        );

        mode_types.push(quote! {
            #(#cfg_attrs)*
            #[doc = #assoc_doc]
            type #v_ident;
        });

        wet_types.push(quote! {
            #(#cfg_attrs)*
            type #v_ident = #inner_wet;
        });

        dry_types.push(quote! {
            #(#cfg_attrs)*
            type #v_ident = #inner_dry;
        });
    }

    let derives = if args.derives.is_empty() {
        quote! {}
    } else {
        let paths = &args.derives;
        quote! { #[derive(#(#paths),*)] }
    };

    let mode_trait_doc = format!(
        "Mode marker trait defining the Higher-Kinded Data (HKD) variant representations for [`{name}`].\n\n\
         This trait governs how enum variants are represented between the fluid wet formwork mold \
         ([`{wet_struct}`] / [`{formwork_type}`]) and the cured watertight domain model \
         ([`{dry_struct}`] / [`{name}`]).",
    );

    let wet_doc = format!(
        "Wet mode marker for [`{name}`].\n\n\
         Configures [`{name}<{wet_struct}>`] (aliased as [`{formwork_type}`]) where each variant \
         holds the wet formwork representation of its inner type.\n\n\
         Implements [`{mode_trait}`].",
    );

    let dry_doc = format!(
        "Dry mode marker for [`{name}`].\n\n\
         Configures [`{name}<{dry_struct}>`] (aliased as [`{name}`]) where each variant \
         holds its resolved concrete domain value.\n\n\
         Implements [`{mode_trait}`].",
    );

    let wet_impl_doc = format!(
        "Configures variants of [`{name}`] as wet formworks ([`{formwork_type}`]).",
    );

    let dry_impl_doc = format!(
        "Configures variants of [`{name}`] as cured, watertight domain types.",
    );

    quote! {
        #[doc = #mode_trait_doc]
        #vis trait #mode_trait {
            #(#mode_types)*
        }

        #[doc = #wet_doc]
        #derives
        #vis struct #wet_struct;

        #[doc = #dry_doc]
        #derives
        #vis struct #dry_struct;

        #[doc = #wet_impl_doc]
        impl #mode_trait for #wet_struct {
            #(#wet_types)*
        }

        #[doc = #dry_impl_doc]
        impl #mode_trait for #dry_struct {
            #(#dry_types)*
        }
    }
}

fn generate_enum_def(
    ast: &FormworkEnum,
    idents: &EnumIdents<'_>,
    original_attrs: &[syn::Attribute],
) -> TokenStream {
    let name = idents.name;
    let vis = idents.vis;
    let mode_trait = &idents.mode_trait;
    let wet_struct = &idents.wet_struct;
    let dry_struct = &idents.dry_struct;
    let formwork_type = &idents.formwork_type;

    let enum_variants = ast.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let v_attrs = &variant.attrs;
        let f_attrs = &variant.field_attrs;
        let discriminant = variant
            .discriminant
            .as_ref()
            .map(|(eq, expr)| quote! { #eq #expr });

        quote! {
            #(#v_attrs)*
            #v_ident(#(#f_attrs)* M::#v_ident) #discriminant
        }
    });

    let formwork_doc = format!(
        "Wet formwork representation for [`{name}`].\n\n\
         This type alias represents the wet formwork enum where each variant wraps the wet \
         formwork representation of its inner type.\n\n\
         Once all choices in the active variant are satisfied, call \
         [`Formwork::resolve`](::formwork::Formwork::resolve) to produce the watertight dry \
         domain model [`{name}`].",
    );

    let hkd_doc = format!(
        "\n\n# Formwork Architecture (Higher-Kinded Data)\n\n\
         This enum uses Higher-Kinded Data (HKD) parameterized by mode `M`.\n\n\
         - **Dry mode (default)**: When unparameterized, [`{name}`] represents the final, \
           watertight dry domain enum where each variant holds a resolved domain value.\n\
         - **Wet mode ([`{formwork_type}`])**: The wet formwork enum where each variant holds \
           a wet inner formwork. Instances can be constructed via enum variants or `From`/`Into` \
           conversions, traversed transparently with visitors, and resolved into [`{name}`] via \
           [`Formwork::resolve`](::formwork::Formwork::resolve).\n",
    );

    quote! {
        #[doc = #formwork_doc]
        #vis type #formwork_type = #name<#wet_struct>;

        #(#original_attrs)*
        #[doc = #hkd_doc]
        #vis enum #name<M: #mode_trait = #dry_struct> {
            #(#enum_variants),*
        }
    }
}

fn generate_enum_formwork(ast: &FormworkEnum, idents: &EnumIdents<'_>) -> TokenStream {
    let name = idents.name;
    let dry_struct = &idents.dry_struct;
    let formwork_type = &idents.formwork_type;

    let complete_arms = ast.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let cfg_attrs = variant_cfg_attrs(variant);
        quote! {
            #(#cfg_attrs)*
            #name::#v_ident(inner) => ::formwork::Formwork::is_complete(inner),
        }
    });

    let resolve_arms = ast.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let cfg_attrs = variant_cfg_attrs(variant);
        quote! {
            #(#cfg_attrs)*
            #name::#v_ident(inner) => ::std::result::Result::Ok(
                #name::#v_ident(::formwork::Formwork::resolve(inner)?)
            ),
        }
    });

    quote! {
        impl ::formwork::Formwork for #formwork_type {
            type Dry = #name<#dry_struct>;

            fn is_complete(&self) -> bool {
                match self {
                    #(#complete_arms)*
                }
            }

            fn resolve(self) -> ::std::result::Result<Self::Dry, ::formwork::FormworkError> {
                match self {
                    #(#resolve_arms)*
                }
            }
        }
    }
}

fn generate_enum_walkable(ast: &FormworkEnum, idents: &EnumIdents<'_>) -> TokenStream {
    let name = idents.name;
    let wet_struct = &idents.wet_struct;

    let where_bounds = ast.variants.iter().map(|variant| {
        let (inner_wet, _) = variant_mode_types(&variant.inner_ty);
        quote! { #inner_wet: ::formwork::Walkable<__V> }
    });

    let walk_step_arms = ast.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let cfg_attrs = variant_cfg_attrs(variant);
        quote! {
            #(#cfg_attrs)*
            #name::#v_ident(inner) => ::formwork::Walkable::<__V>::walk_step(inner, visitor, target, current),
        }
    });

    let unmake_arms = ast.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let cfg_attrs = variant_cfg_attrs(variant);
        quote! {
            #(#cfg_attrs)*
            #name::#v_ident(inner) => ::formwork::Walkable::<__V>::unmake_at_index(inner, target, current),
        }
    });

    let walk_level_arms = ast.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let cfg_attrs = variant_cfg_attrs(variant);
        quote! {
            #(#cfg_attrs)*
            #name::#v_ident(inner) => ::formwork::Walkable::<__V>::walk_step_level(inner, visitor, depth, target, current),
        }
    });

    let unmake_level_arms = ast.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let cfg_attrs = variant_cfg_attrs(variant);
        quote! {
            #(#cfg_attrs)*
            #name::#v_ident(inner) => ::formwork::Walkable::<__V>::unmake_step_level(inner, depth, target, current),
        }
    });

    quote! {
        impl<__V> ::formwork::Walkable<__V> for #name<#wet_struct>
        where
            #(#where_bounds),*
        {
            fn walk_step(
                &mut self,
                visitor: &mut __V,
                target: usize,
                current: &mut usize,
            ) -> ::formwork::StepOutcome {
                match self {
                    #(#walk_step_arms)*
                }
            }

            fn unmake_at_index(&mut self, target: usize, current: &mut usize) -> bool {
                match self {
                    #(#unmake_arms)*
                }
            }

            fn walk_step_level(
                &mut self,
                visitor: &mut __V,
                depth: usize,
                target: usize,
                current: &mut usize,
            ) -> (::formwork::StepOutcome, bool) {
                match self {
                    #(#walk_level_arms)*
                }
            }

            fn unmake_step_level(
                &mut self,
                depth: usize,
                target: usize,
                current: &mut usize,
            ) -> (bool, bool) {
                match self {
                    #(#unmake_level_arms)*
                }
            }
        }
    }
}

fn generate_enum_conversions(ast: &FormworkEnum, idents: &EnumIdents<'_>) -> TokenStream {
    let name = idents.name;
    let wet_struct = &idents.wet_struct;
    let dry_struct = &idents.dry_struct;

    let self_cascade =
        crate::gen_convert::generate_self_cascade_impls(name, wet_struct, dry_struct);

    let dry_to_wet_arms = ast.variants.iter().map(|variant| {
        let v_ident = &variant.ident;
        let cfg_attrs = variant_cfg_attrs(variant);
        quote! {
            #(#cfg_attrs)*
            #name::#v_ident(inner) => #name::#v_ident(::std::convert::Into::into(inner)),
        }
    });

    let variant_impls = ast
        .variants
        .iter()
        .map(|variant| generate_variant_conversions(variant, idents));

    quote! {
        #self_cascade

        impl ::std::convert::From<#name<#dry_struct>> for #name<#wet_struct> {
            fn from(value: #name<#dry_struct>) -> Self {
                match value {
                    #(#dry_to_wet_arms)*
                }
            }
        }

        #(#variant_impls)*
    }
}

fn generate_variant_conversions(variant: &FormworkVariant, idents: &EnumIdents<'_>) -> TokenStream {
    let name = idents.name;
    let wet_struct = &idents.wet_struct;
    let dry_struct = &idents.dry_struct;
    let v_ident = &variant.ident;
    let cfg_attrs = variant_cfg_attrs(variant);
    let (inner_wet, inner_dry) = variant_mode_types(&variant.inner_ty);

    let enum_wet = quote! { #name<#wet_struct> };
    let enum_dry = quote! { #name<#dry_struct> };

    quote! {
        #(#cfg_attrs)*
        impl ::std::convert::From<#inner_dry> for #enum_dry {
            fn from(value: #inner_dry) -> Self {
                #name::#v_ident(value)
            }
        }

        #(#cfg_attrs)*
        impl ::std::convert::From<#inner_wet> for #enum_wet {
            fn from(value: #inner_wet) -> Self {
                #name::#v_ident(value)
            }
        }

        #(#cfg_attrs)*
        impl ::std::convert::From<#inner_dry> for #enum_wet {
            fn from(value: #inner_dry) -> Self {
                #name::#v_ident(::std::convert::Into::into(value))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoSlot<#enum_wet, #enum_dry, __C> for #inner_wet {
            fn into_slot(self) -> ::formwork::Slot<#enum_wet, #enum_dry, __C> {
                ::formwork::Slot::FixedWet(#name::#v_ident(self))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoSlot<#enum_wet, #enum_dry, __C> for #inner_dry {
            fn into_slot(self) -> ::formwork::Slot<#enum_wet, #enum_dry, __C> {
                ::formwork::Slot::FixedDry(#name::#v_ident(self))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoOptionSlot<#enum_wet, #enum_dry, __C> for #inner_wet {
            fn into_option_slot(
                self,
            ) -> ::std::option::Option<::formwork::Slot<#enum_wet, #enum_dry, __C>> {
                ::std::option::Option::Some(::formwork::Slot::FixedWet(#name::#v_ident(self)))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoOptionSlot<#enum_wet, #enum_dry, __C>
            for ::std::option::Option<#inner_wet>
        {
            fn into_option_slot(
                self,
            ) -> ::std::option::Option<::formwork::Slot<#enum_wet, #enum_dry, __C>> {
                self.map(|v| ::formwork::Slot::FixedWet(#name::#v_ident(v)))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoOptionSlot<#enum_wet, #enum_dry, __C> for #inner_dry {
            fn into_option_slot(
                self,
            ) -> ::std::option::Option<::formwork::Slot<#enum_wet, #enum_dry, __C>> {
                ::std::option::Option::Some(::formwork::Slot::FixedDry(#name::#v_ident(self)))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoOptionSlot<#enum_wet, #enum_dry, __C>
            for ::std::option::Option<#inner_dry>
        {
            fn into_option_slot(
                self,
            ) -> ::std::option::Option<::formwork::Slot<#enum_wet, #enum_dry, __C>> {
                self.map(|v| ::formwork::Slot::FixedDry(#name::#v_ident(v)))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl ::formwork::__private::CascadePickElement<#enum_wet, #enum_dry> for #inner_wet {
            fn into_multi_slot<__I: ::std::iter::IntoIterator<Item = Self>>(
                items: __I,
            ) -> ::formwork::MultiSlot<#enum_wet, #enum_dry> {
                ::formwork::Slot::FixedWet(items.into_iter().map(#name::#v_ident).collect())
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl ::formwork::__private::CascadePickElement<#enum_wet, #enum_dry> for #inner_dry {
            fn into_multi_slot<__I: ::std::iter::IntoIterator<Item = Self>>(
                items: __I,
            ) -> ::formwork::MultiSlot<#enum_wet, #enum_dry> {
                ::formwork::Slot::FixedDry(items.into_iter().map(#name::#v_ident).collect())
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoSlot<#enum_dry, #enum_dry, __C> for #inner_dry {
            fn into_slot(self) -> ::formwork::Slot<#enum_dry, #enum_dry, __C> {
                ::formwork::Slot::FixedWet(#name::#v_ident(self))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoOptionSlot<#enum_dry, #enum_dry, __C> for #inner_dry {
            fn into_option_slot(
                self,
            ) -> ::std::option::Option<::formwork::Slot<#enum_dry, #enum_dry, __C>> {
                ::std::option::Option::Some(::formwork::Slot::FixedWet(#name::#v_ident(self)))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoOptionSlot<#enum_dry, #enum_dry, __C>
            for ::std::option::Option<#inner_dry>
        {
            fn into_option_slot(
                self,
            ) -> ::std::option::Option<::formwork::Slot<#enum_dry, #enum_dry, __C>> {
                self.map(|v| ::formwork::Slot::FixedWet(#name::#v_ident(v)))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl ::formwork::__private::IntoDryOption<#enum_dry> for #inner_dry {
            fn into_dry_option(self) -> ::std::option::Option<#enum_dry> {
                ::std::option::Option::Some(#name::#v_ident(self))
            }
        }

        #(#cfg_attrs)*
        #[doc(hidden)]
        impl ::formwork::__private::IntoDryOption<#enum_dry> for ::std::option::Option<#inner_dry> {
            fn into_dry_option(self) -> ::std::option::Option<#enum_dry> {
                self.map(#name::#v_ident)
            }
        }
    }
}
