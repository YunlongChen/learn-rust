use iced::alignment::Alignment;
use iced::widget::{
    button, center, horizontal_space, mouse_area, opaque, stack, Column, Container, Row, Space,
    Text,
};
use iced::{Element, Font, Length};

use crate::gui::components::button::button_hide;
use crate::gui::handlers::message_handler::{
    MessageCategory, NavigationMessage, NotificationMessage,
};
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::style_constants::FONT_SIZE_TITLE;
use crate::gui::styles::types::gradient_type::GradientType;
use crate::translations::translations::{
    ask_clear_all_translation, ask_quit_translation, clear_all_translation,
    quit_analysis_translation, yes_translation,
};
use crate::translations::types::language::Language;
use crate::StyleType;

pub fn get_exit_overlay<'a>(
    message: MessageCategory,
    color_gradient: GradientType,
    font: Font,
    font_headers: Font,
    language: Language,
) -> Container<'a, MessageCategory, StyleType> {
    let row_buttons = confirm_button_row(language, font, message);

    let content = Column::new()
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .push(get_modal_header(
            font,
            font_headers,
            color_gradient,
            language,
            quit_analysis_translation(language),
        ))
        .push(Space::with_height(20))
        .push(
            ask_quit_translation(language)
                .align_x(Alignment::Center)
                .font(font),
        )
        .push(row_buttons);

    Container::new(content)
        .height(160)
        .width(450)
        .class(ContainerType::Modal)
}

pub fn get_clear_all_overlay<'a>(
    color_gradient: GradientType,
    font: Font,
    font_headers: Font,
    language: Language,
) -> Container<'a, MessageCategory, StyleType> {
    let row_buttons = confirm_button_row(
        language,
        font,
        MessageCategory::Notification(NotificationMessage::ClearAllNotifications),
    );

    let content = Column::new()
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .push(get_modal_header(
            font,
            font_headers,
            color_gradient,
            language,
            clear_all_translation(language),
        ))
        .push(Space::with_height(20))
        .push(
            ask_clear_all_translation(language)
                .align_x(Alignment::Center)
                .font(font),
        )
        .push(row_buttons);

    Container::new(content)
        .height(160)
        .width(450)
        .class(ContainerType::Modal)
}

fn get_modal_header<'a>(
    font: Font,
    font_headers: Font,
    color_gradient: GradientType,
    language: Language,
    title: &'static str,
) -> Container<'a, MessageCategory, StyleType> {
    Container::new(
        Row::new()
            .push(horizontal_space())
            .push(
                Text::new(title)
                    .font(font_headers)
                    .size(FONT_SIZE_TITLE)
                    .width(Length::FillPortion(6))
                    .align_x(Alignment::Center),
            )
            .push(
                Container::new(button_hide(
                    MessageCategory::Navigation(NavigationMessage::HideModal),
                    language,
                    font,
                ))
                .width(Length::Fill)
                .align_x(Alignment::Center),
            ),
    )
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .height(40)
    .width(Length::Fill)
    .class(ContainerType::Gradient(color_gradient))
}

fn confirm_button_row<'a>(
    language: Language,
    font: Font,
    message: MessageCategory,
) -> Row<'a, MessageCategory, StyleType> {
    Row::new()
        .height(Length::Fill)
        .align_y(Alignment::Center)
        .push(
            button(
                yes_translation(language)
                    .font(font)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Center),
            )
            .padding(5)
            .height(40)
            .width(80)
            .class(ButtonType::Alert)
            .on_press(message),
        )
}

pub fn modal<'a>(
    base: Element<'a, MessageCategory, StyleType>,
    content: Element<'a, MessageCategory, StyleType>,
    on_blur: MessageCategory,
) -> Element<'a, MessageCategory, StyleType> {
    stack![
        base,
        opaque(
            mouse_area(center(opaque(content)).class(ContainerType::ModalBackground))
                .on_press(on_blur)
        )
    ]
    .into()
}
