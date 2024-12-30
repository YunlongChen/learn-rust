use crate::view::danger_color;
use iced::border::Radius;
use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Theme};

pub fn danger_btn(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let border: Border = Border::default().rounded(Radius::default());

    let default_style: Style = Style {
        border,
        ..Default::default()
    };

    match status {
        Status::Hovered => Style {
            background: Some(Background::Color(Color::WHITE)),
            text_color: Color::BLACK,
            ..default_style
        },
        _ => Style {
            background: Some(Background::Color(danger_color())),
            text_color: Color::WHITE,
            ..default_style
        },
    }
}
