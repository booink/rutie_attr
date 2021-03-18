use rutie::Object;
use rutie_derive::{RbClass, rbclass, rbmethods};

//#[derive(RbClass, Debug, PartialEq)]
#[rbclass]
#[derive(Debug, PartialEq)]
pub struct Foo {
    attr1: String,
    attr2: u16,
}

#[rbmethods]
impl Foo {
    pub fn test() {}
}

#[test]
fn test_foo() {
    let foo = Foo { attr1: String::from("value1"), attr2: 2 };
    assert_eq!(foo, Foo { attr1: String::from("value1"), attr2: 2 });
}

/*
fn type_name<T>(_: T) -> String {
    format!("{}", std::any::type_name::<T>())
}

#[test]
fn test_impl_rb_class() {
    let rb_foo = Class::from_existing("RutieFoo");
    assert_eq!(rb_foo, Class::new("RutieFoo", None));
}
*/
