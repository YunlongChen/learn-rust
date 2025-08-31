use crate::gui::handlers::message_handler::MessageCategory;
use crate::gui::styles::container::ContainerType;
use crate::StyleType;
use iced::widget::text::LineHeight;
use iced::widget::{Container, Text};
use iced::{Alignment, Font};

pub fn notifications_badge<'a>(
    font_headers: Font,
    num: usize,
) -> Container<'a, MessageCategory, StyleType> {
    Container::new(
        Text::new(num.to_string())
            .font(font_headers)
            .size(14)
            .line_height(LineHeight::Relative(1.0)),
    )
    .align_y(Alignment::Center)
    .padding([2, 4])
    .height(20)
    .class(ContainerType::Highlighted)
}
