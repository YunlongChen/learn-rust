use rust_i18n::t;

pub fn get_text(name: &str) -> String {
    t!(name).into()
}
