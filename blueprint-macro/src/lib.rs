use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

mod core;

#[proc_macro_attribute]
pub fn blueprint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    match core::generate_blueprint(item_struct) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
