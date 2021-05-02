pub fn combined_errors(errors: Vec<syn::Error>) -> Option<syn::Error> {
    let mut errors = errors.iter();
    if let Some(e) = errors.next() {
        Some(errors.fold(e.clone(), |mut acc, o| {
            acc.combine(o.clone());
            acc
        }))
    } else {
        // エラー無し
        None
    }
}

pub fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
