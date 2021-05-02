use super::DEFINED_CLASSES;
use proc_macro::TokenStream;
use quote::quote;

pub struct Rbclass {
    item: syn::ItemStruct,
}

impl Rbclass {
    pub fn new(item: syn::ItemStruct) -> Self {
        Self { item }
    }

    pub fn token_stream(&self) -> TokenStream {
        let ast = self.item.clone();
        let class = &self.item.ident;
        let class_name = class.to_string();
        let rutie_class = quote::format_ident!("Rutie{}", class);

        DEFINED_CLASSES
            .lock()
            .unwrap()
            .entry(class_name)
            .or_insert_with(|| self.field_names());

        let content = self.impl_try_from(class);

        let gen = quote! {
            #ast
            rutie::class!(#rutie_class);

            impl std::convert::TryFrom<#rutie_class> for #class {
                type Error = rutie::AnyException;

                fn try_from(f: #rutie_class) -> Result<Self, Self::Error> {
                    #content
                }
            }
        };
        gen.into()
    }

    fn impl_try_from(&self, class: &syn::Ident) -> proc_macro2::TokenStream {
        let mut content = quote! {};
        if let syn::Fields::Named(fields) = &self.item.fields {
            for n in fields.named.iter() {
                if let Some(ident) = &n.ident {
                    let value = ident.to_string();
                    let nil_error_message = format!("{} field is nil.", value);
                    let ty = &n.ty;
                    content = quote! {
                        #content

                        let #ident = unsafe { f.send(#value, &[]) };
                        if let Ok(_) = #ident.try_convert_to::<rutie::NilClass>() {
                            return Err(rutie::AnyException::new("StandardError", Some(#nil_error_message)));
                        }

                        let #ident = #ident.try_convert_to::<#ty>();
                        if let Err(e) = #ident {
                            return Err(e);
                        }
                        let #ident = #ident.unwrap();
                    };
                }
            }
        }

        let class_struct = self.construct_class(&class);
        content = quote! {
            #content
            Ok(#class_struct)
        };
        content
    }

    fn construct_class(&self, class: &syn::Ident) -> syn::ExprStruct {
        let mut cstruct: syn::ExprStruct = syn::parse_quote! { #class {} };
        if let syn::Fields::Named(fields) = &self.item.fields {
            for n in fields.named.iter() {
                if let Some(ident) = &n.ident {
                    let field: syn::FieldValue = syn::parse_quote! { #ident: #ident };
                    cstruct.fields.push(field);
                }
            }
        }
        cstruct
    }

    fn field_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let syn::Fields::Named(fields) = &self.item.fields {
            for n in fields.named.iter() {
                if let Some(ident) = &n.ident {
                    names.push(ident.to_string());
                }
            }
        }
        names
    }
}
