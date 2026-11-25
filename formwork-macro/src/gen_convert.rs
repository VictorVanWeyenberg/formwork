use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate_self_cascade_impls(
    name: &Ident,
    wet_struct: &Ident,
    dry_struct: &Ident,
) -> TokenStream {
    quote! {
        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoSlot<#name<#wet_struct>, #name<#dry_struct>, __C>
            for #name<#dry_struct>
        {
            fn into_slot(self) -> ::formwork::Slot<#name<#wet_struct>, #name<#dry_struct>, __C> {
                ::formwork::Slot::FixedDry(self)
            }
        }

        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoOptionSlot<#name<#wet_struct>, #name<#dry_struct>, __C>
            for #name<#dry_struct>
        {
            fn into_option_slot(
                self,
            ) -> ::std::option::Option<
                ::formwork::Slot<#name<#wet_struct>, #name<#dry_struct>, __C>,
            > {
                ::std::option::Option::Some(::formwork::Slot::FixedDry(self))
            }
        }

        #[doc(hidden)]
        impl<__C> ::formwork::__private::IntoOptionSlot<#name<#wet_struct>, #name<#dry_struct>, __C>
            for ::std::option::Option<#name<#dry_struct>>
        {
            fn into_option_slot(
                self,
            ) -> ::std::option::Option<
                ::formwork::Slot<#name<#wet_struct>, #name<#dry_struct>, __C>,
            > {
                self.map(::formwork::Slot::FixedDry)
            }
        }

        #[doc(hidden)]
        impl ::formwork::__private::CascadePickElement<#name<#wet_struct>, #name<#dry_struct>>
            for #name<#dry_struct>
        {
            fn into_multi_slot<__I: ::std::iter::IntoIterator<Item = Self>>(
                items: __I,
            ) -> ::formwork::MultiSlot<#name<#wet_struct>, #name<#dry_struct>> {
                ::formwork::Slot::FixedDry(items.into_iter().collect())
            }
        }
    }
}
