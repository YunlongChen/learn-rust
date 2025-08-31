use crate::gui::handlers::message_handler::{MessageCategory, NavigationMessage};
use crate::gui::manager_v2::DomainManagerV2;
use crate::gui::pages::names::Page;
use crate::utils::i18_utils::get_text;
use crate::{StyleType, CONTENT_SIZE, TITLE_PADDING, TITLE_SIZE};
use iced::alignment::Horizontal;
use iced::widget::{button, row, text, Column, Container};
use iced::Length;
use std::sync::OnceLock;

pub fn help<'a>(app: &DomainManagerV2) -> Container<'a, MessageCategory, StyleType> {
    let title = button(
        text(get_text("help.title"))
            .width(Length::Shrink)
            .size(TITLE_SIZE)
            .align_x(Horizontal::Center),
    )
    .on_press(MessageCategory::Navigation(NavigationMessage::PageChanged(
        Page::DomainPage,
    )));

    let title = row!(title).padding(TITLE_PADDING);

    let mut content = Column::new().padding(5);
    for (key, desc) in get_help_text() {
        let to_text = |s| text(s).width(Length::Shrink).size(CONTENT_SIZE);
        let (key, desc) = (to_text(key), to_text(desc));
        let row = row!(key, desc).spacing(50).padding(2);
        content = content.push(row);
    }
    let content = Container::new(content)
        .width(Length::Fill)
        .center_x(Length::Fill);

    let container = Column::new().push(title).push(content).spacing(20);
    Container::new(container)
        .width(Length::Shrink)
        .center_x(Length::Fill)
}

const KEY_DESCRIPTION: &[(&str, &str)] = &[
    ("\n● 模式/播放", "\n"),
    ("h", "进入帮助页面"),
    ("[p, space]", "播放/暂停"),
    ("t", "切换语言(默认双语字幕, 每次切换至中文/日语/双语)"),
    ("s", "切换播放速度"),
    ("q", "关闭应用"),
    ("\n\n● 模式/帮助", "\n"),
    ("h", "退出帮助页面"),
    ("\n\n● 模式/退出", "\n"),
    ("y", "确认"),
    ("n", "取消"),
];

fn get_help_text() -> &'static Vec<(String, String)> {
    static KEY_DESCRIPTION_CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();

    KEY_DESCRIPTION_CACHE.get_or_init(|| {
        let get_len = |s: &str| {
            s.chars()
                .fold(0, |acc, ch| acc + if ch.is_ascii() { 1 } else { 2 })
        };

        let get_format = |s: &str, max_len: usize| {
            let count = max_len - get_len(s);
            String::from(s) + " ".repeat(count).as_str()
        };

        let (mut key_max_len, mut desc_max_len) = (0, 0);
        for (key, desc) in KEY_DESCRIPTION {
            key_max_len = get_len(key).max(key_max_len);
            desc_max_len = get_len(desc).max(desc_max_len);
        }

        KEY_DESCRIPTION
            .iter()
            .map(|(key, desc)| {
                let key = get_format(key, key_max_len);
                let desc = get_format(desc, desc_max_len);
                (key, desc)
            })
            .collect()
    })
}
