use crate::gen_struct::context::SlotFieldContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn generate_slot_trait(ctx: &SlotFieldContext) -> TokenStream {
    let trait_def = generate_slot_trait_def(ctx);
    let trait_impls = if ctx.config.pick {
        generate_pick_slot_trait_impls(ctx)
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
    generic_into: bool,
) -> TokenStream {
    if generic_into {
        let val_vec = wrap(quote! { self.into_iter().map(std::convert::Into::into).collect() });
        let val_arr = wrap(quote! { self.into_iter().map(std::convert::Into::into).collect() });
        quote! {
            #[doc(hidden)]
            impl<__V: std::convert::Into<#elem_ty>> #trait_name<#mode_struct> for Vec<__V> {
                fn into_slot(self) -> #return_ty { #val_vec }
            }
            #[doc(hidden)]
            impl<__V: std::convert::Into<#elem_ty>, const __N: usize> #trait_name<#mode_struct> for [__V; __N] {
                fn into_slot(self) -> #return_ty { #val_arr }
            }
        }
    } else {
        let val_vec = wrap(quote! { self });
        let val_arr = wrap(quote! { self.into_iter().collect() });
        quote! {
            #[doc(hidden)]
            impl #trait_name<#mode_struct> for Vec<#elem_ty> {
                fn into_slot(self) -> #return_ty { #val_vec }
            }
            #[doc(hidden)]
            impl<const __N: usize> #trait_name<#mode_struct> for [#elem_ty; __N] {
                fn into_slot(self) -> #return_ty { #val_arr }
            }
        }
    }
}

fn generate_pick_slot_trait_impls(ctx: &SlotFieldContext) -> Vec<TokenStream> {
    let trait_name = &ctx.trait_name;
    let unresolved_struct = &ctx.struct_idents.unresolved_struct;
    let resolved_struct = &ctx.struct_idents.resolved_struct;
    let slot_type = &ctx.slot_type;
    let choice_type = &ctx.choice_type;
    let unresolved_inner_type = &ctx.unresolved_inner_type;
    let resolved_inner_type = &ctx.resolved_inner_type;

    let mut trait_impls = vec![
        quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #slot_type {
                fn into_slot(self) -> #slot_type {
                    self
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #choice_type {
                fn into_slot(self) -> #slot_type {
                    schematic::Slot::Choose(self)
                }
            }
        },
    ];

    trait_impls.push(generate_collection_slot_impl(
        trait_name,
        unresolved_struct,
        unresolved_inner_type,
        slot_type,
        |val| quote! { schematic::Slot::FixedUnresolved(#val) },
        !ctx.config.cascade,
    ));

    if ctx.config.cascade {
        trait_impls.push(generate_collection_slot_impl(
            trait_name,
            unresolved_struct,
            resolved_inner_type,
            slot_type,
            |val| quote! { schematic::Slot::FixedResolved(#val) },
            false,
        ));
    }

    let return_ty_resolved = quote! { Vec<#resolved_inner_type> };
    trait_impls.push(generate_collection_slot_impl(
        trait_name,
        resolved_struct,
        resolved_inner_type,
        &return_ty_resolved,
        |val| val,
        !ctx.config.cascade,
    ));

    trait_impls
}

fn generate_standard_slot_trait_impls(ctx: &SlotFieldContext) -> Vec<TokenStream> {
    let trait_name = &ctx.trait_name;
    let mode_trait = &ctx.struct_idents.mode_trait;
    let unresolved_struct = &ctx.struct_idents.unresolved_struct;
    let resolved_struct = &ctx.struct_idents.resolved_struct;
    let mode_type_name = &ctx.mode_type_name;
    let slot_type = &ctx.slot_type;
    let choice_type = &ctx.choice_type;
    let unresolved_inner_type = &ctx.unresolved_inner_type;
    let resolved_inner_type = &ctx.resolved_inner_type;

    let return_ty_unresolved = if ctx.config.collection {
        quote! { #slot_type }
    } else {
        quote! { <#unresolved_struct as #mode_trait>::#mode_type_name }
    };

    let mut trait_impls = vec![
        quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #return_ty_unresolved {
                fn into_slot(self) -> #return_ty_unresolved {
                    self
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #unresolved_inner_type {
                fn into_slot(self) -> #return_ty_unresolved {
                    schematic::Slot::FixedUnresolved(self)
                }
            }
        },
    ];

    if ctx.config.cascade {
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #resolved_inner_type {
                fn into_slot(self) -> #return_ty_unresolved {
                    schematic::Slot::FixedResolved(self)
                }
            }
        });
    }

    trait_impls.push(quote! {
        #[doc(hidden)]
        impl #trait_name<#unresolved_struct> for #choice_type {
            fn into_slot(self) -> #return_ty_unresolved {
                schematic::Slot::Choose(self)
            }
        }
    });

    trait_impls.push(quote! {
        #[doc(hidden)]
        impl<__V: std::convert::Into<#resolved_inner_type>> #trait_name<#resolved_struct> for __V {
            fn into_slot(self) -> #resolved_inner_type {
                self.into()
            }
        }
    });

    trait_impls
}
