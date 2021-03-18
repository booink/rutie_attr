use proc_macro2::{Ident, Span};
use proc_macro::TokenStream;
use quote::quote;
use std::convert::From;
use syn::{parse_macro_input, ItemFn, ItemStruct, ItemImpl};

#[proc_macro_derive(RbClass)]
pub fn derive_rb_class(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = item.ident;
    let name = struct_name.to_string();
    let rutie_name = format!("Rutie/{}", name);
    let rutie_class_name: &str = &format!("Rutie{}", &name);
    let rutie_class = Ident::new(rutie_class_name, Span::call_site());
    let error_message = format!("Error converting to {}", rutie_class_name);

    let wrapped_class_name: &str = &format!("{}Wrapper", &name);
    let wrapped_class = Ident::new(wrapped_class_name, Span::call_site());

    let lazy_wrapped_class_name: &str = &format!("{}LazyWrapper", &name);
    let lazy_wrapped_class = Ident::new(lazy_wrapped_class_name, Span::call_site());

    let lazy_wrapped_static_name: &str = &format!("{}_LAZY_WRAPPER", &name.to_ascii_uppercase());
    let lazy_wrapped_static = Ident::new(lazy_wrapped_static_name, Span::call_site());
    let gen = quote! {
        #[repr(C)]
        pub struct #rutie_class {
            value: rutie::types::Value,
        }

        impl From<rutie::types::Value> for #rutie_class {
            fn from(value: rutie::types::Value) -> Self {
                Self { value }
            }
        }

        impl rutie::Object for #rutie_class {
            #[inline]
            fn value(&self) -> rutie::types::Value {
                self.value
            }
        }

        use rutie::Class;
        impl rutie::VerifiedObject for #rutie_class {
            fn is_correct_type<T: rutie::Object>(object: &T) -> bool {
                Class::from_existing(#rutie_class_name).case_equals(object)
            }

            fn error_message() -> &'static str {
                #error_message
            }
        }

        pub struct #wrapped_class<T> {
            data_type: ::rutie::types::DataType,
            _marker: ::std::marker::PhantomData<T>,
        }

        pub struct #lazy_wrapped_class {
            __private_field: (),
        }

        #[doc(hidden)]
        pub static #lazy_wrapped_static: #lazy_wrapped_class = #lazy_wrapped_class{__private_field: ()};
        impl ::lazy_static::__Deref for #lazy_wrapped_class {
            type Target = #wrapped_class<#struct_name>;
            fn deref(&self) -> &#wrapped_class<#struct_name> {
                #[inline(always)]
                fn __static_ref_initialize() -> #wrapped_class<#struct_name> {
                    #wrapped_class::new()
                }
                #[inline(always)]
                fn __stability() -> &'static #wrapped_class<#struct_name> {
                    static LAZY: ::lazy_static::lazy::Lazy<#wrapped_class<#struct_name>> = ::lazy_static::lazy::Lazy::INIT;
                    LAZY.get(__static_ref_initialize)
                }
                __stability()
            }
        }

        impl ::lazy_static::LazyStatic for #lazy_wrapped_class {
            fn initialize(lazy: &Self) { let _ = &**lazy; }
        }

        impl <T> #wrapped_class<T> {
            fn new() -> #wrapped_class<T> {
                let name = #rutie_name;
                let name = ::rutie::util::str_to_cstring(name);
                let reserved_bytes: [*mut ::rutie::types::c_void; 2] = [::std::ptr::null_mut(); 2];
                let dmark = None as Option<extern "C" fn(*mut ::rutie::types::c_void)>;
                let data_type = ::rutie::types::DataType {
                    wrap_struct_name: name.into_raw(),
                    parent: ::std::ptr::null(),
                    data: ::std::ptr::null_mut(),
                    flags: ::rutie::types::Value::from(0),
                    function: ::rutie::types::DataTypeFunction{
                        dmark: dmark,
                        dfree: Some(::rutie::typed_data::free::<T>),
                        dsize: None,
                        reserved: reserved_bytes
                    }
                };
                #wrapped_class { data_type: data_type, _marker: ::std::marker::PhantomData }
            }
        }

        unsafe impl <T> Sync for #wrapped_class<T> { }
        impl <T> ::rutie::typed_data::DataTypeWrapper<T> for #wrapped_class<T> {
            fn data_type(&self) -> &::rutie::types::DataType { &self.data_type }
        }
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn rbclass(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let ast = item.clone();
    let struct_name = item.ident;
    let name = struct_name.to_string();
    let rutie_name = format!("Rutie/{}", name);
    let rutie_class_name: &str = &format!("Rutie{}", &name);
    let rutie_class = Ident::new(rutie_class_name, Span::call_site());
    let gen = quote! {
        #ast
        rutie::class!(#rutie_class);
    };
    gen.into()
}

#[derive(Debug)]
struct MethodInfo {
    pub def_name: String,
    pub new_fn_name: String,
    pub method_without_attr: proc_macro2::TokenStream, // パースが終わったらattributesをclearしてTokenStreamにする
}

fn parse_rbdef(item: &syn::ImplItemMethod, struct_name: &String) -> Option<MethodInfo> {
    use quote::ToTokens;
    // メソッドの場合
    for attr in item.attrs.iter() {
        // attributeを取得
        let attr_name = attr.path.segments.first().unwrap().ident.clone();
        if attr_name == "rbdef" {
            // attribute名が rbdef だったらrutie::methods!で定義して、externする
            let fn_name = parse_rbdef_fn_name(&item.sig);
            let def_name = if let Some(n) = parse_rbdef_def_name(attr) {
                n
            } else {
                fn_name.clone()
            };

            let new_fn_name = extern_impl_fn_name(struct_name, fn_name);

            let mut item = item.clone();

            // 関数の可視性をはずす
            // pub fn foo()
            // to
            // fn foo()
            item.vis = syn::Visibility::Inherited;

            // 関数名を new_fn_name にする
            item.sig.ident = Ident::new(&new_fn_name, Span::call_site());

            // 関数のattributeを削除する
            item.attrs.clear();

            let mut token_stream = proc_macro2::TokenStream::new();
            item.to_tokens(&mut token_stream);

            return Some(MethodInfo {
                def_name,
                new_fn_name,
                method_without_attr: token_stream,
            });
        }
    }
    None
}

// impl内に定義されているメソッドのattributeから、Rubyで使用する際のメソッド名を取得する
fn parse_rbdef_def_name(attr: &syn::Attribute) -> Option<String> {
    if !attr.tokens.is_empty() {
        for tree in attr.tokens.clone() {
            if let proc_macro2::TokenTree::Group(group) = tree {
                for kind in group.stream() {
                    if let proc_macro2::TokenTree::Literal(l) = kind {
                         return Some(l.to_string().trim_matches('"').to_string());
                    }
                }
            }
        }
    }
    None
}

// impl内に定義されているメソッド名を取得する
fn parse_rbdef_fn_name(sig: &syn::Signature) -> String {
    sig.ident.to_string()
}

fn extern_impl_fn_name(struct_name: &String, fn_name: String) -> String {
    format!("rutie_{}_{}", struct_name, fn_name)
}

#[proc_macro_attribute]
pub fn rbmethods(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
    let mut gen = token_stream.clone();
    let ast = parse_macro_input!(token_stream as ItemImpl);
    let item_impl = ast.clone();
    if let syn::Type::Path(syn::TypePath { path: ref path, .. }) = *item_impl.self_ty {
        // "impl Foo" の "Foo" の部分
        let struct_name = path.segments.first().unwrap().ident.clone();
        let name = struct_name.to_string();
        let rutie_name = format!("Rutie/{}", name);
        let rutie_class_name: &str = &format!("Rutie{}", &name);
        let rutie_class = Ident::new(rutie_class_name, Span::call_site());

        let extern_fn_name: &str = &format!("init_{}", &name.to_ascii_lowercase());
        let extern_fn = Ident::new(extern_fn_name, Span::call_site());

        let mut methods: Vec<MethodInfo> = Vec::new();
        for item in item_impl.items.iter() {
            match item {
                syn::ImplItem::Method(impl_item_method) => {
                    if let Some(m) = parse_rbdef(impl_item_method, &name) {
                        methods.push(m);
                    }
                },
                _ => {},
            }
        }


        /* methods! の中身 */
        let mut body = quote! {};
        for m in methods.iter() {
            let method_without_attr = &m.method_without_attr;
            // bodyに関数の定義を詰め込んでいく
            body = quote! {
                #body

                #method_without_attr
            };
        }
        let inner_methods = quote! {
            rutie::methods!(
                #rutie_class,
                rtself,

                #body
            );
        };
        /* ここまで methods! の中身 */


        /* FFIで外出しする関数の記述 */
        //let extern_token_stream = methods_to_extern_fn_token_stream(methods, extern_fn, rutie_class_name);
        let block = quote! { {} }.into();
        let mut block = parse_macro_input!(block as syn::Block);

        for m in methods.iter() {
            let def_name = &m.def_name;
            let fn_name = Ident::new(&m.new_fn_name, Span::call_site());
            let token = quote! {
                klass.def(stringify!(#def_name), #fn_name);
            }.into();
            block.stmts.push(parse_macro_input!(token as syn::Stmt));
        }

        let extern_token_stream = quote! {
            #[no_mangle]
            pub extern "C" fn #extern_fn() {
                rutie::Class::new(#rutie_class_name, None).define(|klass|
                    #block
                );
            }
        };
        /* ここまで FFIで外出しする関数の記述 */

        gen = quote! {
            #item_impl

            #inner_methods

            #extern_token_stream
        }.into();
    }
    gen
}

/// def hoge()
#[proc_macro_attribute]
pub fn rbdef(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut gen = item.clone();
    let ast = parse_macro_input!(item as ItemFn);
    gen
}

/// def self.hoge()
#[proc_macro_attribute]
pub fn rbdefself(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut gen = item.clone();
    let ast = parse_macro_input!(item as ItemFn);
    gen
}
