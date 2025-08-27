//! Containers style

#![allow(clippy::module_name_repetitions)]

use iced::border::Radius;
use iced::widget::container::{Catalog, Style};
use iced::{Background, Border, Color, Shadow};

use crate::gui::styles::style_constants::{BORDER_ROUNDED_RADIUS, BORDER_WIDTH};
use crate::gui::styles::types::gradient_type::{get_gradient_headers, GradientType};
use crate::configs::gui_config::BackgroundType;
use crate::StyleType;

#[derive(Debug, Clone, Copy)]
pub struct ContainerStyle {
    pub background: Option<Background>,
    pub border: Border,
    pub text_color: Option<Color>,
    pub shadow: Shadow,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: Border::default(),
            text_color: None,
            shadow: Shadow::default(),
        }
    }
}


#[derive(Default)]
pub enum ContainerType {
    #[default]
    Standard,
    Background,
    BorderedRound,
    Bordered,
    Tooltip,
    Badge,
    BadgeInfo,
    Palette,
    Gradient(GradientType),
    Modal,
    Highlighted,
    HighlightedOnHeader,
    ModalBackground,
    Selected,  // 选中状态
    Hoverable, // 可悬停
    /// 背景图片容器
    BackgroundImage(BackgroundType, f32), // 背景类型和透明度
}

impl ContainerType {
    fn appearance(&self, style: &StyleType) -> Style {
        let colors = style.get_palette();
        let ext = style.get_extension();
        Style {
            // 文本颜色
            text_color: Some(match self {
                ContainerType::Gradient(_) | ContainerType::Highlighted => colors.text_headers,
                _ => colors.text_body,
            }),
            background: Some(match self {
                ContainerType::Gradient(GradientType::None) | ContainerType::Highlighted => {
                    Background::Color(colors.secondary)
                }
                ContainerType::Tooltip => Background::Color(ext.buttons_color),
                ContainerType::BorderedRound | ContainerType::Bordered => {
                    Background::Color(Color {
                        a: ext.alpha_round_containers,
                        ..ext.buttons_color
                    })
                }
                ContainerType::Badge | ContainerType::BadgeInfo => Background::Color(Color {
                    a: ext.alpha_chart_badge,
                    ..colors.secondary
                }),
                ContainerType::Gradient(gradient_type) => Background::Gradient(
                    get_gradient_headers(&colors, *gradient_type, ext.is_nightly),
                ),
                ContainerType::Modal | ContainerType::HighlightedOnHeader => {
                    Background::Color(colors.primary)
                }
                ContainerType::Standard | ContainerType::Palette => {
                    Background::Color(Color::TRANSPARENT)
                }
                ContainerType::Background => Background::Color(ext.buttons_color),
                ContainerType::ModalBackground => Background::Color(Color {
                    a: 0.9,
                    ..Color::BLACK
                }),
                ContainerType::Selected => Background::Color(Color {
                    a: 0.3,
                    ..colors.primary
                }),
                ContainerType::Hoverable => Background::Color(Color {
                    a: 0.1,
                    ..colors.secondary
                }),
                ContainerType::BackgroundImage(_, opacity) => Background::Color(Color {
                    a: *opacity,
                    ..Color::TRANSPARENT
                }),
            }),
            border: Border {
                radius: match self {
                    ContainerType::BorderedRound => BORDER_ROUNDED_RADIUS.into(),
                    ContainerType::Modal => Radius::new(0).bottom(BORDER_ROUNDED_RADIUS),
                    ContainerType::Tooltip => 7.0.into(),
                    ContainerType::Badge
                    | ContainerType::BadgeInfo
                    | ContainerType::Highlighted
                    | ContainerType::HighlightedOnHeader => 100.0.into(),
                    ContainerType::Bordered => Radius::new(1).bottom(BORDER_ROUNDED_RADIUS),
                    _ => 0.0.into(),
                },
                width: match self {
                    ContainerType::Standard
                    | ContainerType::ModalBackground
                    | ContainerType::Gradient(_)
                    | ContainerType::HighlightedOnHeader
                    | ContainerType::Highlighted => 0.0,
                    ContainerType::Tooltip => BORDER_WIDTH / 2.0,
                    ContainerType::BorderedRound => BORDER_WIDTH * 2.0,
                    _ => BORDER_WIDTH,
                },
                color: match self {
                    ContainerType::Palette => Color::BLACK,
                    ContainerType::BadgeInfo => colors.secondary,
                    ContainerType::Modal => ext.buttons_color,
                    _ => Color {
                        a: ext.alpha_round_borders,
                        ..ext.buttons_color
                    },
                },
            },
            shadow: Shadow::default(),
        }
    }
}

impl Catalog for StyleType {
    type Class<'a> = ContainerType;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class.appearance(self)
    }
}
