use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

mod ast;
mod gen_mode;
mod gen_struct;
mod gen_blueprint;
mod gen_visitor;

#[proc_macro_attribute]
pub fn blueprint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    
    let ast_struct = match ast::BlueprintStruct::parse(item_struct) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };

    let modes = gen_mode::generate_modes(&ast_struct);
    let st = gen_struct::generate_struct(&ast_struct);
    let bp = gen_blueprint::generate_blueprint(&ast_struct);
    let vis = gen_visitor::generate_visitor(&ast_struct);

    let expanded = quote::quote! {
        #modes
        #st
        #bp
        #vis
    };

    expanded.into()
}
