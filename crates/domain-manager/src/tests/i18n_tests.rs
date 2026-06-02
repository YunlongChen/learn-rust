//! 国际化功能测试
//!
//! 测试多语言切换和翻译功能

use crate::gui::state::app_state::{AppState, UiUpdate};
use crate::translations::types::locale::Locale;
use crate::utils::i18_utils::get_text;
use rust_i18n;
use tracing::info;

#[test]
fn test_locale_switching() {
    // 1. 初始化应用状态
    // 注意：在测试环境中，Config可能没有正确加载locale，默认为Chinese或English
    // 我们手动设置一个初始状态
    let mut app_state = AppState::default();

    // 强制设置为英文初始状态
    app_state.ui.locale = Locale::English;
    rust_i18n::set_locale("en");

    // 验证初始状态的翻译
    let en_text = get_text("add_domain");

    // 2. 切换语言
    app_state.update_ui(UiUpdate::ToggleLocale);

    // 验证状态是否更新
    assert_eq!(app_state.ui.locale, Locale::Chinese);

    // 验证 rust_i18n 的全局 locale 是否更新
    assert_eq!(rust_i18n::locale().to_string(), "zh_CN");

    // 验证翻译是否更新
    let zh_text = get_text("add_domain");

    // 验证 change_locale 翻译
    let zh_change_locale = get_text("change_locale");
    assert_eq!(zh_change_locale, "切换语言");

    // 再次切换回英文
    app_state.update_ui(UiUpdate::ToggleLocale);
    assert_eq!(app_state.ui.locale, Locale::English);
    assert_eq!(rust_i18n::locale().to_string(), "en");

    let en_text_again = get_text("add_domain");
    assert_eq!(en_text, en_text_again);
}
