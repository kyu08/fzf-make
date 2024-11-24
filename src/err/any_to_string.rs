// ref: https://qiita.com/kgtkr/items/a17827c4bb704f39c854
pub fn any_to_string(any: &dyn std::any::Any) -> String {
    if let Some(s) = any.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = any.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Any".to_string()
    }
}
