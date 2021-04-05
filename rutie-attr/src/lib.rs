mod rutie_attr;
use rutie_attr::rbclass::Rbclass;
use rutie_attr::rbmethods::Rbmethods;

use proc_macro::TokenStream;
use std::convert::From;
use syn::{parse_macro_input, ItemFn, ItemImpl, ItemStruct};

#[proc_macro_attribute]
pub fn rbclass(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    Rbclass::new(item).token_stream()
}

#[proc_macro_attribute]
pub fn rbmethods(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemImpl);
    Rbmethods::new(item).token_stream()
}

/// def hoge()
#[proc_macro_attribute]
pub fn rbdef(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let gen = item.clone();
    let _ast = parse_macro_input!(item as ItemFn);
    gen
}

/// def self.hoge()
#[proc_macro_attribute]
pub fn rbdefself(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let gen = item.clone();
    let _ast = parse_macro_input!(item as ItemFn);
    gen
}
