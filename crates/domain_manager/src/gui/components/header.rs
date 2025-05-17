//! GUI upper header

use crate::gui::components::tab::notifications_badge;
use crate::gui::manager::DomainManager;
use crate::gui::pages::types::settings::SettingsPage;
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::types::gradient_type::GradientType;
use crate::gui::types::message::Message;
use crate::translations::translations::{quit_analysis_translation, settings_translation};
use crate::translations::translations_3::thumbnail_mode_translation;
use crate::translations::types::language::Language;
use crate::utils::types::icon::Icon;
use crate::{StyleType, DOMAIN_MANAGER_LOWERCASE};
use iced::widget::text::LineHeight;
use iced::widget::tooltip::Position;
use iced::widget::{button, horizontal_space, Container, Row, Space, Text, Tooltip};
use iced::{Alignment, Font, Length};

pub fn header<'a>(app: &DomainManager) -> Container<'a, Message, StyleType> {
    let is_running = true;
    let config = &app.config;
    let font = app.config.style_type.get_extension().font;

    let logo = Icon::Sniffnet
        .to_text()
        .align_y(Alignment::Center)
        .height(Length::Fill)
        .line_height(LineHeight::Relative(0.7))
        .size(80);

    Container::new(
        Row::new()
            .padding([0, 20])
            .align_y(Alignment::Center)
            .push(if is_running {
                Container::new(get_button_reset(font, config.language))
            } else {
                Container::new(Space::with_width(60))
            })
            .push(horizontal_space())
            .push(Container::new(Space::with_width(40)))
            .push(Space::with_width(20))
            .push(logo)
            .push(Space::with_width(20))
            // .push(if is_running {
            //     Container::new(get_button_minimize(
            //         NIGHT_PALETTE_EXTENSION.font,
            //         config.language,
            //         false,
            //     ))
            // } else {
            //     Container::new(Space::with_width(40))
            // })
            .push(horizontal_space())
            .push(get_button_settings(
                font,
                config.language,
                //TODO 这里不能固定写
                SettingsPage::Appearance,
            ))
            .push(get_button_exit(
                font,
                config.language,
                SettingsPage::Appearance,
            )),
    )
    .height(70)
    .align_y(Alignment::Center)
    .class(ContainerType::Gradient(config.color_gradient))
}

fn get_button_reset<'a>(font: Font, language: Language) -> Tooltip<'a, Message, StyleType> {
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
    .on_press(Message::ResetButtonPressed);

    Tooltip::new(
        content,
        Text::new(quit_analysis_translation(language)).font(font),
        Position::Right,
    )
    .gap(5)
    .class(ContainerType::Tooltip)
}

pub fn get_button_settings<'a>(
    font: Font,
    language: Language,
    open_overlay: SettingsPage,
) -> Tooltip<'a, Message, StyleType> {
    let content = button(
        Icon::Settings
            .to_text()
            .size(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .height(40)
    .width(60)
    .on_press(Message::OpenSettings(open_overlay));

    Tooltip::new(
        content,
        Text::new(settings_translation(language)).font(font),
        Position::Left,
    )
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

    Tooltip::new(content, Text::new("退出").font(font), Position::Left)
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
        .gap(0)
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
