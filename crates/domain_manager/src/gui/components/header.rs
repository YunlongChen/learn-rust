//! GUI upper header

use crate::gui::components::tab::notifications_badge;
use crate::gui::manager::DomainManager;
use crate::gui::pages::names::Page;
use crate::gui::pages::types::settings::SettingsPage;
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::types::gradient_type::GradientType;
use crate::gui::types::message::Message;
use crate::translations::translations::quit_analysis_translation;
use crate::translations::translations_3::thumbnail_mode_translation;
use crate::translations::types::language::Language;
use crate::translations::types::locale::Locale;
use crate::utils::types::icon::Icon;
use crate::configs::gui_config::BackgroundType;
use crate::{get_text, StyleType, DOMAIN_MANAGER_LOWERCASE};
use iced::widget::text::LineHeight;
use iced::widget::tooltip::Position;
use iced::widget::{button, horizontal_space, Button, Container, Row, Space, Text, Tooltip};
use iced::{Alignment, Font, Length};
use iced::widget::MouseArea;

pub fn header<'a>(app: &DomainManager) -> Container<'a, Message, StyleType> {
    let is_running = true;
    let config = &app.config;
    let font = app.config.style_type.get_extension().font;

    let logo = Icon::DomainManager
        .to_text()
        .align_y(Alignment::Center)
        .height(Length::Fill)
        .line_height(LineHeight::Relative(0.7))
        .size(40);

    Container::new(
        MouseArea::new(
            Row::new()
                .padding([0, 20])
                .align_y(Alignment::Center)
                .push(if is_running {
                    Container::new(get_button_reset(&app, font, config.language))
                } else {
                    Container::new(Space::with_width(60))
                })
                .push(horizontal_space())
                .push(Container::new(Space::with_width(40)))
                .push(Space::with_width(20))
                .push(logo)
                .push(Space::with_width(20))
                .push(horizontal_space())
                .push(Row::new().push(Text::new(format!("{}", &app.message))))
                .push_maybe(if app.in_query {
                    Some(get_custom_button(
                        font,
                        config.language,
                        SettingsPage::Appearance,
                        None,
                        Icon::Sync,
                        get_text("in_query"),
                    ))
                } else {
                    None
                })
                .push(get_custom_button(
                    font,
                    config.language,
                    SettingsPage::Appearance,
                    Some(Message::AddDnsProvider),
                    Icon::Add,
                    get_text("provider.add"),
                ))
                .push(get_custom_button(
                    font,
                    config.language,
                    SettingsPage::Appearance,
                    Some(Message::ToggleTheme),
                    Icon::HalfSun,
                    get_text("change_theme"),
                ))
                .push(get_custom_button(
                    font,
                    config.language,
                    SettingsPage::Appearance,
                    Some(Message::ChangeLocale(Locale::Chinese)),
                    Icon::Language,
                    get_text("change_locale"),
                ))
                .push(get_custom_button(
                    font,
                    config.language,
                    SettingsPage::Appearance,
                    Some(Message::ChangePage(Page::Console)),
                    Icon::Terminal,
                    "控制台",
                ))
                .push(get_custom_button(
                    font,
                    config.language,
                    SettingsPage::Appearance,
                    Some(Message::OpenSettings(SettingsPage::Appearance)),
                    Icon::Generals,
                    get_text("settings"),
                ))
                .push(get_background_button(
                    font,
                    config.language,
                    &app.config.background_config.background_type,
                ))
                .push(get_button_window_minimize(
                    font,
                    config.language,
                ))
                .push(get_button_window_maximize(
                    font,
                    config.language,
                ))
                .push(get_button_exit(
                    font,
                    config.language,
                    SettingsPage::Appearance,
                ))
                .spacing(10),
        )
        .on_press(Message::DragWindow),
    )
    .height(70)
    .align_y(Alignment::Center)
    .class(ContainerType::Gradient(config.color_gradient))
}

fn get_button_reset<'a>(
    app: &DomainManager,
    font: Font,
    language: Language,
) -> Tooltip<'a, Message, StyleType> {
    let last_page: Page = match &app.last_page {
        None => Page::DomainPage,
        Some(last_page) => last_page.clone(),
    };

    let content = button(
        Icon::ArrowBack
            .to_text()
            .size(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .line_height(LineHeight::Relative(1.0)),
    )
    .padding(10)
    .height(40)
    .width(60)
    .on_press(Message::ChangePage(last_page));

    Tooltip::new(
        content,
        Text::new(quit_analysis_translation(language)),
        Position::Right,
    )
    .gap(5)
    .class(ContainerType::Tooltip)
}

pub fn get_custom_button<'a>(
    font: Font,
    language: Language,
    open_overlay: SettingsPage,
    message: Option<Message>,
    icon: Icon,
    title: String,
) -> Tooltip<'a, Message, StyleType> {
    let content: Button<Message, StyleType> = button(
        icon.to_text()
            .size(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .height(40)
    .width(60)
    .on_press_maybe(message);

    Tooltip::new(content, Text::new(title.clone()), Position::Top)
        .gap(5)
        .class(ContainerType::Tooltip)
}

pub fn get_button_exit<'a>(
    font: Font,
    language: Language,
    open_overlay: SettingsPage,
) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Icon::Error
            .to_text()
            .size(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .height(40)
    .width(60)
    .on_press(Message::Quit);

    Tooltip::new(content, Text::new(get_text("exit")), Position::Top)
        .gap(5)
        .class(ContainerType::Tooltip)
}

/// 创建最小化按钮
/// 
/// # 参数
/// * `font` - 字体
/// * `language` - 语言
pub fn get_button_window_minimize<'a>(
    font: Font,
    language: Language,
) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Icon::Minimize
            .to_text()
            .size(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .height(40)
    .width(60)
    .on_press(Message::WindowMinimize);

    Tooltip::new(content, Text::new(get_text("minimize")), Position::Top)
        .gap(5)
        .class(ContainerType::Tooltip)
}

/// 创建最大化按钮
/// 
/// # 参数
/// * `font` - 字体
/// * `language` - 语言
pub fn get_button_window_maximize<'a>(
    font: Font,
    language: Language,
) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Icon::Maximize
            .to_text()
            .size(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .height(40)
    .width(60)
    .on_press(Message::WindowMaximize);

    Tooltip::new(content, Text::new(get_text("maximize")), Position::Top)
        .gap(5)
        .class(ContainerType::Tooltip)
}

/// 创建背景切换按钮
/// 
/// # 参数
/// * `font` - 字体
/// * `language` - 语言
/// * `current_background` - 当前背景类型
pub fn get_background_button<'a>(
    font: Font,
    language: Language,
    current_background: &BackgroundType,
) -> Tooltip<'a, Message, StyleType> {
    let (icon, tooltip, next_background) = match current_background {
        BackgroundType::None => (
            Icon::Image,
            get_text("background.china_red"),
            BackgroundType::ChinaRed,
        ),
        BackgroundType::ChinaRed => (
            Icon::Image,
            get_text("background.qipao_girl"),
            BackgroundType::QipaoGirl,
        ),
        BackgroundType::QipaoGirl => (
            Icon::Image,
            get_text("background.none"),
            BackgroundType::None,
        ),
    };

    let content = button(
        icon.to_text()
            .size(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .height(40)
    .width(60)
    .on_press(Message::ChangeBackground(next_background));

    Tooltip::new(content, Text::new(tooltip), Position::Top)
        .gap(5)
        .class(ContainerType::Tooltip)
}

pub fn get_button_minimize<'a>(
    font: Font,
    language: Language,
    thumbnail: bool,
) -> Tooltip<'a, Message, StyleType> {
    let size = if thumbnail { 20 } else { 24 };
    let button_size = if thumbnail { 30 } else { 40 };
    let icon = if thumbnail {
        Icon::ThumbnailClose
    } else {
        Icon::ThumbnailOpen
    };
    let tooltip = if thumbnail {
        ""
    } else {
        thumbnail_mode_translation(language)
    };
    let tooltip_style = if thumbnail {
        ContainerType::Standard
    } else {
        ContainerType::Tooltip
    };

    let content = button(
        icon.to_text()
            .size(size)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
        .padding(0)
        .height(button_size)
        .width(button_size)
        .class(ButtonType::Thumbnail)
        // .on_press(Message::ToggleThumbnail(false))
        ;

    Tooltip::new(content, Text::new(tooltip).font(font), Position::Right)
        .gap(5)
        .class(tooltip_style)
}

fn thumbnail_header<'a>(
    font: Font,
    font_headers: Font,
    language: Language,
    color_gradient: GradientType,
    unread_notifications: usize,
) -> Container<'a, Message, StyleType> {
    Container::new(
        Row::new()
            .align_y(Alignment::Center)
            .push(horizontal_space())
            .push(Space::with_width(80))
            .push(Text::new(DOMAIN_MANAGER_LOWERCASE).font(font_headers))
            .push(Space::with_width(10))
            .push(get_button_minimize(font, language, true))
            .push(horizontal_space())
            .push(if unread_notifications > 0 {
                Container::new(
                    notifications_badge(font, unread_notifications)
                        .class(ContainerType::HighlightedOnHeader),
                )
                .width(40)
                .align_x(Alignment::Center)
            } else {
                Container::new(Space::with_width(40))
            }),
    )
    .height(30)
    .align_y(Alignment::Center)
    .class(ContainerType::Gradient(color_gradient))
}
