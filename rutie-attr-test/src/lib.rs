use rutie::{Fixnum, Object, RString};
use rutie_attr::{rbclass, rbdef, rbmethods};
use std::convert::TryFrom;
use rutie::Exception;

#[rbclass]
pub struct Foo {
    foo1: RString,
    foo2: Fixnum,
}

#[rbmethods]
impl Foo {
    #[rbdef(test?(a = "a"))]
    fn test(a: RString) -> RString {
        a
    }

    #[rbdef(_hoge!(b = "-112"))]
    fn hoge(&self, b: RString) -> RString {
        b
    }
}
