//! 域名列表组件
//! 
//! 显示和管理域名列表的可重用组件

use super::{Component, ComponentConfig, ComponentStyle};
use crate::gui::state::AppState;
use crate::gui::Message;
use crate::models::Domain;
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
    fn render_domain_item(
        &self,
        domain: &Domain,
        is_selected: bool,
        item_config: &DomainListItemConfig,
        state: &AppState,
    ) -> Element<Message> {
        let mut content = column![];
        
        // 域名名称行
        let mut name_row = row![
            text(&domain.domain_name)
                .size(16)
                .style(if is_selected {
                    iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.4, 0.8))
                } else {
                    iced::theme::Text::Default
                })
        ]
        .align_items(Alignment::Center)
        .spacing(10);
        
        // 添加状态指示器
        if item_config.show_status {
            let status_color = match domain.status.as_str() {
                "Active" => iced::Color::from_rgb(0.2, 0.7, 0.3),
                "Inactive" => iced::Color::from_rgb(0.6, 0.6, 0.6),
                "Error" => iced::Color::from_rgb(0.8, 0.2, 0.2),
                _ => iced::Color::from_rgb(0.5, 0.5, 0.5),
            };
            
            name_row = name_row.push(
                container(
                    text("●")
                        .size(12)
                        .style(iced::theme::Text::Color(status_color))
                )
            );
        }
        
        content = content.push(name_row);
        
        // 详细信息行
        if self.show_details {
            let mut details_row = row![].spacing(15);
            
            // 提供商信息
            if item_config.show_provider {
                details_row = details_row.push(
                    text(format!("提供商: {}", domain.provider))
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                );
            }
            
            // DNS记录数量
            if item_config.show_record_count {
                let record_count = state.data.dns_records_cache
                    .get(&domain.domain_name)
                    .map(|records| records.len())
                    .unwrap_or(0);
                
                details_row = details_row.push(
                    text(format!("记录: {}", record_count))
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                );
            }
            
            // 最后同步时间
            if item_config.show_last_sync {
                let last_sync = state.data.domain_stats
                    .get(&domain.domain_name)
                    .and_then(|stats| stats.last_sync)
                    .map(|time| time.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "未同步".to_string());
                
                details_row = details_row.push(
                    text(format!("同步: {}", last_sync))
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                );
            }
            
            content = content.push(details_row);
        }
        
        // 包装在容器中
        let mut item_container = container(content)
            .padding(Padding::from([8, 12]))
            .width(Length::Fill);
        
        // 设置选中状态的样式
        if is_selected {
            item_container = item_container
                .style(iced::theme::Container::Custom(Box::new(SelectedItemStyle)));
        }
        
        // 如果可点击，包装在按钮中
        if item_config.clickable {
            button(item_container)
                .on_press(Message::DomainSelected(domain.domain_name.clone()))
                .style(iced::theme::Button::Text)
                .width(Length::Fill)
                .into()
        } else {
            item_container.into()
        }
    }
    
    /// 渲染空状态
    fn render_empty_state(&self) -> Element<Message> {
        container(
            column![
                text("暂无域名")
                    .size(18)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                text("点击添加按钮来添加您的第一个域名")
                    .size(14)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                button("添加域名")
                    .on_press(Message::PageChanged(crate::gui::Page::AddDomain))
                    .style(iced::theme::Button::Primary)
            ]
            .align_items(Alignment::Center)
            .spacing(10)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fixed(200.0))
        .into()
    }
    
    /// 渲染加载状态
    fn render_loading_state(&self) -> Element<Message> {
        container(
            column![
                text("正在加载域名...")
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                // 这里可以添加加载动画
            ]
            .align_items(Alignment::Center)
            .spacing(10)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fixed(100.0))
        .into()
    }
    
    /// 渲染错误状态
    fn render_error_state(&self, error: &str) -> Element<Message> {
        container(
            column![
                text("加载域名失败")
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.2, 0.2))),
                text(error)
                    .size(12)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                button("重试")
                    .on_press(Message::Reload)
                    .style(iced::theme::Button::Secondary)
            ]
            .align_items(Alignment::Center)
            .spacing(10)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fixed(150.0))
        .into()
    }
}

impl Component<AppState> for DomainListComponent {
    fn name(&self) -> &'static str {
        "domain_list"
    }
    
    fn view(&self, state: &AppState) -> Element<Message> {
        // 检查加载状态
        if state.ui.is_loading {
            return self.render_loading_state();
        }
        
        // 检查错误状态
        if let Some(error) = &state.ui.error_message {
            return self.render_error_state(error);
        }
        
        // 检查是否有域名
        if state.data.domains.is_empty() {
            return self.render_empty_state();
        }
        
        // 渲染域名列表
        let item_config = DomainListItemConfig::default();
        
        let domain_items: Vec<Element<Message>> = state.data.domains
            .iter()
            .map(|domain| {
                let is_selected = self.selected_domain
                    .as_ref()
                    .map(|selected| selected == &domain.domain_name)
                    .unwrap_or(false);
                
                self.render_domain_item(domain, is_selected, &item_config, state)
            })
            .collect();
        
        let content = column(domain_items)
            .spacing(2)
            .width(Length::Fill);
        
        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    fn update(&mut self, state: &mut AppState, message: Message) -> bool {
        match message {
            Message::DomainSelected(domain_name) => {
                self.selected_domain = Some(domain_name.clone());
                state.data.selected_domain = Some(domain_name);
                true
            },
            _ => false,
        }
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

/// 选中项样式
struct SelectedItemStyle;

impl iced::widget::container::StyleSheet for SelectedItemStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.2, 0.4, 0.8, 0.1))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.2, 0.4, 0.8),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}