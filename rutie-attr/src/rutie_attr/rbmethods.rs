use super::rbdef::Rbdef;
use super::method::{Method, MethodKind};
use super::DEFINED_CLASSES;

pub struct Rbmethods {
    item: syn::ItemImpl,
    class_name: proc_macro2::Ident,
}

impl Rbmethods {
    pub fn new(item: syn::ItemImpl) -> Self {
        Self {
            item: item.clone(),
            class_name: Self::class_name(item),
        }
    }

    fn class_name(item: syn::ItemImpl) -> proc_macro2::Ident {
        // "impl Foo" の "Foo" の部分
        if let syn::Type::Path(p) = *item.self_ty {
            p.path.get_ident().unwrap().clone()
        } else {
            unreachable!("exists other Type at Rbmethods::class_name")
        }
    }

    pub fn token_stream(&self) -> proc_macro::TokenStream {
        let rutie_class = quote::format_ident!("Rutie{}", &self.class_name);

        let methods = self.parse_rbdefs();

        // validationのうちどれか一つでも syn::Error なら to_compile_error()
        if let Err(e) = self.validate(&methods) {
            return e.to_compile_error().into();
        }

        let mut extern_fns = quote::quote! {};
        for extern_fn in self.extern_fns(&rutie_class, &methods).into_iter() {
            extern_fns = quote::quote! {
                #extern_fns
                #extern_fn
            };
        }

        let extern_class = self.extern_class(&methods);

        let method_structs_and_impls = self.method_structs_and_impls(&self.class_name, &rutie_class, &methods);

        let item_impl = self.item.clone();
        let gen = quote::quote! {
            #item_impl

            use rutie_attr_backend::FromArgWithDefault;
            #method_structs_and_impls

            #extern_fns

            #extern_class
        };
        gen.into()
    }

    fn validate(&self, methods: &[Method]) -> syn::Result<()> {
        let errors = methods
            .iter()
            .filter_map(|method| method.validate().err())
            .collect::<Vec<syn::Error>>();

        let mut errors = errors.iter();
        if let Some(e) = errors.next() {
            Err(errors.fold(e.clone(), |mut acc, o| {
                acc.combine(o.clone());
                acc
            }))
        } else {
            // エラー無し
            Ok(())
        }
    }

    fn class_attributes(&self) -> Vec<String> {
        DEFINED_CLASSES
            .lock()
            .unwrap()
            .get(&self.class_name.to_string())
            .unwrap()
            .to_vec()
    }

    fn method_structs_and_impls(&self, class_name: &syn::Ident, rutie_class: &syn::Ident, methods: &[Method]) -> proc_macro2::TokenStream {
        let mut m = proc_macro2::TokenStream::new();
        for method in methods.iter() {
            let s = method.method_struct(rutie_class);
            let i = method.method_struct_impl(class_name, rutie_class);
            //dbg!(&s.to_string());
            //dbg!(&i.to_string());
            m = quote::quote! {
                #m
                #s
                #i
            };
        }
        m
    }

    fn extern_fns(&self, rutie_class: &syn::Ident, methods: &[Method]) -> Vec<syn::ItemFn> {
        /* methods! の中身 */
        let mut imethods = Vec::new();
        for m in methods.iter() {
            let new_fn_name = self.extern_impl_fn_name(&m.fn_name);
            //let content = &m.fn_call(&self.class_name);
            let content = &m.fn_call(rutie_class);
            // bodyに関数の定義を詰め込んでいく
            let item_fn: syn::ItemFn = syn::parse_quote! {
                #[allow(unused_mut)]
                #[allow(non_snake_case)]
                pub extern fn #new_fn_name(
                    argc: rutie::types::Argc,
                    argv: *const rutie::AnyObject,
                    mut _rtself: #rutie_class
                ) -> rutie::AnyObject {
                    #content
                }
            };
            imethods.push(item_fn);
        }
        imethods
    }

    fn extern_class(&self, methods: &[Method]) -> syn::ItemFn {
        /* FFIで外出しする関数の記述 */
        let mut block: syn::Block = syn::parse_quote! { {} };

        for n in self.class_attributes().iter() {
            let stmt: syn::Stmt = syn::parse_quote! {
                klass.attr_accessor(#n);
            };
            block.stmts.push(stmt);
        }

        for m in methods.iter() {
            let def_name = &m.def_name();
            let new_fn_name = self.extern_impl_fn_name(&m.fn_name);
            let stmt: syn::Stmt = match m.kind {
                MethodKind::Instance => syn::parse_quote! {
                    klass.def(#def_name, #new_fn_name);
                },
                MethodKind::Static => syn::parse_quote! {
                    klass.def_self(#def_name, #new_fn_name);
                },
            };
            block.stmts.push(stmt);
        }

        //let rutie_class_name = format!("Rutie{}", &self.class_name);
        let rutie_class_name = &self.class_name.to_string();

        let extern_fn_name = quote::format_ident!("Init_{}", &self.class_name);
        syn::parse_quote! {
            #[no_mangle]
            pub extern "C" fn #extern_fn_name() {
                rutie::Class::new(#rutie_class_name, None).define(|klass|
                    #block
                );
            }
        }
    }

    // impl内の各メソッドをパースしてMethodのVecを作る
    fn parse_rbdefs(&self) -> Vec<Method> {
        self.item
            .items
            .iter()
            .filter_map(|item| {
                if let syn::ImplItem::Method(m) = item {
                    self.parse_rbdef(m)
                } else {
                    unreachable!("exists other ImplItem at Rbmethods.parse_rbdefs")
                }
            })
            .collect()
    }

    // メソッドをパースしてMethodを作る
    fn parse_rbdef(&self, item: &syn::ImplItemMethod) -> Option<Method> {
        item
            .attrs
            .iter()
            .find(|attr| attr.path.get_ident().unwrap() == "rbdef" ) // rbdefアトリビュートが設定されているメソッドのみを対象にする
            .map(|attr| Rbdef::new(item.clone(), attr.clone()).method_info())
    }

    // Rubyで読み込む際に渡すシンボル名
    fn extern_impl_fn_name(&self, fn_name: &proc_macro2::Ident) -> syn::Ident {
        quote::format_ident!("rutie_{}_{}", self.class_name, fn_name)
    }
}
