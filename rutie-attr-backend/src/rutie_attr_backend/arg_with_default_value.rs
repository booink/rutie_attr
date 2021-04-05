use rutie::{Boolean, Fixnum, Float, Hash, Integer, NilClass, RString, Symbol, AnyException, AnyObject, Exception, Object};

pub struct DArg<T> {
    pub result: Result<T, AnyException>,
}

pub trait FromArgWithDefault<T>: Sized {
    fn from_arg_with_default(from: Option<&AnyObject>, default_value: Option<&AnyObject>) -> Self;
}

macro_rules! impl_from_arg_with_default {
    ($($struct_name:ty),*) => ($(
        impl FromArgWithDefault<$struct_name> for DArg<$struct_name> {
            fn from_arg_with_default(from: Option<&AnyObject>, default_value: Option<&AnyObject>) -> DArg<$struct_name> {
                let result = if let Some(o) = from {
                    o.try_convert_to::<$struct_name>()
                } else {
                    if let Some(o) = default_value {
                        o.try_convert_to::<$struct_name>()
                    } else {
                        Err(AnyException::new("ArgumentError", Some("missing argument")))
                    }
                };
                DArg { result }
            }
        }

        impl Into<$struct_name> for DArg<$struct_name> {
            fn into(self) -> $struct_name {
                self.result.as_ref().ok().unwrap().value().clone().into()
            }
        }
    )*)
}

impl_from_arg_with_default!(Boolean, Fixnum, Float, Hash, Integer, NilClass, RString, Symbol);
