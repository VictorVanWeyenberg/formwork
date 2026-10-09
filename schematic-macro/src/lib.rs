use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use darling::{FromMeta, ast::NestedMeta};

mod ast;
mod gen_mode;
mod gen_struct;
mod gen_schematic;
mod gen_visitor;

#[proc_macro_attribute]
pub fn schematic(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as DeriveInput);
    
    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };

    let mut args = match ast::SchematicArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    for attr in &item_struct.attrs {
        if attr.path().is_ident("derive") {
            if let Ok(meta) = attr.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated) {
                for path in meta {
                    let path_str = quote::quote!(#path).to_string();
                    if !args.derives.iter().any(|p| quote::quote!(#p).to_string() == path_str) {
                        args.derives.push(path);
                    }
                }
            }
        }
    }

    let ast_struct = match ast::SchematicStruct::parse_input(&item_struct) {
        Ok(ast) => ast,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    // Validate fields
    if let darling::ast::Data::Struct(fields) = &ast_struct.data {
        for field in &fields.fields {
            if let Err(e) = field.slot_config() {
                return TokenStream::from(e.write_errors());
            }
        }
    }

    let original_attrs = &item_struct.attrs;

    let modes = gen_mode::generate_modes(&ast_struct, &args);
    let st = gen_struct::generate_struct(&ast_struct, original_attrs, &args);
    let bp = gen_schematic::generate_schematic(&ast_struct);
    let vis = gen_visitor::generate_visitor(&ast_struct);

    let expanded = quote::quote! {
        #modes
        #st
        #bp
        #vis
    };

    expanded.into()
}
