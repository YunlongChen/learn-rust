//! 设置页面实现
//! 
//! 提供应用程序设置界面，包括外观设置、通知设置和常规设置

use crate::configs::gui_config::BackgroundType;
use crate::gui::manager::DomainManager;
use crate::gui::pages::types::settings::SettingsPage;
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::types::message::Message;
use crate::translations::types::language::Language;
use crate::utils::types::icon::Icon;
use crate::{get_text, StyleType};
use iced::widget::{
    button, column, container, horizontal_space, row, slider, text, Button, Column, Container,
    Row, Slider, Text,
};
use iced::{Alignment, Element, Font, Length};

/// 创建设置页面
/// 
/// # 参数
/// * `app` - 应用程序状态
/// * `settings_page` - 当前设置页面类型
/// 
/// # 返回
/// 设置页面的UI元素
pub fn settings_page(
    app: &DomainManager,
    settings_page: SettingsPage,
) -> Element<'static, Message, StyleType> {
    let font = app.config.style_type.get_extension().font;
    let language = app.config.language;

    // 设置页面标题
    let title = text(settings_page.get_tab_label(language))
        .size(24)
        .font(font);

    // 根据设置页面类型显示不同内容
    let content = match &settings_page {
        SettingsPage::Appearance => appearance_settings(app, font, language),
        SettingsPage::Notifications => notifications_settings(app, font, language),
        SettingsPage::General => general_settings(app, font, language),
    };

    Container::new(
        Column::new()
            .spacing(20)
            .padding(20)
            .push(title)
            .push(content)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .class(ContainerType::Bordered)
    .into()
}

/// 外观设置页面
/// 
/// # 参数
/// * `app` - 应用程序状态
/// * `font` - 字体
/// * `language` - 语言
/// 
/// # 返回
/// 外观设置的UI元素
fn appearance_settings<'a>(
    app: &DomainManager,
    font: Font,
    language: Language,
) -> Element<'a, Message, StyleType> {
    let background_config = &app.config.background_config;

    // 背景类型选择
    let background_section = Column::new()
        .spacing(10)
        .push(
            text(get_text("settings.background_type"))
                .size(16)
                .font(font),
        )
        .push(
            Row::new()
                .spacing(10)
                .push(background_type_button(
                    BackgroundType::None,
                    background_config.background_type.clone(),
                    get_text("settings.none"),
                    font,
                ))
                .push(background_type_button(
                    BackgroundType::ChinaRed,
                    background_config.background_type.clone(),
                    get_text("settings.china_red"),
                    font,
                ))
                .push(background_type_button(
                    BackgroundType::QipaoGirl,
                    background_config.background_type.clone(),
                    get_text("settings.qipao_girl"),
                    font,
                )),
        );

    // 透明度调节
    let opacity_section = Column::new()
        .spacing(10)
        .push(
            text(format!(
                "{}: {:.0}%",
                get_text("settings.background_opacity"),
                background_config.opacity * 100.0
            ))
            .size(16)
            .font(font),
        )
        .push(
            Slider::new(0.0..=1.0, background_config.opacity, Message::BackgroundOpacityChanged)
                .step(0.01)
                .width(Length::Fixed(300.0)),
        );

    // 主题切换
    let theme_section = Column::new()
        .spacing(10)
        .push(
            text(get_text("settings.theme"))
                .size(16)
                .font(font),
        )
        .push(
            Button::new(
                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        Icon::HalfSun
                            .to_text()
                            .size(16)
                            .align_y(Alignment::Center),
                    )
                    .push(text(get_text("settings.toggle_theme")).font(font)),
            )
            .on_press(Message::ToggleTheme)
            .class(ButtonType::Standard),
        );

    Column::new()
        .spacing(30)
        .push(background_section)
        .push(opacity_section)
        .push(theme_section)
        .width(Length::Fill)
        .into()
}

/// 创建背景类型选择按钮
/// 
/// # 参数
/// * `background_type` - 背景类型
/// * `current_type` - 当前选中的背景类型
/// * `label` - 按钮标签
/// * `font` - 字体
/// 
/// # 返回
/// 背景类型按钮
fn background_type_button(
    background_type: BackgroundType,
    current_type: BackgroundType,
    label: String,
    font: Font,
) -> Button<'static, Message, StyleType> {
    let is_selected = background_type == current_type;
    
    Button::new(
        text(label)
            .font(font)
            .align_x(Alignment::Center),
    )
    .on_press(Message::ChangeBackground(background_type))
    .width(Length::Fixed(120.0))
    .class(if is_selected {
        ButtonType::BorderedRoundSelected
    } else {
        ButtonType::BorderedRound
    })
}

/// 通知设置页面
/// 
/// # 参数
/// * `app` - 应用程序状态
/// * `font` - 字体
/// * `language` - 语言
/// 
/// # 返回
/// 通知设置的UI元素
fn notifications_settings<'a>(
    _app: &DomainManager,
    font: Font,
    language: Language,
) -> Element<'a, Message, StyleType> {
    Column::new()
        .spacing(20)
        .push(
            text(get_text("notifications_under_development"))
                .size(16)
                .font(font),
        )
        .push(
            Row::new()
                .spacing(15)
                .push(
                    Button::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(
                                Icon::Bell
                                    .to_text()
                                    .size(16)
                                    .align_y(Alignment::Center),
                            )
                            .push(text(get_text("settings.enable_notifications")).font(font)),
                    )
                    .on_press(Message::ShowToast(get_text("feature_under_development")))
                    .class(ButtonType::Standard),
                )
                .push(
                    Button::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(
                                Icon::Settings
                                    .to_text()
                                    .size(16)
                                    .align_y(Alignment::Center),
                            )
                            .push(text(get_text("settings.notification_preferences")).font(font)),
                    )
                    .on_press(Message::ShowToast(get_text("feature_under_development")))
                    .class(ButtonType::Standard),
                )
        )
        .width(Length::Fill)
        .into()
}

/// 常规设置页面
/// 
/// # 参数
/// * `app` - 应用程序状态
/// * `font` - 字体
/// * `language` - 语言
/// 
/// # 返回
/// 常规设置的UI元素
fn general_settings<'a>(
    app: &DomainManager,
    font: Font,
    language: Language,
) -> Element<'a, Message, StyleType> {
    Column::new()
        .spacing(20)
        .push(
            text(get_text("general_settings_under_development"))
                .size(16)
                .font(font),
        )
        .push(
            Row::new()
                .spacing(15)
                .push(
                    Button::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(
                                Icon::Language
                                    .to_text()
                                    .size(16)
                                    .align_y(Alignment::Center),
                            )
                            .push(text(get_text("settings.auto_start")).font(font)),
                    )
                    .on_press(Message::ShowToast(get_text("feature_under_development")))
                    .class(ButtonType::Standard),
                )
                .push(
                    Button::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(
                                Icon::Update
                                    .to_text()
                                    .size(16)
                                    .align_y(Alignment::Center),
                            )
                            .push(text(get_text("settings.check_updates")).font(font)),
                    )
                    .on_press(Message::ShowToast(get_text("feature_under_development")))
                    .class(ButtonType::Standard),
                )
                .push(
                    Button::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(
                                Icon::Window
                                    .to_text()
                                    .size(16)
                                    .align_y(Alignment::Center),
                            )
                            .push(text(if app.floating_window_enabled {
                                get_text("settings.disable_floating_window")
                            } else {
                                get_text("settings.enable_floating_window")
                            }).font(font)),
                    )
                    .on_press(Message::ToggleFloatingWindow)
                    .class(if app.floating_window_enabled {
                        ButtonType::BorderedRoundSelected
                    } else {
                        ButtonType::Standard
                    }),
                )
        )
        .push(
            Row::new()
                .spacing(15)
                .push(
                    Button::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(
                                Icon::Export
                                    .to_text()
                                    .size(16)
                                    .align_y(Alignment::Center),
                            )
                            .push(text(get_text("settings.export_data")).font(font)),
                    )
                    .on_press(Message::ShowToast(get_text("feature_under_development")))
                    .class(ButtonType::Standard),
                )
                .push(
                    Button::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(
                                Icon::Import
                                    .to_text()
                                    .size(16)
                                    .align_y(Alignment::Center),
                            )
                            .push(text(get_text("settings.import_data")).font(font)),
                    )
                    .on_press(Message::ShowToast(get_text("feature_under_development")))
                    .class(ButtonType::Standard),
                )
        )
        .width(Length::Fill)
        .into()
}