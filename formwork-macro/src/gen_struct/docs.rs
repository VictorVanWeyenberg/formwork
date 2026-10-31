use syn::Ident;

pub(crate) fn extract_doc_comments(attrs: &[syn::Attribute]) -> Vec<String> {
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

pub(crate) fn format_accepted_values_footer(inner_ty_str: &str, is_iterator: bool) -> String {
    let prefix = if is_iterator {
        "An iterator yielding:\n"
    } else {
        ""
    };
    let item_label = if is_iterator { "Raw items" } else { "Raw item" };
    format!(
        "# Accepted Values\n\
         {}- {} (`{}`)\n\
         - [`SingleChoice`](formwork::choice::SingleChoice)\n\
         - [`SingleSlot`](formwork::SingleSlot)",
        prefix, item_label, inner_ty_str
    )
}

pub(crate) fn format_doc_comment(
    summary: &str,
    doc_lines: &[String],
    footer: Option<&str>,
) -> String {
    let mut parts = vec![summary.to_string()];
    if !doc_lines.is_empty() {
        parts.push(doc_lines.join("\n"));
    }
    if let Some(footer_text) = footer {
        parts.push(footer_text.to_string());
    }
    parts.join("\n\n")
}

pub(crate) fn generate_formwork_alias_doc(name: &Ident) -> String {
    format!(
        "Wet formwork representation for [`{}`].\n\n\
         This type alias represents the wet formwork frame where slot fields may contain \
         pending choices, fixed wet instances, or raw values.\n\n\
         Once all choices are satisfied, call [`Formwork::resolve`](formwork::Formwork::resolve) \
         to produce the watertight dry domain model [`{}`].",
        name, name
    )
}

pub(crate) fn generate_hkd_struct_doc(name: &Ident, formwork_type: &Ident) -> String {
    format!(
        "\n\n# Formwork Architecture (Higher-Kinded Data)\n\n\
         This struct uses Higher-Kinded Data (HKD) parameterized by mode `M`.\n\n\
         - **Dry mode (default)**: When unparameterized, [`{}`] represents the final, \
           watertight dry domain model where all choices have been resolved into concrete values.\n\
         - **Wet mode ([`{}`])**: The wet formwork mold containing stateful \
           slots and choices. Formworks can be constructed via [`{}::builder()`], inspected, \
           mutated with visitors, and resolved into [`{}`] via [`Formwork::resolve`](formwork::Formwork::resolve).\n",
        name, formwork_type, formwork_type, name
    )
}
