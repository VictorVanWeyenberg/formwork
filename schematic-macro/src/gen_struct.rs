use crate::ast::{SchematicArgs, SchematicField, SchematicStruct, SlotConfig};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

struct StructIdents<'a> {
    name: &'a Ident,
    mode_trait: Ident,
    unresolved_struct: Ident,
    resolved_struct: Ident,
    schematic_type: Ident,
}

impl<'a> StructIdents<'a> {
    fn new(name: &'a Ident) -> Self {
        Self {
            name,
            mode_trait: format_ident!("{}Mode", name),
            unresolved_struct: format_ident!("{}Unresolved", name),
            resolved_struct: format_ident!("{}Resolved", name),
            schematic_type: format_ident!("{}Schematic", name),
        }
    }
}

struct SlotFieldContext<'a> {
    struct_idents: &'a StructIdents<'a>,
    field_name: &'a Ident,
    vis: &'a syn::Visibility,
    field_attrs: Vec<&'a syn::Attribute>,
    doc_lines: Vec<String>,
    config: &'a SlotConfig,
    mode_type_name: Ident,
    trait_name: Ident,
    unresolved_inner_type: TokenStream,
    resolved_inner_type: TokenStream,
    slot_type: TokenStream,
    choice_type: TokenStream,
}

impl<'a> SlotFieldContext<'a> {
    fn new(
        field: &'a SchematicField,
        config: &'a SlotConfig,
        struct_idents: &'a StructIdents<'a>,
    ) -> Self {
        let field_name = field.ident.as_ref().unwrap();
        let camel_name = RenameRule::PascalCase.apply_to_field(field_name.to_string());
        let mode_type_name = format_ident!("{}", camel_name);
        let trait_name = format_ident!("Into{}{}Slot", struct_idents.name, camel_name);

        let unresolved_inner_type = field.unresolved_inner_type(config);
        let resolved_inner_type = field.resolved_inner_type(config);

        let slot_type = if config.pick {
            quote! { schematic::MultiSlot<#unresolved_inner_type, #resolved_inner_type> }
        } else {
            quote! { schematic::SingleSlot<#unresolved_inner_type, #resolved_inner_type> }
        };

        let choice_type = if config.pick {
            quote! { schematic::choice::MultiChoice<#unresolved_inner_type> }
        } else {
            quote! { schematic::choice::SingleChoice<#unresolved_inner_type> }
        };

        let field_attrs = field
            .attrs
            .iter()
            .filter(|attr| !attr.path().is_ident("slot"))
            .collect();

        Self {
            struct_idents,
            field_name,
            vis: &field.vis,
            field_attrs,
            doc_lines: extract_doc_comments(&field.attrs),
            config,
            mode_type_name,
            trait_name,
            unresolved_inner_type,
            resolved_inner_type,
            slot_type,
            choice_type,
        }
    }
}

struct GeneratedField {
    field_tokens: TokenStream,
    slot_trait: Option<TokenStream>,
    mutator: Option<TokenStream>,
}

pub fn generate_struct(
    ast: &SchematicStruct,
    original_attrs: &[syn::Attribute],
    args: &SchematicArgs,
) -> TokenStream {
    let idents = StructIdents::new(&ast.ident);
    let mut struct_fields = Vec::new();
    let mut mutators = Vec::new();
    let mut slot_traits = Vec::new();

    if let darling::ast::Data::Struct(fields) = &ast.data {
        for field in &fields.fields {
            let generated = generate_field(field, &idents, args);
            struct_fields.push(generated.field_tokens);
            if let Some(slot_trait) = generated.slot_trait {
                slot_traits.push(slot_trait);
            }
            if let Some(mutator) = generated.mutator {
                mutators.push(mutator);
            }
        }
    }

    let builder_derives = generate_builder_derives(args, &idents.mode_trait, &mutators);
    let schematic_doc = generate_schematic_alias_doc(idents.name);
    let hkd_doc = generate_hkd_struct_doc(idents.name, &idents.schematic_type);

    let name = idents.name;
    let mode_trait = &idents.mode_trait;
    let unresolved_struct = &idents.unresolved_struct;
    let resolved_struct = &idents.resolved_struct;
    let schematic_type = &idents.schematic_type;

    quote! {
        #(#slot_traits)*

        #[doc = #schematic_doc]
        pub type #schematic_type = #name<#unresolved_struct>;

        #(#original_attrs)*
        #[doc = #hkd_doc]
        #builder_derives
        pub struct #name<M: #mode_trait = #resolved_struct> {
            #(#struct_fields)*
        }
    }
}

fn generate_field(
    field: &SchematicField,
    idents: &StructIdents,
    args: &SchematicArgs,
) -> GeneratedField {
    if let Some(config) = field.slot_config().unwrap() {
        generate_slot_field(field, &config, idents, args)
    } else {
        generate_regular_field(field, args)
    }
}

fn generate_slot_field(
    field: &SchematicField,
    config: &SlotConfig,
    idents: &StructIdents,
    args: &SchematicArgs,
) -> GeneratedField {
    let ctx = SlotFieldContext::new(field, config, idents);
    let slot_trait = generate_slot_trait(&ctx);
    let builder_attr = generate_slot_builder_attr(&ctx, args.builder);
    let mutator = if args.builder && config.collection {
        Some(generate_slot_collection_mutators(&ctx))
    } else {
        None
    };

    let field_attrs = &ctx.field_attrs;
    let vis = ctx.vis;
    let field_name = ctx.field_name;
    let mode_type_name = &ctx.mode_type_name;

    let field_tokens = quote! {
        #(#field_attrs)*
        #builder_attr
        #vis #field_name: M::#mode_type_name,
    };

    GeneratedField {
        field_tokens,
        slot_trait: Some(slot_trait),
        mutator,
    }
}

fn generate_slot_trait(ctx: &SlotFieldContext) -> TokenStream {
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

fn generate_pick_slot_trait_impls(ctx: &SlotFieldContext) -> Vec<TokenStream> {
    let trait_name = &ctx.trait_name;
    let unresolved_struct = &ctx.struct_idents.unresolved_struct;
    let resolved_struct = &ctx.struct_idents.resolved_struct;
    let slot_type = &ctx.slot_type;
    let choice_type = &ctx.choice_type;
    let unresolved_inner_type = &ctx.unresolved_inner_type;
    let resolved_inner_type = &ctx.resolved_inner_type;

    let mut trait_impls = vec![
        // 1. Raw MultiSlot
        quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #slot_type {
                fn into_slot(self) -> #slot_type {
                    self
                }
            }
        },
        // 2. MultiChoice
        quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #choice_type {
                fn into_slot(self) -> #slot_type {
                    schematic::Slot::Choose(self)
                }
            }
        },
    ];

    if ctx.config.cascade {
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for Vec<#unresolved_inner_type> {
                fn into_slot(self) -> #slot_type {
                    schematic::Slot::FixedUnresolved(self)
                }
            }
            #[doc(hidden)]
            impl<const __N: usize> #trait_name<#unresolved_struct> for [#unresolved_inner_type; __N] {
                fn into_slot(self) -> #slot_type {
                    schematic::Slot::FixedUnresolved(self.into_iter().collect())
                }
            }
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for Vec<#resolved_inner_type> {
                fn into_slot(self) -> #slot_type {
                    schematic::Slot::FixedResolved(self)
                }
            }
            #[doc(hidden)]
            impl<const __N: usize> #trait_name<#unresolved_struct> for [#resolved_inner_type; __N] {
                fn into_slot(self) -> #slot_type {
                    schematic::Slot::FixedResolved(self.into_iter().collect())
                }
            }
        });
    } else {
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl<__V: std::convert::Into<#unresolved_inner_type>> #trait_name<#unresolved_struct> for Vec<__V> {
                fn into_slot(self) -> #slot_type {
                    schematic::Slot::FixedUnresolved(self.into_iter().map(std::convert::Into::into).collect())
                }
            }
            #[doc(hidden)]
            impl<__V: std::convert::Into<#unresolved_inner_type>, const __N: usize> #trait_name<#unresolved_struct> for [__V; __N] {
                fn into_slot(self) -> #slot_type {
                    schematic::Slot::FixedUnresolved(self.into_iter().map(std::convert::Into::into).collect())
                }
            }
        });
    }

    // Resolved Struct Impl
    let return_ty_resolved = quote! { Vec<#resolved_inner_type> };
    if ctx.config.cascade {
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl #trait_name<#resolved_struct> for Vec<#resolved_inner_type> {
                fn into_slot(self) -> #return_ty_resolved {
                    self
                }
            }
            #[doc(hidden)]
            impl<const __N: usize> #trait_name<#resolved_struct> for [#resolved_inner_type; __N] {
                fn into_slot(self) -> #return_ty_resolved {
                    self.into_iter().collect()
                }
            }
        });
    } else {
        trait_impls.push(quote! {
            #[doc(hidden)]
            impl<__V: std::convert::Into<#resolved_inner_type>> #trait_name<#resolved_struct> for Vec<__V> {
                fn into_slot(self) -> #return_ty_resolved {
                    self.into_iter().map(std::convert::Into::into).collect()
                }
            }
            #[doc(hidden)]
            impl<__V: std::convert::Into<#resolved_inner_type>, const __N: usize> #trait_name<#resolved_struct> for [__V; __N] {
                fn into_slot(self) -> #return_ty_resolved {
                    self.into_iter().map(std::convert::Into::into).collect()
                }
            }
        });
    }

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
        // 1. Raw Slot object
        quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #return_ty_unresolved {
                fn into_slot(self) -> #return_ty_unresolved {
                    self
                }
            }
        },
        // 2. Unresolved Inner
        quote! {
            #[doc(hidden)]
            impl #trait_name<#unresolved_struct> for #unresolved_inner_type {
                fn into_slot(self) -> #return_ty_unresolved {
                    schematic::Slot::FixedUnresolved(self)
                }
            }
        },
    ];

    // 3. Resolved Inner (if cascade)
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

    // 4. Choice
    trait_impls.push(quote! {
        #[doc(hidden)]
        impl #trait_name<#unresolved_struct> for #choice_type {
            fn into_slot(self) -> #return_ty_unresolved {
                schematic::Slot::Choose(self)
            }
        }
    });

    // 5. Resolved Struct Impl
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

fn generate_slot_builder_attr(ctx: &SlotFieldContext, builder_enabled: bool) -> TokenStream {
    if !builder_enabled {
        return quote! {};
    }

    if ctx.config.collection {
        return quote! { #[builder(default, via_mutators)] };
    }

    let field_name = ctx.field_name;
    let trait_name = &ctx.trait_name;
    let doc_str = if !ctx.doc_lines.is_empty() {
        format_doc_with_details(
            format!("Sets `{}`.", field_name),
            &ctx.doc_lines,
            Some("Accepts an item, choice, or slot."),
        )
    } else {
        format!("Sets `{}`. Accepts an item, choice, or slot.", field_name)
    };

    quote! {
        #[builder(setter(doc = #doc_str, transform = |item: impl #trait_name<M>| {
            item.into_slot()
        }))]
    }
}

fn generate_slot_collection_mutators(ctx: &SlotFieldContext) -> TokenStream {
    let field_name = ctx.field_name;
    let trait_name = &ctx.trait_name;
    let fn_name = format_ident!("with_{}", field_name);
    let unresolved_inner_type = &ctx.unresolved_inner_type;
    let unresolved_inner_type_str = quote!(#unresolved_inner_type).to_string();

    let with_footer = format!(
        "# Accepted Values\n\
         - Raw item (`{}`)\n\
         - [`SingleChoice`](schematic::choice::SingleChoice)\n\
         - [`SingleSlot`](schematic::SingleSlot)",
        unresolved_inner_type_str
    );
    let with_fn_doc = format_doc_with_details(
        format!(
            "Appends a single item, choice, or slot to `{}`.",
            field_name
        ),
        &ctx.doc_lines,
        Some(&with_footer),
    );

    let iter_footer = format!(
        "# Accepted Values\n\
         An iterator yielding:\n\
         - Raw items (`{}`)\n\
         - [`SingleChoice`](schematic::choice::SingleChoice)\n\
         - [`SingleSlot`](schematic::SingleSlot)",
        unresolved_inner_type_str
    );
    let iter_fn_doc = format_doc_with_details(
        format!(
            "Appends an iterator of items, choices, or slots to `{}`.",
            field_name
        ),
        &ctx.doc_lines,
        Some(&iter_footer),
    );

    quote! {
        #[doc = #with_fn_doc]
        pub fn #fn_name(&mut self, item: impl #trait_name<M>) {
            self.#field_name.extend(std::iter::once(item.into_slot()));
        }

        #[doc = #iter_fn_doc]
        pub fn #field_name<I>(&mut self, items: I)
        where
            I: std::iter::IntoIterator,
            I::Item: #trait_name<M>
        {
            self.#field_name.extend(items.into_iter().map(|item| item.into_slot()));
        }
    }
}

fn generate_regular_field(field: &SchematicField, args: &SchematicArgs) -> GeneratedField {
    let field_name = field.ident.as_ref().unwrap();
    let field_attrs: Vec<_> = field
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("slot"))
        .collect();
    let doc_lines = extract_doc_comments(&field.attrs);

    if let Some(base_inner_type) = field.inner_type_if_vec() {
        generate_regular_collection_field(
            field,
            field_name,
            &field_attrs,
            &doc_lines,
            base_inner_type,
            args,
        )
    } else {
        generate_regular_scalar_field(field, field_name, &field_attrs, &doc_lines, args)
    }
}

fn generate_regular_collection_field(
    field: &SchematicField,
    field_name: &Ident,
    field_attrs: &[&syn::Attribute],
    doc_lines: &[String],
    base_inner_type: &syn::Type,
    args: &SchematicArgs,
) -> GeneratedField {
    let vis = &field.vis;
    let ty = &field.ty;

    let builder_attr = if args.builder {
        quote! { #[builder(default, via_mutators)] }
    } else {
        quote! {}
    };

    let mutator = if args.builder {
        let fn_name = format_ident!("with_{}", field_name);
        let with_non_slot_doc = format_doc_with_details(
            format!("Appends a single item to `{}`.", field_name),
            doc_lines,
            None,
        );
        Some(quote! {
            #[doc = #with_non_slot_doc]
            pub fn #fn_name(&mut self, item: impl Into<#base_inner_type>) {
                self.#field_name.push(item.into());
            }
        })
    } else {
        None
    };

    let field_tokens = quote! {
        #(#field_attrs)*
        #builder_attr
        #vis #field_name: #ty,
    };

    GeneratedField {
        field_tokens,
        slot_trait: None,
        mutator,
    }
}

fn generate_regular_scalar_field(
    field: &SchematicField,
    field_name: &Ident,
    field_attrs: &[&syn::Attribute],
    doc_lines: &[String],
    args: &SchematicArgs,
) -> GeneratedField {
    let vis = &field.vis;
    let ty = &field.ty;

    let builder_attr = if args.builder {
        let doc_setter = if !doc_lines.is_empty() {
            let doc_str =
                format_doc_with_details(format!("Sets `{}`.", field_name), doc_lines, None);
            quote! { doc = #doc_str, }
        } else {
            quote! {}
        };
        quote! { #[builder(setter(into, #doc_setter))] }
    } else {
        quote! {}
    };

    let field_tokens = quote! {
        #(#field_attrs)*
        #builder_attr
        #vis #field_name: #ty,
    };

    GeneratedField {
        field_tokens,
        slot_trait: None,
        mutator: None,
    }
}

fn generate_builder_derives(
    args: &SchematicArgs,
    mode_trait: &Ident,
    mutators: &[TokenStream],
) -> TokenStream {
    if !args.builder {
        return quote! {};
    }

    if mutators.is_empty() {
        quote! {
            #[derive(typed_builder::TypedBuilder)]
            #[builder(doc)]
        }
    } else {
        quote! {
            #[schematic::__qualify_mode(#mode_trait)]
            #[derive(typed_builder::TypedBuilder)]
            #[builder(doc)]
            #[builder(mutators(
                #(#mutators)*
            ))]
        }
    }
}

fn extract_doc_comments(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc")
                && let syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }),
                    ..
                }) = &attr.meta
            {
                return Some(s.value().trim().to_string());
            }
            None
        })
        .collect()
}

fn format_doc_with_details(summary: String, doc_lines: &[String], footer: Option<&str>) -> String {
    let mut parts = vec![summary];
    if !doc_lines.is_empty() {
        parts.push(doc_lines.join("\n"));
    }
    if let Some(footer_text) = footer {
        parts.push(footer_text.to_string());
    }
    parts.join("\n\n")
}

fn generate_schematic_alias_doc(name: &Ident) -> String {
    format!(
        "Unresolved schematic blueprint for [`{}`].\n\n\
         This type alias represents the unresolved blueprint where slot fields may contain \
         pending choices, fixed unresolved instances, or raw values.\n\n\
         Once all choices are satisfied, call [`Schematic::resolve`](schematic::Schematic::resolve) \
         to produce the watertight resolved domain model [`{}`].",
        name, name
    )
}

fn generate_hkd_struct_doc(name: &Ident, schematic_type: &Ident) -> String {
    format!(
        "\n\n# Schematic Architecture (Higher-Kinded Data)\n\n\
         This struct uses Higher-Kinded Data (HKD) parameterized by mode `M`.\n\n\
         - **Resolved mode (default)**: When unparameterized, [`{}`] represents the final, \
           watertight domain model where all choices have been resolved into concrete values.\n\
         - **Unresolved mode ([`{}`])**: The unresolved blueprint containing stateful \
           slots and choices. Blueprints can be constructed via [`{}::builder()`], inspected, \
           mutated with visitors, and resolved into [`{}`] via [`Schematic::resolve`](schematic::Schematic::resolve).\n",
        name, schematic_type, schematic_type, name
    )
}
