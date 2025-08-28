//! DNS记录组件
//! 
//! 显示和管理DNS记录的可重用组件

use super::{Component, ComponentConfig, ComponentStyle};
use crate::gui::state::AppState;
use crate::gui::Message;
use crate::models::DnsRecord;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Padding};
use std::collections::HashMap;

/// DNS记录组件
#[derive(Debug, Clone)]
pub struct DnsRecordsComponent {
    config: ComponentConfig,
    selected_record: Option<String>,
    filter_type: Option<String>,
    search_query: String,
    show_details: bool,
    edit_mode: bool,
    editing_record: Option<DnsRecord>,
}

/// DNS记录显示配置
#[derive(Debug, Clone)]
pub struct DnsRecordDisplayConfig {
    pub show_type: bool,
    pub show_value: bool,
    pub show_ttl: bool,
    pub show_priority: bool,
    pub show_status: bool,
    pub show_actions: bool,
    pub editable: bool,
    pub deletable: bool,
}

impl Default for DnsRecordDisplayConfig {
    fn default() -> Self {
        Self {
            show_type: true,
            show_value: true,
            show_ttl: true,
            show_priority: true,
            show_status: true,
            show_actions: true,
            editable: true,
            deletable: true,
        }
    }
}

/// DNS记录类型过滤器
#[derive(Debug, Clone, PartialEq)]
pub enum DnsRecordFilter {
    All,
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
    SRV,
    PTR,
}

impl DnsRecordFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            DnsRecordFilter::All => "全部",
            DnsRecordFilter::A => "A",
            DnsRecordFilter::AAAA => "AAAA",
            DnsRecordFilter::CNAME => "CNAME",
            DnsRecordFilter::MX => "MX",
            DnsRecordFilter::TXT => "TXT",
            DnsRecordFilter::NS => "NS",
            DnsRecordFilter::SRV => "SRV",
            DnsRecordFilter::PTR => "PTR",
        }
    }
    
    pub fn matches(&self, record_type: &str) -> bool {
        match self {
            DnsRecordFilter::All => true,
            _ => self.as_str() == record_type,
        }
    }
}

impl DnsRecordsComponent {
    /// 创建新的DNS记录组件
    pub fn new() -> Self {
        Self {
            config: ComponentConfig::default(),
            selected_record: None,
            filter_type: None,
            search_query: String::new(),
            show_details: true,
            edit_mode: false,
            editing_record: None,
        }
    }
    
    /// 设置选中的记录
    pub fn set_selected_record(&mut self, record_id: Option<String>) {
        self.selected_record = record_id;
    }
    
    /// 设置过滤类型
    pub fn set_filter_type(&mut self, filter_type: Option<String>) {
        self.filter_type = filter_type;
    }
    
    /// 设置搜索查询
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }
    
    /// 开始编辑记录
    pub fn start_edit(&mut self, record: DnsRecord) {
        self.edit_mode = true;
        self.editing_record = Some(record);
    }
    
    /// 取消编辑
    pub fn cancel_edit(&mut self) {
        self.edit_mode = false;
        self.editing_record = None;
    }
    
    /// 过滤DNS记录
    fn filter_records(&self, records: &[DnsRecord]) -> Vec<&DnsRecord> {
        records
            .iter()
            .filter(|record| {
                // 类型过滤
                if let Some(filter_type) = &self.filter_type {
                    if &record.record_type != filter_type {
                        return false;
                    }
                }
                
                // 搜索过滤
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    let matches = record.name.to_lowercase().contains(&query)
                        || record.value.to_lowercase().contains(&query)
                        || record.record_type.to_lowercase().contains(&query);
                    
                    if !matches {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }
    
    /// 渲染过滤器栏
    fn render_filter_bar(&self) -> Element<Message> {
        let filters = vec![
            DnsRecordFilter::All,
            DnsRecordFilter::A,
            DnsRecordFilter::AAAA,
            DnsRecordFilter::CNAME,
            DnsRecordFilter::MX,
            DnsRecordFilter::TXT,
            DnsRecordFilter::NS,
        ];
        
        let filter_buttons: Vec<Element<Message>> = filters
            .into_iter()
            .map(|filter| {
                let is_active = match &self.filter_type {
                    Some(current) => current == filter.as_str(),
                    None => filter == DnsRecordFilter::All,
                };
                
                let filter_str = if filter == DnsRecordFilter::All {
                    None
                } else {
                    Some(filter.as_str().to_string())
                };
                
                button(text(filter.as_str()).size(12))
                    .on_press(Message::DnsFilterChanged(filter_str))
                    .style(if is_active {
                        iced::theme::Button::Primary
                    } else {
                        iced::theme::Button::Secondary
                    })
                    .into()
            })
            .collect();
        
        let search_input = text_input("搜索DNS记录...", &self.search_query)
            .on_input(Message::DnsSearchChanged)
            .width(Length::Fixed(200.0));
        
        row![
            row(filter_buttons).spacing(5),
            search_input,
            button("添加记录")
                .on_press(Message::DnsAddRecord)
                .style(iced::theme::Button::Primary)
        ]
        .align_items(Alignment::Center)
        .spacing(10)
        .padding(Padding::from([10, 0]))
        .into()
    }
    
    /// 渲染DNS记录项
    fn render_record_item(
        &self,
        record: &DnsRecord,
        is_selected: bool,
        config: &DnsRecordDisplayConfig,
    ) -> Element<Message> {
        let mut content = column![];
        
        // 主要信息行
        let mut main_row = row![
            text(&record.name)
                .size(14)
                .style(if is_selected {
                    iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.4, 0.8))
                } else {
                    iced::theme::Text::Default
                })
        ]
        .align_items(Alignment::Center)
        .spacing(10);
        
        // 记录类型
        if config.show_type {
            let type_color = match record.record_type.as_str() {
                "A" => iced::Color::from_rgb(0.2, 0.7, 0.3),
                "AAAA" => iced::Color::from_rgb(0.3, 0.6, 0.8),
                "CNAME" => iced::Color::from_rgb(0.8, 0.6, 0.2),
                "MX" => iced::Color::from_rgb(0.7, 0.3, 0.7),
                "TXT" => iced::Color::from_rgb(0.6, 0.4, 0.2),
                _ => iced::Color::from_rgb(0.5, 0.5, 0.5),
            };
            
            main_row = main_row.push(
                container(
                    text(&record.record_type)
                        .size(10)
                        .style(iced::theme::Text::Color(iced::Color::WHITE))
                )
                .padding(Padding::from([2, 6]))
                .style(iced::theme::Container::Custom(Box::new(TypeBadgeStyle { color: type_color })))
            );
        }
        
        // 状态指示器
        if config.show_status {
            let status_color = if record.enabled {
                iced::Color::from_rgb(0.2, 0.7, 0.3)
            } else {
                iced::Color::from_rgb(0.6, 0.6, 0.6)
            };
            
            main_row = main_row.push(
                text(if record.enabled { "●" } else { "○" })
                    .size(12)
                    .style(iced::theme::Text::Color(status_color))
            );
        }
        
        content = content.push(main_row);
        
        // 详细信息行
        if self.show_details {
            let mut details_row = row![].spacing(15);
            
            // 记录值
            if config.show_value {
                let value_text = if record.value.len() > 50 {
                    format!("{}...", &record.value[..47])
                } else {
                    record.value.clone()
                };
                
                details_row = details_row.push(
                    text(format!("值: {}", value_text))
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                );
            }
            
            // TTL
            if config.show_ttl {
                details_row = details_row.push(
                    text(format!("TTL: {}", record.ttl))
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                );
            }
            
            // 优先级（MX记录）
            if config.show_priority && record.record_type == "MX" {
                if let Some(priority) = record.priority {
                    details_row = details_row.push(
                        text(format!("优先级: {}", priority))
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                    );
                }
            }
            
            content = content.push(details_row);
        }
        
        // 操作按钮行
        if config.show_actions {
            let mut actions_row = row![].spacing(5);
            
            if config.editable {
                actions_row = actions_row.push(
                    button("编辑")
                        .on_press(Message::DnsEditRecord(record.id.clone()))
                        .style(iced::theme::Button::Secondary)
                );
            }
            
            if config.deletable {
                actions_row = actions_row.push(
                    button("删除")
                        .on_press(Message::DnsDeleteRecord(record.id.clone()))
                        .style(iced::theme::Button::Destructive)
                );
            }
            
            actions_row = actions_row.push(
                button(if record.enabled { "禁用" } else { "启用" })
                    .on_press(Message::DnsToggleRecord(record.id.clone()))
                    .style(iced::theme::Button::Secondary)
            );
            
            content = content.push(actions_row);
        }
        
        // 包装在容器中
        let mut item_container = container(content)
            .padding(Padding::from([10, 12]))
            .width(Length::Fill);
        
        // 设置选中状态的样式
        if is_selected {
            item_container = item_container
                .style(iced::theme::Container::Custom(Box::new(SelectedRecordStyle)));
        }
        
        button(item_container)
            .on_press(Message::DnsRecordSelected(record.id.clone()))
            .style(iced::theme::Button::Text)
            .width(Length::Fill)
            .into()
    }
    
    /// 渲染空状态
    fn render_empty_state(&self, domain: Option<&str>) -> Element<Message> {
        let message = if let Some(domain) = domain {
            format!("域名 {} 暂无DNS记录", domain)
        } else {
            "请先选择一个域名".to_string()
        };
        
        container(
            column![
                text(message)
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                text("点击添加按钮来添加DNS记录")
                    .size(12)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                button("添加DNS记录")
                    .on_press(Message::DnsAddRecord)
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
            text("正在加载DNS记录...")
                .size(16)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fixed(100.0))
        .into()
    }
}

impl Component<AppState> for DnsRecordsComponent {
    fn name(&self) -> &'static str {
        "dns_records"
    }
    
    fn view(&self, state: &AppState) -> Element<Message> {
        let mut content = column![];
        
        // 添加过滤器栏
        content = content.push(self.render_filter_bar());
        
        // 检查是否选择了域名
        let selected_domain = match &state.data.selected_domain {
            Some(domain) => domain,
            None => {
                content = content.push(self.render_empty_state(None));
                return container(content).into();
            }
        };
        
        // 检查加载状态
        if state.ui.is_loading {
            content = content.push(self.render_loading_state());
            return container(content).into();
        }
        
        // 获取DNS记录
        let records = state.data.dns_records_cache
            .get(selected_domain)
            .map(|records| records.as_slice())
            .unwrap_or(&[]);
        
        // 检查是否有记录
        if records.is_empty() {
            content = content.push(self.render_empty_state(Some(selected_domain)));
            return container(content).into();
        }
        
        // 过滤记录
        let filtered_records = self.filter_records(records);
        
        if filtered_records.is_empty() {
            content = content.push(
                container(
                    text("没有匹配的DNS记录")
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                )
                .center_x()
                .padding(Padding::from([20, 0]))
            );
            return container(content).into();
        }
        
        // 渲染记录列表
        let display_config = DnsRecordDisplayConfig::default();
        
        let record_items: Vec<Element<Message>> = filtered_records
            .into_iter()
            .map(|record| {
                let is_selected = self.selected_record
                    .as_ref()
                    .map(|selected| selected == &record.id)
                    .unwrap_or(false);
                
                self.render_record_item(record, is_selected, &display_config)
            })
            .collect();
        
        let records_list = column(record_items)
            .spacing(2)
            .width(Length::Fill);
        
        content = content.push(
            scrollable(records_list)
                .width(Length::Fill)
                .height(Length::Fill)
        );
        
        container(content).into()
    }
    
    fn update(&mut self, state: &mut AppState, message: Message) -> bool {
        match message {
            Message::DnsRecordSelected(record_id) => {
                self.selected_record = Some(record_id);
                true
            },
            Message::DnsFilterChanged(filter_type) => {
                self.filter_type = filter_type;
                true
            },
            Message::DnsSearchChanged(query) => {
                self.search_query = query;
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

impl Default for DnsRecordsComponent {
    fn default() -> Self {
        Self::new()
    }
}

/// 类型标签样式
struct TypeBadgeStyle {
    color: iced::Color,
}

impl iced::widget::container::StyleSheet for TypeBadgeStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(self.color)),
            border: iced::Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// 选中记录样式
struct SelectedRecordStyle;

impl iced::widget::container::StyleSheet for SelectedRecordStyle {
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