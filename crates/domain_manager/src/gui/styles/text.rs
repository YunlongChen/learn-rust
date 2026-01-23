//! Text style

#![allow(clippy::module_name_repetitions)]

use crate::gui::handlers::message_handler::MessageCategory;
use crate::StyleType;
use iced::widget::text::{Catalog, Style};
use iced::widget::{Column, Text};
use iced::{Color, Font};

#[derive(Copy, Clone, Default, PartialEq)]
pub enum TextType {
    #[default]
    Standard,
    Incoming,
    Outgoing,
    Title,
    Subtitle,
    Danger,
    Success,
    Warning,
    Sponsor,
    Starred,
}

/// Returns a formatted caption followed by subtitle, new line, tab, and desc
impl TextType {
    pub fn highlighted_subtitle_with_desc<'a>(
        subtitle: &str,
        desc: &str,
        font: Font,
    ) -> Column<'a, MessageCategory, StyleType> {
        Column::new()
            .push(
                Text::new(format!("{subtitle}:"))
                    .class(TextType::Subtitle)
                    .font(font),
            )
            .push(Text::new(format!("   {desc}")).font(font))
    }

    fn appearance(self, style: &StyleType) -> Style {
        Style {
            color: if self == TextType::Standard {
                None
            } else {
                Some(highlight(style, self))
            },
        }
    }
}

pub fn highlight(style: &StyleType, element: TextType) -> Color {
    let colors = style.get_palette();
    let ext = style.get_extension();
    let secondary = colors.secondary;
    let is_nightly = style.get_extension().is_nightly;
    match element {
        TextType::Title => {
            let (p1, c) = if is_nightly { (0.6, 1.0) } else { (0.9, 0.7) };
            Color {
                r: c * (1.0 - p1) + secondary.r * p1,
                g: c * (1.0 - p1) + secondary.g * p1,
                b: c * (1.0 - p1) + secondary.b * p1,
                a: 1.0,
            }
        }
        TextType::Subtitle => {
            let (p1, c) = if is_nightly { (0.4, 1.0) } else { (0.6, 0.7) };
            Color {
                r: c * (1.0 - p1) + secondary.r * p1,
                g: c * (1.0 - p1) + secondary.g * p1,
                b: c * (1.0 - p1) + secondary.b * p1,
                a: 1.0,
            }
        }
        TextType::Incoming => colors.secondary,
        TextType::Outgoing => colors.outgoing,
        TextType::Danger | TextType::Sponsor => ext.red_alert_color,
        TextType::Success => Color::from_rgb(0.0, 0.8, 0.0),
        TextType::Warning => Color::from_rgb(1.0, 0.8, 0.0),
        TextType::Standard => colors.text_body,
        TextType::Starred => colors.starred,
    }
}

impl Catalog for StyleType {
    type Class<'a> = TextType;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class.appearance(self)
    }
}
