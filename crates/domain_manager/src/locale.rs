#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Chinese,
    English,
}



impl From<String> for Locale {
    fn from(value: String) -> Self {
        match value.as_str() {
            "zh_CN" => Self::Chinese,
            "en" => Self::English,
            &_ => Self::Chinese,
        }
    }
}
