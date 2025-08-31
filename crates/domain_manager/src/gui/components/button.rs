#![allow(clippy::module_name_repetitions)]

use crate::gui::handlers::message_handler::{MessageCategory, NavigationMessage};
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::text::TextType;
use crate::translations::translations::hide_translation;
use crate::translations::types::language::Language;
use crate::utils::types::file_info::FileInfo;
use crate::utils::types::icon::Icon;
use crate::StyleType;
use iced::widget::text::LineHeight;
use iced::widget::tooltip::Position;
use iced::widget::{button, Row, Text, Tooltip};
use iced::{Alignment, Font};

pub fn button_hide<'a>(
    message: MessageCategory,
    language: Language,
    font: Font,
) -> Tooltip<'a, MessageCategory, StyleType> {
    Tooltip::new(
        button(
            Text::new("×")
                .font(font)
                .align_y(Alignment::Center)
                .align_x(Alignment::Center)
                .size(15)
                .line_height(LineHeight::Relative(1.0)),
        )
        .padding(2)
        .height(20)
        .width(20)
        .on_press(message),
        Text::new(hide_translation(language)).font(font),
        Position::Right,
    )
    .gap(5)
    .class(ContainerType::Tooltip)
}

pub fn button_open_file<'a>(
    old_file: String,
    file_info: FileInfo,
    language: Language,
    font: Font,
    is_editable: bool,
    action: fn(String) -> MessageCategory,
) -> Tooltip<'a, MessageCategory, StyleType> {
    let mut tooltip_str = "";
    let mut tooltip_style = ContainerType::Standard;

    let mut button = button(
        Icon::File
            .to_text()
            .align_y(Alignment::Center)
            .align_x(Alignment::Center)
            .size(16.0),
    )
    .padding(0)
    .height(25)
    .width(40);

    if is_editable {
        tooltip_str = file_info.action_info(language);
        tooltip_style = ContainerType::Tooltip;
        button = button.on_press(MessageCategory::Navigation(NavigationMessage::OpenFile(
            old_file, file_info, action,
        )));
    }

    Tooltip::new(button, Text::new(tooltip_str).font(font), Position::Right)
        .gap(5)
        .class(tooltip_style)
}

pub fn row_open_link_tooltip<'a>(
    text: &'static str,
    font: Font,
) -> Row<'a, MessageCategory, StyleType> {
    Row::new()
        .align_y(Alignment::Center)
        .spacing(10)
        .push(Text::new(text).font(font))
        .push(Icon::OpenLink.to_text().size(16).class(TextType::Title))
}
