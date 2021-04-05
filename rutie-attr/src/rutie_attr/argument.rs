#[derive(Debug)]
pub struct Argument {
    pub name: syn::PatIdent,
    pub ty: syn::Type,
    pub default_value: Option<ArgumentDefaultValue>,
    pub kind: ArgumentKind,
}

impl Argument {
    pub fn validate(&self) -> syn::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum ArgumentDefaultValue {
    Nil,
    Boolean(proc_macro2::Ident),
    StringLiteral(proc_macro2::Literal),
    NumberLiteral(proc_macro2::Literal),
}

impl From<proc_macro2::TokenStream> for ArgumentDefaultValue {
    fn from(tokens: proc_macro2::TokenStream) -> ArgumentDefaultValue {
        let s = tokens.into_iter().fold(String::new(), |mut acc, tree| {
            let s = match tree {
                proc_macro2::TokenTree::Ident(ident) => ident.to_string(),
                proc_macro2::TokenTree::Literal(literal) => literal.to_string(),
                proc_macro2::TokenTree::Punct(punct) => punct.to_string(),
                _ => String::new(),
            };
            acc.push_str(&s);
            acc
        });
        match s.as_str() {
            "nil" => Self::Nil,
            "true" => Self::Boolean(quote::format_ident!("true")),
            "false" => Self::Boolean(quote::format_ident!("false")),
            _ => {
                let l = s.to_string();
                let trimed = l.trim_matches('"').to_string();
                // 文字列の両端がダブルクオーテーションで囲まれていたらStringLiteral,
                // 囲まれていなかったら数値リテラル
                if l == trimed {
                    if let Ok(n) = l.parse::<i128>() {
                        Self::NumberLiteral(proc_macro2::Literal::i128_suffixed(n))
                    } else {
                        Self::StringLiteral(proc_macro2::Literal::string(&l))
                    }
                } else {
                    Self::StringLiteral(proc_macro2::Literal::string(&trimed))
                }
            },
        }
    }
}

impl ArgumentDefaultValue {
    pub fn to_default_value(&self, ty: &syn::Type) -> proc_macro2::TokenStream {
        match self {
            Self::Nil => quote::quote! { None },
            Self::Boolean(b) => {
                let b = quote::format_ident!("{}", b);
                quote::quote! { Boolean::new(#b) }
            },
            Self::StringLiteral(s) => quote::quote! { RString::from(#s) },
            Self::NumberLiteral(n) => quote::quote! { #ty::new(#n) },
        }
    }
}

#[derive(Debug)]
pub enum ArgumentKind {
    Arg,
    DArg,
    KwArg,
}

impl ArgumentKind {
    pub fn type_for_struct_field(&self, ty: &syn::Type) -> syn::Type {
        match &self {
            Self::Arg => syn::parse_quote! { rutie_attr_backend::Arg<#ty> },
            Self::DArg => syn::parse_quote! { rutie_attr_backend::DArg<#ty> },
            Self::KwArg => syn::parse_quote! { rutie_attr_backend::KwArg<#ty> },
        }
    }

    pub fn expr_call_for_initialize_struct_field(&self, index: usize, field_name: &str) -> syn::Expr {
        // a: Arg::from_arg(_arguments.get(0)),
        //    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        match &self {
            Self::Arg => syn::parse_quote! { rutie_attr_backend::Arg::from_arg(_arguments.get(#index)) },
            Self::DArg => syn::parse_quote! { rutie_attr_backend::DArg::from_arg_with_default(_arguments.get(#index), default_value_map.get(#field_name)) },
            Self::KwArg => syn::parse_quote! { rutie_attr_backend::KwArg::from_arg_with_key_and_default(_arguments.get(#index), #field_name, def_signature_map.get(#field_name)) },
        }
    }
}
