use super::argument::{Argument, ArgumentKind};
use std::collections::HashMap;
use super::util::{combined_errors, uppercase_first_letter};
use std::iter::FromIterator;

#[derive(Debug, PartialEq)]
pub enum MethodKind {
    Static,
    Instance,
}

#[derive(Debug)]
pub struct Method {
    pub fn_name: proc_macro2::Ident,
    pub kind: MethodKind,
    pub return_type: syn::ReturnType,
    pub arguments: Vec<Argument>,
    pub def_name: proc_macro2::TokenStream,
    pub def_signature_map: HashMap<String, Vec<proc_macro2::TokenTree>>,
}

impl Method {
    fn method_struct_fields(&self) -> Vec<syn::Field> {
        let colon = syn::token::Colon { spans: [proc_macro2::Span::call_site()] };
        self.arguments
            .iter()
            .map(|arg| {
                syn::Field {
                    attrs: Vec::new(),
                    vis: syn::Visibility::Inherited,
                    ident: Some(arg.name.ident.clone()),
                    colon_token: Some(colon),
                    ty: arg.kind.type_for_struct_field(&arg.ty),
                }
            })
            .collect()
    }

    fn method_struct_name(&self, class_name: &proc_macro2::Ident) -> proc_macro2::Ident {
        quote::format_ident!("{}{}Method", class_name, uppercase_first_letter(&self.fn_name.to_string()))
    }

    pub fn method_struct(&self, rutie_class: &proc_macro2::Ident) -> proc_macro2::TokenStream {
/*
struct RutieFooTestMethod {
    rtself: RutieFoo,
    a: Arg<RString>,
    b: DArg<RString>,
    c: KwArg<RString>,
    d: KwArg<RString>,
    e: KwArg<RString>,
}
*/
        let struct_name = self.method_struct_name(rutie_class);
        let mut punct = syn::punctuated::Punctuated::new();
        punct.push(self.method_struct_impl_field_rtself_expr(rutie_class));
        let punct = self.method_struct_fields().iter().fold(punct, |mut acc, f| {
            acc.push(f.clone());
            acc
        });
        let span = proc_macro2::Span::call_site();
        let fields_named = syn::FieldsNamed { brace_token: syn::token::Brace(span), named: punct };
        let fields = syn::Fields::Named(fields_named);
        let mut item: syn::ItemStruct = syn::parse_quote! { struct #struct_name {} };
        item.fields = fields;

        quote::quote! { #item }
    }

    fn method_struct_impl_field_rtself_expr(&self, rutie_class: &proc_macro2::Ident) -> syn::Field {
        let colon = syn::token::Colon { spans: [proc_macro2::Span::call_site()] };
        syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            ident: Some(quote::format_ident!("rtself")),
            colon_token: Some(colon),
            ty: syn::parse_quote! { #rutie_class },
        }
    }

    fn method_struct_impl_field_value_exprs(&self) -> Vec<syn::FieldValue> {
        let orders = self.arguments_order();
        let colon = syn::token::Colon { spans: [proc_macro2::Span::call_site()] };
        self.arguments
            .iter()
            .map(|arg| {
                let order = orders[&arg.name];
                syn::FieldValue {
                    attrs: Vec::new(),
                    member: syn::Member::Named(arg.name.ident.clone()),
                    colon_token: Some(colon),
                    expr: arg.kind.expr_call_for_initialize_struct_field(order, &arg.name.ident.to_string()),
                }
            })
            .collect()
    }

    fn method_struct_impl_field_value_rtself_expr(&self) -> syn::FieldValue {
        let colon = syn::token::Colon { spans: [proc_macro2::Span::call_site()] };
        syn::FieldValue {
            attrs: Vec::new(),
            member: syn::Member::Named(quote::format_ident!("rtself")),
            colon_token: Some(colon),
            expr: syn::parse_quote! { rtself },
        }
    }

    fn method_struct_impl_expr_struct(&self) -> syn::ExprStruct {
/*
Self {
    rtself: RutieFoo,
    a: Arg::from_arg(_arguments.get(0)),
    b: DArg::from_arg_with_default(_arguments.get(1), default_value_map.get("b")),
    c: KwArg::from_arg_with_key_and_default(_arguments.get(2), "c", default_value_map.get("c")),
    d: KwArg::from_arg_with_key_and_default(_arguments.get(2), "d", default_value_map.get("d")),
    e: KwArg::from_arg_with_key_and_default(_arguments.get(2), "e", default_value_map.get("e")),
}
*/
        let mut punct = syn::punctuated::Punctuated::new();
        punct.push(self.method_struct_impl_field_value_rtself_expr());
        let punct = self.method_struct_impl_field_value_exprs().iter().fold(punct, |mut acc, f| {
            acc.push(f.clone());
            acc
        });
        let mut expr: syn::ExprStruct = syn::parse_quote! { Self {} };
        expr.fields = punct;

        expr
    }

    fn method_exception_block_from_arguments(&self) -> syn::Block {
        let mut block: syn::Block = syn::parse_quote! { {} };
        for arg in self.arguments.iter() {
            let ident = &arg.name.ident;
            block.stmts.push(syn::parse_quote! {
                if let Err(e) = &self.#ident.result {
                    return Some(e);
                }
            });
        }
        block.stmts.push(syn::parse_quote! { return None; });
        block
    }

    fn method_fn_call_expr(&self) -> syn::ExprCall {
        let fn_name = &self.fn_name;
        let mut call: syn::ExprCall = syn::parse_quote! { #fn_name() };
        for arg in self.arguments.iter() {
            let arg_name = &arg.name.ident;
            let arg_name: syn::ExprMethodCall = syn::parse_quote! { self.#arg_name() };
            call.args.push(syn::Expr::from(arg_name));
        }
        call
    }

    fn method_argument_methods(&self) -> proc_macro2::TokenStream {
        let mut content = quote::quote! {};
        for arg in self.arguments.iter() {
            let ident = &arg.name.ident;
            let ty = &arg.ty;
            content = quote::quote! {
                fn #ident(&self) -> #ty {
                    self.#ident.result.as_ref().ok().unwrap().value().clone().into()
                }
            };
        }
        content
    }

    fn method_fn_call(&self, class_name: &proc_macro2::Ident, rutie_class: &proc_macro2::Ident) -> proc_macro2::TokenStream {
        let fn_call = self.method_fn_call_expr();
        let content = if self.kind == MethodKind::Instance {
            quote::quote! {
                let _self = #class_name::try_from(#rutie_class { value: self.rtself.value() });
                if let Err(e) = _self {
                    return e.to_any_object();
                }
                let result = _self.unwrap().#fn_call;
            }
        } else {
            quote::quote! {
                let result = #class_name::#fn_call;
            }
        };

        quote::quote! {
            #content
            result.to_any_object()
        }
    }

    pub fn method_struct_impl(&self, class_name: &proc_macro2::Ident, rutie_class: &proc_macro2::Ident) -> proc_macro2::TokenStream {
        let struct_name = self.method_struct_name(rutie_class);
        let fn_call = self.method_fn_call(class_name, rutie_class);
        let expr_struct = self.method_struct_impl_expr_struct();
        let exception_block = self.method_exception_block_from_arguments();
        let methods = self.method_argument_methods();
        quote::quote! {
            impl #struct_name {
                pub fn new(argc: rutie::types::Argc, argv: *const rutie::AnyObject, rtself: #rutie_class, default_value_map: &std::collections::HashMap<&str, rutie::AnyObject>) -> Self {
                    let _arguments = rutie::util::parse_arguments(argc, argv);
                    #expr_struct
                }

                fn exception(&self) -> Option<&rutie::AnyException> #exception_block

                #methods

                pub fn invoke(&self) -> rutie::AnyObject {
                    if let Some(e) = self.exception() {
                        return e.to_any_object();
                    }
                    #fn_call
                }
            }
        }
    }

    pub fn fn_call(&self, class_name: &proc_macro2::Ident) -> proc_macro2::TokenStream {
        let struct_name = self.method_struct_name(class_name);
        let mut content = quote::quote! {
            let mut default_value_map = std::collections::HashMap::new();
        };
        for arg in self.arguments.iter() {
            let arg_name = &arg.name.ident.to_string();
            if let Some(default_value) = &arg.default_value {
                let default_value = default_value.to_default_value(&arg.ty);
                content = quote::quote! {
                    #content
                    default_value_map.insert(#arg_name, #default_value.to_any_object());
                };
            }
        }
        quote::quote! {
            #content
            #struct_name::new(argc, argv, _rtself, &default_value_map).invoke()
        }
    }

    fn arguments_order(&self) -> HashMap<syn::PatIdent, usize> {
        let mut i = 0;
        let mut h = HashMap::new();
        for arg in self.arguments.iter() {
            h.insert(arg.name.clone(), i);
            if let ArgumentKind::KwArg = arg.kind {
            } else {
                i += 1;
            }
        }
        h
    }

    pub fn def_name(&self) -> proc_macro2::Literal {
        let s = if self.def_name.is_empty() {
            self.fn_name.to_string()
        } else {
            self.def_name
                .clone()
                .into_iter()
                .fold(String::from(""), |mut acc, tree| {
                    match tree {
                        proc_macro2::TokenTree::Ident(ident) => {
                            acc.push_str(&ident.to_string());
                            acc
                        },
                        proc_macro2::TokenTree::Punct(punct) => {
                            acc.push_str(&punct.as_char().to_string());
                            acc
                        },
                        _ => unreachable!("exists other TokenTree at def_name"),
                    }
                })
        };
        proc_macro2::Literal::string(&s)
    }

    fn validate_def_name(&self) -> syn::Result<()> {
        let mut names = self.def_name.clone().into_iter();

        let message = "The def_name must be an alphabetic or underscore ident and end with ! or ? symbol at the end.";

        // def_nameの最初のTokenTreeはIdentであること
        let first = names.next();
        if let Some(proc_macro2::TokenTree::Ident(_)) = first {
            // noop
        } else {
            return Err(syn::Error::new_spanned(first, message));
        }

        // def_nameの２つめのTokenTreeが存在する場合は、!か?であること。
        let second = names.next();
        if let Some(proc_macro2::TokenTree::Punct(p)) = second {
            let c = p.as_char();
            if c != '!' && c != '?' {
                return Err(syn::Error::new_spanned(p, "The end of the def_name is ! or ? punctuations can be specified."));
            }
        } else {
            return Err(syn::Error::new_spanned(second, message));
        }

        if let Some(third) = names.next() {
            // 3つ目はありえない
            return Err(syn::Error::new_spanned(third, message));
        }

        Ok(())
    }

    fn validate_def_signature(&self) -> syn::Result<()> {
        let arg_names = self.arguments
            .iter()
            .map(|arg| arg.name.ident.to_string())
            .collect::<Vec<String>>();

        // rbdef attributeで定義している引数名が、実装の引数に存在しないときはエラー
        let errors = self.def_signature_map
            .keys()
            .filter_map(|ident| {
                if !arg_names.contains(&ident) {
                    let t = proc_macro2::TokenStream::from_iter(self.def_signature_map[ident].clone());
                    Some(syn::Error::new_spanned(t, "not found in arguments."))
                } else {
                    None
                }
            })
            .collect::<Vec<syn::Error>>();

        if let Some(e) = combined_errors(errors) {
            Err(e)
        } else {
            Ok(())
        }

        // TODO: default_valueの値がty型にコンバートできなかったら、コンパイルエラーにする
        // コンパイル時に評価するいい方法が思いつかないので、一旦実行時にRuntimeErrorにする
    }

    pub fn validate(&self) -> syn::Result<()> {
        let mut errors = [
            self.validate_def_name(),
            self.validate_def_signature(),
        ].iter()
            .filter_map(|e| e.clone().err())
            .collect::<Vec<syn::Error>>();

        let arguments_errors = self.arguments
            .iter()
            .filter_map(|argument| argument.validate().err())
            .collect::<Vec<syn::Error>>();

        errors.extend(arguments_errors);
        if let Some(e) = combined_errors(errors) {
            Err(e)
        } else {
            Ok(())
        }
    }
}
