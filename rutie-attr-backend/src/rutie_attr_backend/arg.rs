use rutie::{Boolean, Fixnum, Float, Hash, Integer, NilClass, RString, Symbol, AnyException, AnyObject, Exception, Object};

pub struct Arg<T> {
    pub result: Result<T, AnyException>,
}

pub trait FromArg<T>: Sized {
    fn from_arg(from: Option<&AnyObject>) -> Self;
}

macro_rules! impl_from_arg {
    ($($struct_name:ty),*) => ($(
        impl FromArg<$struct_name> for Arg<$struct_name> {
            fn from_arg(from: Option<&AnyObject>) -> Arg<$struct_name> {
                let result = if let Some(o) = from {
                    o.try_convert_to::<$struct_name>()
                } else {
                    Err(AnyException::new("ArgumentError", Some("missing argument")))
                };
                Arg { result }
            }
        }

        impl Into<$struct_name> for Arg<$struct_name> {
            fn into(self) -> $struct_name {
                self.result.as_ref().ok().unwrap().value().clone().into()
            }
        }
    )*)
}

impl_from_arg!(Boolean, Fixnum, Float, Hash, Integer, NilClass, RString, Symbol);
