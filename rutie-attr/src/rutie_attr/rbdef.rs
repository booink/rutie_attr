use super::argument::{Argument, ArgumentKind, ArgumentDefaultValue};
use super::method::{Method, MethodKind};
use std::collections::HashMap;
use std::iter::FromIterator;

pub struct Rbdef {
    item: syn::ImplItemMethod,
    attr: syn::Attribute,
}

impl Rbdef {
    pub fn new(item: syn::ImplItemMethod, attr: syn::Attribute) -> Self {
        Self { item, attr }
    }

    pub fn method_info(&self) -> Method {
        // attribute名が rbdef だったらrutie::methods!で定義して、externする
        let fn_name = self.parse_fn_name();
        let (def_name, def_signature) = self.parse_attribute();
        let def_signature_map = self.def_signature_map(def_signature);

        Method {
            def_name,
            fn_name,
            kind: self.parse_method_kind(),
            return_type: self.parse_return_type(),
            arguments: self.parse_arguments(&def_signature_map),
            def_signature_map,
        }
    }

    // impl内に定義されているメソッド名を取得する
    fn parse_fn_name(&self) -> proc_macro2::Ident {
        self.item.sig.ident.clone()
    }

    // impl内に定義されているメソッドのreturn型を取得する
    fn parse_return_type(&self) -> syn::ReturnType {
        self.item.sig.output.clone()
    }

    // 関数のattributeからRuby側の関数名と関数のデフォルト引数をTokenStreamで取得する
    fn parse_attribute(&self) -> (proc_macro2::TokenStream, proc_macro2::Group) {
        let empty_token_stream = proc_macro2::TokenStream::new();
        let empty_def_signature = proc_macro2::Group::new(
            proc_macro2::Delimiter::Parenthesis,
            empty_token_stream.clone(),
        );
        if self.attr.tokens.is_empty() {
            return (empty_token_stream, empty_def_signature);
        }

        if let proc_macro2::TokenTree::Group(outer_group) =
            self.attr.tokens.clone().into_iter().next().unwrap()
        {
            let (def_signature, def_name): (
                Vec<proc_macro2::TokenTree>,
                Vec<proc_macro2::TokenTree>,
            ) = outer_group.stream().into_iter().partition(|tree| {
                matches!(tree, proc_macro2::TokenTree::Group(_))
            });
            let def_name = proc_macro2::TokenStream::from_iter(def_name);
            let def_signature = def_signature.into_iter().find_map(|tree| {
                if let proc_macro2::TokenTree::Group(g) = tree {
                    Some(g)
                } else {
                    None
                }
            });
            (
                def_name,
                def_signature.unwrap_or(empty_def_signature),
            )
        } else {
            unreachable!("exists other TokenTree at rbdef attribute");
        }
    }

    fn parse_method_kind(&self) -> MethodKind {
        self.item
            .sig
            .inputs
            .iter()
            .find_map(|input| {
                if let syn::FnArg::Receiver(_) = input {
                    Some(MethodKind::Instance)
                } else {
                    None
                }
            })
            .unwrap_or(MethodKind::Static)
    }

    // メソッドの引数をいい感じにする
    fn parse_arguments(&self, def_signature_map: &HashMap<String, Vec<proc_macro2::TokenTree>>) -> Vec<Argument> {
        self.item
            .sig
            .inputs
            .iter()
            .filter_map(|input| {
                if let syn::FnArg::Typed(pat_type) = input {
                    Some(pat_type)
                } else {
                    // Receiverは無視する
                    // unreachable!()
                    None
                }
            })
            .filter_map(|pat_type| {
                if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    let (kind, default_value) = self.arg_type_and_default_value(def_signature_map, &pat_ident.ident);
                    //let ty = ArgumentType::from(*pat_type.ty.clone());
                    let ty = *pat_type.ty.clone();
                    Some(Argument {
                        name: pat_ident,
                        ty,
                        kind,
                        default_value: default_value.map(ArgumentDefaultValue::from),
                    })
                } else {
                    unreachable!("exists other PatType of PatIdent")
                }
            })
            .collect()
    }

    fn arg_type_and_default_value(&self, def_signature_map: &HashMap<String, Vec<proc_macro2::TokenTree>>, ident: &proc_macro2::Ident) -> (ArgumentKind, Option<ArgumentDefaultValue>) {
        if let Some(tokens) = def_signature_map.get(&ident.to_string()) {
            if tokens.is_empty() {
                return (ArgumentKind::Arg, None);
            }
            let mut tokens = tokens.clone();
            if let proc_macro2::TokenTree::Punct(p) = tokens.remove(0) {
                let c = p.as_char();
                let t = proc_macro2::TokenStream::from_iter(tokens.to_vec());
                let default_value = Some(ArgumentDefaultValue::from(t));
                if c == '=' {
                    return (ArgumentKind::DArg, default_value);
                } else if c == ':' {
                    return (ArgumentKind::KwArg, default_value);
                }
            }
        }
        unreachable!("def_signature_map has not ident.")
    }

    fn def_signature_map(
        &self,
        group: proc_macro2::Group,
    ) -> HashMap<String, Vec<proc_macro2::TokenTree>> {
        group
            .stream()
            .into_iter()
            .collect::<Vec<proc_macro2::TokenTree>>()
            .split(Self::is_comma)
            .into_iter()
            .filter_map(|v| {
                let mut sig = v.to_vec();
                if let proc_macro2::TokenTree::Ident(ident) = sig.remove(0) {
                    Some((ident.to_string(), sig))
                } else {
                    //unreachable!("The first of def_signature is not Ident at def_signature_map.")
                    None
                }
            })
            .into_iter()
            .collect()
    }

    fn is_comma(token: &proc_macro2::TokenTree) -> bool {
        if let proc_macro2::TokenTree::Punct(p) = token {
            if p.as_char() == ',' {
                return true;
            }
        }
        false
    }
}
