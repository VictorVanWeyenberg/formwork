use crate::gen_struct::context::SlotFieldContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn generate_slot_trait(ctx: &SlotFieldContext) -> TokenStream {
    let trait_def = generate_slot_trait_def(ctx);
    let trait_impls = if ctx.config.pick {
        generate_pick_slot_trait_impls(ctx)
    } else if ctx.config.option {
        generate_option_slot_trait_impls(ctx)
    } else {
        generate_standard_slot_trait_impls(ctx)
    };

    quote! {
        #trait_def
        #(#trait_impls)*
    }
}

fn generate_slot_trait_def(ctx: &SlotFieldContext) -> TokenStream {
    let trait_name = &ctx.trait_name;
    let mode_trait = &ctx.struct_idents.mode_trait;
    let mode_type_name = &ctx.mode_type_name;

    if ctx.config.collection {
        quote! {
            #[doc(hidden)]
            pub trait #trait_name<M: #mode_trait> {
                fn into_slot(self) -> <M::#mode_type_name as std::iter::IntoIterator>::Item;
            }
        }
    } else if ctx.config.option {
        quote! {
            #[doc(hidden)]
            pub trait #trait_name<M: #mode_trait> {
                fn into_slot(self) -> Option<M::#mode_type_name>;
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            pub trait #trait_name<M: #mode_trait> {
                fn into_slot(self) -> M::#mode_type_name;
            }
        }
    }
}

fn generate_collection_slot_impl(
    trait_name: &Ident,
    mode_struct: &Ident,
    elem_ty: &TokenStream,
    return_ty: &TokenStream,
    wrap: impl Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    let val_vec = wrap(quote! { self.into_iter().map(::std::convert::Into::into).collect() });
    let val_arr = wrap(quote! { self.into_iter().map(::std::convert::Into::into).collect() });
    quote! {
        #[doc(hidden)]
        impl<__V: ::std::convert::Into<#elem_ty>> #trait_name<#mode_struct> for Vec<__V> {
            fn into_slot(self) -> #return_ty { #val_vec }
        }
        #[doc(hidden)]
        impl<__V: ::std::convert::Into<#elem_ty>, const __N: usize> #trait_name<#mode_struct> for [__V; __N] {
            fn into_slot(self) -> #return_ty { #val_arr }
        }
    }
}

fn generate_pick_slot_trait_impls(ctx: &SlotFieldContext) -> Vec<TokenStream> {
    let trait_name = &ctx.trait_name;
    let wet_struct = &ctx.struct_idents.wet_struct;
    let dry_struct = &ctx.struct_idents.dry_struct;
    let slot_type = &ctx.slot_type;
    let choice_type = &ctx.choice_type;
    let wet_inner_type = &ctx.wet_inner_type;
    let dry_inner_type = &ctx.dry_inner_type;

    let mut trait_impls = vec![
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #slot_type {
                fn into_slot(self) -> #slot_type {
                    self
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #choice_type {
                fn into_slot(self) -> #slot_type {
                    ::formwork::Slot::Choose(self)
                }
            }
        },
    ];

    if ctx.config.cascade {
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl<__V: ::formwork::__private::CascadePickElement<#wet_inner_type, #dry_inner_type>>
                #trait_name<#wet_struct> for Vec<__V>
            {
                fn into_slot(self) -> #slot_type {
                    ::formwork::__private::CascadePickElement::into_multi_slot(self)
                }
            }
            #[doc(hidden)]
            impl<
                __V: ::formwork::__private::CascadePickElement<#wet_inner_type, #dry_inner_type>,
                const __N: usize,
            > #trait_name<#wet_struct> for [__V; __N]
            {
                fn into_slot(self) -> #slot_type {
                    ::formwork::__private::CascadePickElement::into_multi_slot(self)
                }
            }
        });
    } else {
        trait_impls.push(generate_collection_slot_impl(
            trait_name,
            wet_struct,
            wet_inner_type,
            slot_type,
            |val| quote! { ::formwork::Slot::FixedWet(#val) },
        ));
    }

    let return_ty_dry = quote! { Vec<#dry_inner_type> };
    trait_impls.push(generate_collection_slot_impl(
        trait_name,
        dry_struct,
        dry_inner_type,
        &return_ty_dry,
        |val| val,
    ));

    trait_impls
}

fn generate_standard_slot_trait_impls(ctx: &SlotFieldContext) -> Vec<TokenStream> {
    let trait_name = &ctx.trait_name;
    let mode_trait = &ctx.struct_idents.mode_trait;
    let wet_struct = &ctx.struct_idents.wet_struct;
    let dry_struct = &ctx.struct_idents.dry_struct;
    let mode_type_name = &ctx.mode_type_name;
    let slot_type = &ctx.slot_type;
    let choice_type = &ctx.choice_type;
    let wet_inner_type = &ctx.wet_inner_type;
    let dry_inner_type = &ctx.dry_inner_type;

    let return_ty_wet = if ctx.config.collection {
        quote! { #slot_type }
    } else {
        quote! { <#wet_struct as #mode_trait>::#mode_type_name }
    };

    vec![
        quote! {
            #[doc(hidden)]
            impl<__V: ::formwork::__private::IntoSlot<#wet_inner_type, #dry_inner_type, #choice_type>>
                #trait_name<#wet_struct> for __V
            {
                fn into_slot(self) -> #return_ty_wet {
                    ::formwork::__private::IntoSlot::into_slot(self)
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl<__V: ::std::convert::Into<#dry_inner_type>> #trait_name<#dry_struct> for __V {
                fn into_slot(self) -> #dry_inner_type {
                    self.into()
                }
            }
        },
    ]
}

fn generate_option_slot_trait_impls(ctx: &SlotFieldContext) -> Vec<TokenStream> {
    let trait_name = &ctx.trait_name;
    let wet_struct = &ctx.struct_idents.wet_struct;
    let dry_struct = &ctx.struct_idents.dry_struct;
    let slot_type = &ctx.slot_type;
    let choice_type = &ctx.choice_type;
    let wet_inner_type = &ctx.wet_inner_type;
    let dry_inner_type = &ctx.dry_inner_type;

    let opt_slot_ty = quote! { Option<#slot_type> };
    let opt_dry_ty = quote! { Option<#dry_inner_type> };

    vec![
        quote! {
            #[doc(hidden)]
            impl<__V: ::formwork::__private::IntoOptionSlot<#wet_inner_type, #dry_inner_type, #choice_type>>
                #trait_name<#wet_struct> for __V
            {
                fn into_slot(self) -> #opt_slot_ty {
                    ::formwork::__private::IntoOptionSlot::into_option_slot(self)
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl<__V: ::formwork::__private::IntoDryOption<#dry_inner_type>>
                #trait_name<#dry_struct> for __V
            {
                fn into_slot(self) -> #opt_dry_ty {
                    ::formwork::__private::IntoDryOption::into_dry_option(self)
                }
            }
        },
    ]
}
