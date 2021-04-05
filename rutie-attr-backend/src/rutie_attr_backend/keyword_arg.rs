use rutie::{Boolean, Fixnum, Float, Hash, Integer, NilClass, RString, Symbol, AnyException, AnyObject, Exception, Object};

pub struct KwArg<T> {
    pub result: Result<T, AnyException>,
}

pub trait FromArgWithKeyAndDefault<T>: Sized {
    fn from_arg_with_key_and_default(from: Option<&AnyObject>, key: &str, default_value: Option<&AnyObject>) -> Self;
}

macro_rules! impl_from_arg_with_key_and_default {
    ($($struct_name:ty),*) => ($(
        impl FromArgWithKeyAndDefault<$struct_name> for KwArg<$struct_name> {
            fn from_arg_with_key_and_default(from: Option<&AnyObject>, key: &str, default_value: Option<&AnyObject>) -> KwArg<$struct_name> {
                let result = if let Some(o) = from {
                    if let Ok(h) = o.try_convert_to::<Hash>() {
                        h.at(&Symbol::new(key)).try_convert_to::<$struct_name>()
                    } else {
                        Err(AnyException::new("ArgumentError", Some("missing argument")))
                    }
                } else {
                    if let Some(o) = default_value {
                        o.try_convert_to::<$struct_name>()
                    } else {
                        Err(AnyException::new("ArgumentError", Some("missing argument")))
                    }
                };
                KwArg { result }
            }
        }

        impl Into<$struct_name> for KwArg<$struct_name> {
            fn into(self) -> $struct_name {
                self.result.as_ref().ok().unwrap().value().clone().into()
            }
        }
    )*)
}

impl_from_arg_with_key_and_default!(Boolean, Fixnum, Float, Hash, Integer, NilClass, RString, Symbol);
