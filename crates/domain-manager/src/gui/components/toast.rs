//! Toast notification component
//!
//! 提供临时通知消息的UI组件

use crate::gui::handlers::message_handler::{MessageCategory, NotificationMessage};
use crate::gui::styles::container::ContainerType;
use crate::StyleType;
use iced::widget::{button, text, Column, Container, Row};
use iced::{Alignment, Element, Length, Padding};

/// Toast通知组件
///
/// # 参数
/// * `message` - 通知消息内容
/// * `show` - 是否显示通知
///
/// # 返回
/// Toast通知的UI元素
pub fn toast_notification(
    message: &str,
    show: bool,
) -> Option<Element<'_, MessageCategory, StyleType>> {
    if !show || message.is_empty() {
        return None;
    }

    Some(
        Container::new(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text("ℹ️").size(16))
                .push(text(message).size(14).width(Length::Fill))
                .push(
                    button(text("✕").size(12).align_x(Alignment::Center))
                        .on_press(MessageCategory::Notification(
                            NotificationMessage::HideToast,
                        ))
                        .width(Length::Fixed(24.0))
                        .height(Length::Fixed(24.0))
                        .padding(0),
                ),
        )
        .padding(Padding::new(12.0))
        .width(Length::Fixed(300.0))
        .class(ContainerType::Toast)
        .into(),
    )
}

/// 创建Toast容器，用于在界面上显示通知
///
/// # 参数
/// * `content` - 主要内容
/// * `toast_message` - Toast消息
/// * `show_toast` - 是否显示Toast
///
/// # 返回
/// 包含Toast的完整UI元素
pub fn with_toast<'a>(
    content: Element<'a, MessageCategory, StyleType>,
    toast_message: &'a str,
    show_toast: bool,
) -> Element<'a, MessageCategory, StyleType> {
    let mut column = Column::new().push(content);

    if let Some(toast) = toast_notification(toast_message, show_toast) {
        column = column.push(
            Container::new(toast)
                .width(Length::Fill)
                .align_x(Alignment::End)
                .padding(Padding::new(20.0)),
        );
    }

    column.into()
}
