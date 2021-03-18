use rutie::{RString, Object, VM};
use rutie_attr::{rbclass, rbmethods, rbdef};

#[rbclass]
pub struct Foo {}

#[rbmethods]
impl Foo {
    #[rbdef(name = "test?")]
    fn test(a: RString) -> RString {
        a
    }

    #[rbdef]
    fn hoge(b: RString) -> RString {
        b
    }
}
