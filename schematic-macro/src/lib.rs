use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

mod ast;
mod gen_mode;
mod gen_struct;
mod gen_schematic;
mod gen_visitor;

#[proc_macro_attribute]
pub fn schematic(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    
    let mut ast_struct = match ast::SchematicStruct::parse(item_struct) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };

    if !attr.is_empty() {
        let attr_str = attr.to_string();
        if attr_str.contains("builder") {
            ast_struct.generate_builder = true;
        } else {
            return syn::Error::new_spanned(proc_macro2::TokenStream::from(attr), "unsupported schematic attribute").to_compile_error().into();
        }
    }

    let modes = gen_mode::generate_modes(&ast_struct);
    let st = gen_struct::generate_struct(&ast_struct);
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
