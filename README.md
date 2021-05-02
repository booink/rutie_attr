# Attribute for Rutie

Rutieの各種マクロをAttributeマクロで書けるようにするラッパー。

## Example

```rust
use rutie::{RString, Object, VM};
use rutie_attr::{rbclass, rbmethods, rbdef};

#[rbclass]
pub struct Foo {
  pub field1: RString,
  field2: Fixnum,
}

#[rbmethods]
impl Foo {
    #[rbdef(hoge?(a = "a"))]
    fn hoge(a: RString) -> RString {
        a
    }

    #[rbdef(fuga!(b = "1"))]
    fn fuga(b: RString) -> RString {
        b
    }
}
```

上記のRustのコードが以下のRubyのクラスとして使えるようになります。

```ruby
class Foo
  attr_accessor :field1
  attr_reader   :field2

  def self.hoge?(a = "a")
    a
  end

  def fuga!(b = "1")
    b
  end
end
```
