//! 域名列表组件
//!
//! 显示和管理域名列表的可重用组件

use super::{Component, ComponentConfig, State};
use crate::gui::handlers::message_handler::{
    DomainMessage, MessageCategory, NavigationMessage, SyncMessage,
};
use crate::gui::model::domain::{DnsProvider, Domain, DomainStatus};
use crate::gui::pages::Page;
use crate::gui::state::AppState;
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::text::TextType;
use crate::gui::styles::types::style_type::StyleType;
use crate::{get_text, StyleType as CrateStyleType}; // Ensure StyleType is imported correctly
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Padding};

/// 域名列表组件
#[derive(Debug, Clone)]
pub struct DomainListComponent {
    config: ComponentConfig,
    selected_domain: Option<String>,
    show_details: bool,
}

/// 域名列表项配置
#[derive(Debug, Clone)]
pub struct DomainListItemConfig {
    pub show_provider: bool,
    pub show_status: bool,
    pub show_record_count: bool,
    pub show_last_sync: bool,
    pub clickable: bool,
}

impl Default for DomainListItemConfig {
    fn default() -> Self {
        Self {
            show_provider: true,
            show_status: true,
            show_record_count: true,
            show_last_sync: true,
            clickable: true,
        }
    }
}

impl DomainListComponent {
    /// 创建新的域名列表组件
    pub fn new() -> Self {
        Self {
            config: ComponentConfig::default(),
            selected_domain: None,
            show_details: true,
        }
    }

    /// 设置选中的域名
    pub fn set_selected_domain(&mut self, domain: Option<String>) {
        self.selected_domain = domain;
    }

    /// 获取选中的域名
    pub fn get_selected_domain(&self) -> Option<&String> {
        self.selected_domain.as_ref()
    }

    /// 设置是否显示详细信息
    pub fn set_show_details(&mut self, show_details: bool) {
        self.show_details = show_details;
    }

    /// 渲染域名列表项
    fn render_domain_item<'a>(
        &'a self,
        domain: Domain,
        is_selected: bool,
        item_config: &DomainListItemConfig,
        state: &AppState,
    ) -> Element<'a, MessageCategory, StyleType> {
        let mut content = column![];

        let mut row_content = row![].align_y(Alignment::Center).spacing(10);

        // 添加状态指示器
        if item_config.show_status {
            let status_color = match domain.status.as_str() {
                "Active" => iced::Color::from_rgb(0.2, 0.7, 0.3),   // 绿色
                "Error" => iced::Color::from_rgb(0.8, 0.2, 0.2),    // 红色
                _ => iced::Color::from_rgb(0.6, 0.6, 0.6),          // 灰色
            };

            row_content = row_content.push(
                container(text(" ").size(12))
                    .width(Length::Fixed(4.0))
                    .height(Length::Fixed(16.0))
                    .class(ContainerType::CustomRound(status_color)),
            );
        }

        // 域名名称行
        row_content = row_content.push(
            text(domain.name.clone())
                .size(16)
                .width(Length::Fill)
                .class(if is_selected {
                    TextType::Outgoing
                } else {
                    TextType::Standard
                }),
        );

        content = content.push(row_content);

        // 详细信息行
        if self.show_details {
            let mut details_row = row![].spacing(15);

            // 提供商信息
            if item_config.show_provider {
                details_row =
                    details_row.push(text(format!("提供商: {}", domain.provider)).size(12));
            }

            // DNS记录数量
            if item_config.show_record_count {
                let record_count = state
                    .data
                    .dns_records_cache
                    // Fixme 这里没有错误处理，后续进行修复
                    .get(&(domain.id as usize))
                    .map(|records| records.len())
                    .unwrap_or(0);

                details_row = details_row.push(text(format!("记录: {}", record_count)).size(12));
            }

            // 最后同步时间
            if item_config.show_last_sync {
                let last_sync = state
                    .data
                    .domain_stats
                    .get(&domain.name)
                    .and_then(|stats| stats.last_sync)
                    .map(|time| time.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "未同步".to_string());

                details_row = details_row.push(text(format!("同步: {}", last_sync)).size(12));
            }

            content = content.push(details_row);
        }

        // 包装在容器中
        let item_container = container(content)
            .padding(Padding::from([8, 12]))
            .width(Length::Fill)
            .class(if is_selected {
                ContainerType::Selected
            } else {
                ContainerType::Standard
            });

        // 如果可点击，包装在按钮中（实际上是用 button 模拟点击区域，但样式自定义）
        if item_config.clickable {
            button(item_container)
                .on_press(MessageCategory::Domain(DomainMessage::Selected(
                    domain.clone(),
                )))
                .padding(0) // 移除按钮内边距
                .class(ButtonType::Transparent) // 移除按钮默认样式
                .width(Length::Fill)
                .into()
        } else {
            item_container.into()
        }
    }

    /// 渲染空状态
    fn render_empty_state(&self) -> Element<'_, MessageCategory, StyleType> {
        container(
            column![
                text("暂无域名").size(18),
                text("点击添加按钮来添加您的第一个域名").size(14),
                button("添加域名").on_press(MessageCategory::Navigation(
                    NavigationMessage::PageChanged(Page::AddDomain)
                ))
            ]
            .align_x(Alignment::Center)
            .spacing(10),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fixed(200.0))
        .into()
    }

    /// 渲染加载状态
    fn render_loading_state(&self) -> Element<'_, MessageCategory, StyleType> {
        container(
            column![
                text(get_text("message.list_is_blank")).size(16),
                // 这里可以添加加载动画
            ]
            .align_x(Alignment::Center)
            .spacing(10),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fixed(100.0))
        .into()
    }

    /// 渲染错误状态
    fn render_error_state(&self, error: &str) -> Element<'_, MessageCategory, StyleType> {
        container(
            column![
                text(get_text("message.list_is_blank")).size(16),
                text(error.to_string()).size(12),
                button(text(get_text("message.retry")))
                    .on_press(MessageCategory::Sync(SyncMessage::Reload))
            ]
            .align_x(Alignment::Center)
            .spacing(10),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fixed(150.0))
        .into()
    }
}

impl Component<AppState> for DomainListComponent {
    fn name(&self) -> &'static str {
        "domain_list"
    }

    fn view<'a>(&'a self, state: &'a State) -> Element<'a, MessageCategory, StyleType> {
        // 检查加载状态
        if state.ui.is_loading {
            return self.render_loading_state();
        }

        // 优先渲染数据（如果有）
        if !state.data.domain_list.is_empty() {
            let item_config = DomainListItemConfig::default();

            let domain_items: Vec<Element<MessageCategory, StyleType>> = state
                .data
                .domain_list
                .iter()
                .map(|domain| {
                    let is_selected = self
                        .selected_domain
                        .as_ref()
                        .map(|selected| selected == &domain.name)
                        .unwrap_or(false);

                    let domain_model = Domain {
                        id: domain.id.clone(),
                        name: domain.name.clone(),
                        status: DomainStatus::Active,
                        expiry: "".to_string(),
                        provider: DnsProvider::Aliyun,
                        records: vec![],
                    };
                    self.render_domain_item(domain_model, is_selected, &item_config, state)
                })
                .collect();

            let content = column(domain_items).spacing(2).width(Length::Fill);

            return scrollable(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        // 如果没有数据，再检查错误状态
        if !state.ui.message.is_empty() {
            return self.render_error_state(&state.ui.message);
        }

        // 最后渲染空状态
        self.render_empty_state()
    }

    fn is_visible(&self, _state: &AppState) -> bool {
        self.config.visible
    }

    fn is_enabled(&self, _state: &AppState) -> bool {
        self.config.enabled
    }
}

impl Default for DomainListComponent {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: 实现自定义样式
// struct SelectedItemStyle;
