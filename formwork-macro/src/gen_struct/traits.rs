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
                    formwork::Slot::Choose(self)
                }
            }
        },
    ];

    trait_impls.push(generate_collection_slot_impl(
        trait_name,
        wet_struct,
        wet_inner_type,
        slot_type,
        |val| quote! { formwork::Slot::FixedWet(#val) },
        !ctx.config.cascade,
    ));

    if ctx.config.cascade {
        trait_impls.push(generate_collection_slot_impl(
            trait_name,
            wet_struct,
            dry_inner_type,
            slot_type,
            |val| quote! { formwork::Slot::FixedDry(#val) },
            false,
        ));
    }

    let return_ty_dry = quote! { Vec<#dry_inner_type> };
    trait_impls.push(generate_collection_slot_impl(
        trait_name,
        dry_struct,
        dry_inner_type,
        &return_ty_dry,
        |val| val,
        !ctx.config.cascade,
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

    let mut trait_impls = vec![
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #return_ty_wet {
                fn into_slot(self) -> #return_ty_wet {
                    self
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #wet_inner_type {
                fn into_slot(self) -> #return_ty_wet {
                    formwork::Slot::FixedWet(self)
                }
            }
        },
    ];

    if ctx.config.cascade {
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #dry_inner_type {
                fn into_slot(self) -> #return_ty_wet {
                    formwork::Slot::FixedDry(self)
                }
            }
        });
    }

    trait_impls.push(quote! {
        #[doc(hidden)]
        impl #trait_name<#wet_struct> for #choice_type {
            fn into_slot(self) -> #return_ty_wet {
                formwork::Slot::Choose(self)
            }
        }
    });

    trait_impls.push(quote! {
        #[doc(hidden)]
        impl<__V: std::convert::Into<#dry_inner_type>> #trait_name<#dry_struct> for __V {
            fn into_slot(self) -> #dry_inner_type {
                self.into()
            }
        }
    });

    trait_impls
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

    let mut trait_impls = vec![
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #slot_type {
                fn into_slot(self) -> #opt_slot_ty {
                    Some(self)
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #wet_inner_type {
                fn into_slot(self) -> #opt_slot_ty {
                    Some(formwork::Slot::FixedWet(self))
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #choice_type {
                fn into_slot(self) -> #opt_slot_ty {
                    Some(formwork::Slot::Choose(self))
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #opt_slot_ty {
                fn into_slot(self) -> #opt_slot_ty {
                    self
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for Option<#wet_inner_type> {
                fn into_slot(self) -> #opt_slot_ty {
                    self.map(formwork::Slot::FixedWet)
                }
            }
        },
        quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for Option<#choice_type> {
                fn into_slot(self) -> #opt_slot_ty {
                    self.map(formwork::Slot::Choose)
                }
            }
        },
    ];

    if ctx.config.cascade {
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for #dry_inner_type {
                fn into_slot(self) -> #opt_slot_ty {
                    Some(formwork::Slot::FixedDry(self))
                }
            }
        });
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl #trait_name<#wet_struct> for Option<#dry_inner_type> {
                fn into_slot(self) -> #opt_slot_ty {
                    self.map(formwork::Slot::FixedDry)
                }
            }
        });
    }

    trait_impls.push(quote! {
        #[doc(hidden)]
        impl #trait_name<#dry_struct> for #dry_inner_type {
            fn into_slot(self) -> #opt_dry_ty {
                Some(self)
            }
        }
    });

    trait_impls.push(quote! {
        #[doc(hidden)]
        impl #trait_name<#dry_struct> for #opt_dry_ty {
            fn into_slot(self) -> #opt_dry_ty {
                self
            }
        }
    });

    trait_impls
}
