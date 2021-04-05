pub mod rbclass;
pub mod rbdef;
pub mod rbmethods;
mod method;
mod argument;
mod rbdef_attr_util;
mod util;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// rbclassで解析した構造体のフィールド情報をrbmethodsで使うためのグローバル変数
static DEFINED_CLASSES: Lazy<Mutex<HashMap<String, Vec<String>>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});
