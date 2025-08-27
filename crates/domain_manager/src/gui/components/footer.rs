//! GUI bottom footer

use std::sync::Mutex;

use crate::gui::components::button::row_open_link_tooltip;
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::style_constants::{FONT_SIZE_FOOTER, FONT_SIZE_SUBTITLE};
use crate::gui::styles::text::TextType;
use crate::gui::styles::types::gradient_type::GradientType;
use crate::gui::styles::types::style_type::StyleType;
use crate::gui::types::message::Message;
use crate::translations::translations_2::new_version_available_translation;
use crate::translations::translations_4::share_feedback_translation;
use crate::translations::types::language::Language;
use crate::utils::formatted_strings::APP_VERSION;
use crate::utils::types::icon::Icon;
use crate::utils::types::web_page::WebPage;
use crate::DOMAIN_MANAGER_LOWERCASE;
use iced::widget::text::LineHeight;
use iced::widget::tooltip::Position;
use iced::widget::{button, rich_text, span, Column, Container, Row, Text, Tooltip};
use iced::widget::{horizontal_space, Space};
use iced::{Alignment, Font, Length, Padding};

pub fn footer<'a>(
    thumbnail: bool,
    language: Language,
    color_gradient: GradientType,
    font: Font,
    font_footer: Font,
    newer_release_available: &Mutex<Option<bool>>,
) -> Container<'a, Message, StyleType> {
    if thumbnail {
        return thumbnail_footer();
    }

    let release_details_row =
        get_release_details(language, font, font_footer, newer_release_available);

    let footer_row = Row::new()
        .spacing(10)
        .padding([0, 20])
        .align_y(Alignment::Center)
        .push(release_details_row)
        .push(get_button_feedback(font, language))
        .push(get_button_wiki(font))
        .push(get_button_github(font))
        .push(get_button_news(font))
        .push(get_button_sponsor(font))
        .push(
            Column::new()
                .push(
                    rich_text![
                        "Made with ❤ by ",
                        span("Stanic.xyz")
                            .underline(true)
                            .link(Message::OpenWebPage(WebPage::MyGitHub)),
                    ]
                    .size(FONT_SIZE_FOOTER)
                    .font(font_footer),
                )
                .width(Length::Fill)
                .align_x(Alignment::End),
        );

    Container::new(footer_row)
        .height(45)
        .align_y(Alignment::Center)
        .class(ContainerType::Gradient(color_gradient))
}

fn get_button_feedback<'a>(font: Font, language: Language) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Icon::Feedback
            .to_text()
            .size(15)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .line_height(LineHeight::Relative(1.0)),
    )
    .padding(Padding::ZERO.top(2))
    .height(30)
    .width(30)
    .on_press(Message::OpenWebPage(WebPage::Issues));

    Tooltip::new(
        content,
        row_open_link_tooltip(share_feedback_translation(language), font),
        Position::Top,
    )
    .gap(10)
    .class(ContainerType::Tooltip)
}

fn get_button_wiki<'a>(font: Font) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Icon::Book
            .to_text()
            .size(19)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .line_height(LineHeight::Relative(1.0)),
    )
    .padding(Padding::ZERO.top(1))
    .height(35)
    .width(35)
    .on_press(Message::OpenWebPage(WebPage::Wiki));

    Tooltip::new(
        content,
        row_open_link_tooltip("Domain Manager Wiki", font),
        Position::Top,
    )
    .gap(7.5)
    .class(ContainerType::Tooltip)
}

fn get_button_github<'a>(font: Font) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Icon::GitHub
            .to_text()
            .size(26)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .line_height(LineHeight::Relative(1.0)),
    )
    .height(40)
    .width(40)
    .on_press(Message::OpenWebPage(WebPage::Repo));

    Tooltip::new(
        content,
        row_open_link_tooltip("GitHub", font),
        Position::Top,
    )
    .gap(5)
    .class(ContainerType::Badge)
}

fn get_button_news<'a>(font: Font) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Icon::News
            .to_text()
            .size(16)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .line_height(LineHeight::Relative(1.0)),
    )
    .height(35)
    .width(35)
    .on_press(Message::OpenWebPage(WebPage::WebsiteNews));

    Tooltip::new(
        content,
        row_open_link_tooltip("Sniffnet News", font),
        Position::Top,
    )
    .gap(7.5)
    .class(ContainerType::Tooltip)
}

fn get_button_sponsor<'a>(font: Font) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Text::new('❤'.to_string())
            .font(font)
            .size(23)
            .class(TextType::Sponsor)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .line_height(LineHeight::Relative(1.0)),
    )
    .padding(Padding::ZERO.top(2))
    .height(30)
    .width(30)
    .on_press(Message::OpenWebPage(WebPage::WebsiteSponsor));

    Tooltip::new(
        content,
        row_open_link_tooltip("Sponsor", font),
        Position::Top,
    )
    .gap(10)
    .class(ContainerType::Tooltip)
}

/// 获取版本信息显示组件
/// 
/// # 参数
/// * `language` - 语言设置
/// * `font` - 字体
/// * `font_footer` - footer字体
/// * `newer_release_available` - 是否有新版本可用
fn get_release_details<'a>(
    language: Language,
    font: Font,
    font_footer: Font,
    newer_release_available: &Mutex<Option<bool>>,
) -> Row<'a, Message, StyleType> {
    let has_update = if let Some(boolean_response) = *newer_release_available.lock().unwrap() {
        boolean_response
    } else {
        false
    };

    // 创建可点击的版本号按钮
    let version_button = button(
        Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(
                Icon::DomainManager
                    .to_text()
                    .size(16)
                    .align_y(Alignment::Center)
                    .class(if has_update { TextType::Danger } else { TextType::Standard })
            )
            .push(
                Text::new(format!("{DOMAIN_MANAGER_LOWERCASE} v{APP_VERSION}"))
                    .size(FONT_SIZE_FOOTER)
                    .font(font_footer)
                    .class(if has_update { TextType::Danger } else { TextType::Standard })
            )
            .push_maybe(if has_update {
                Some(
                    Icon::Update
                        .to_text()
                        .size(14)
                        .class(TextType::Danger)
                        .align_y(Alignment::Center)
                )
            } else {
                Some(
                    Text::new(" ✔")
                        .size(12)
                        .font(font_footer)
                        .class(TextType::Subtitle)
                        .align_y(Alignment::Center)
                )
            })
    )
    .padding([4, 8])
    .class(ButtonType::Standard)
    .on_press(if has_update {
        Message::OpenWebPage(WebPage::WebsiteDownload)
    } else {
        Message::ShowToast("当前已是最新版本".to_string())
    });

    let tooltip_text = if has_update {
        new_version_available_translation(language).to_string()
    } else {
        "点击检查更新".to_string()
    };

    let version_tooltip = Tooltip::new(
        version_button,
        Text::new(tooltip_text).font(font),
        Position::Top,
    )
    .gap(5)
    .class(ContainerType::Tooltip);

    Row::new()
        .align_y(Alignment::Center)
        .height(Length::Fill)
        .width(Length::Fill)
        .push(version_tooltip)
}

fn thumbnail_footer<'a>() -> Container<'a, Message, StyleType> {
    Container::new(horizontal_space()).height(0)
}
