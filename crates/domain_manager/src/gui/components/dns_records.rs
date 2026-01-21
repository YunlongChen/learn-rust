//! DNS记录组件
//!
//! 显示和管理DNS记录的可重用组件

use super::{Component, ComponentConfig, State};
use crate::gui::handlers::message_handler::NavigationMessage::PageChanged;
use crate::gui::handlers::message_handler::{DnsMessage, MessageCategory, NavigationMessage};
use crate::gui::model::domain::DnsRecord;
use crate::gui::pages::Page;
use crate::gui::state::AppState;
use crate::gui::styles::ContainerType;
use crate::storage::DnsRecordModal;
use crate::StyleType;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length, Padding};

/// DNS记录组件
#[derive(Debug, Clone)]
pub struct DnsRecordsComponent {
    config: ComponentConfig,
    selected_record: Option<String>,
    filter_type: Option<String>,
    search_query: String,
    show_details: bool,
    edit_mode: bool,
    editing_record: Option<DnsRecordModal>,
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
    pub fn start_edit(&mut self, record: DnsRecordModal) {
        self.edit_mode = true;
        self.editing_record = Some(record);
    }

    /// 取消编辑
    pub fn cancel_edit(&mut self) {
        self.edit_mode = false;
        self.editing_record = None;
    }

    /// 过滤DNS记录
    fn filter_records<'a>(&self, records: &'a [DnsRecordModal]) -> Vec<&'a DnsRecordModal> {
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
    fn render_filter_bar(&self, _state: &State) -> Element<'_, MessageCategory, StyleType> {
        let filters = vec![
            DnsRecordFilter::All,
            DnsRecordFilter::A,
            DnsRecordFilter::AAAA,
            DnsRecordFilter::CNAME,
            DnsRecordFilter::MX,
            DnsRecordFilter::TXT,
            DnsRecordFilter::NS,
        ];

        let filter_buttons: Vec<Element<MessageCategory, StyleType>> = filters
            .into_iter()
            .map(|filter| {
                let _is_active = match &self.filter_type {
                    Some(current) => current == filter.as_str(),
                    None => filter == DnsRecordFilter::All,
                };

                let filter_str = if filter == DnsRecordFilter::All {
                    None
                } else {
                    Some(filter.as_str().to_string())
                };

                iced::widget::Button::<'_, MessageCategory, StyleType>::new(
                    iced::widget::Text::<'_, StyleType>::new(filter.as_str()).size(12),
                )
                .on_press(MessageCategory::Dns(DnsMessage::DnsFilterChanged(
                    filter_str,
                )))
                .into()
            })
            .collect();

        let search_input = iced::widget::TextInput::<'_, MessageCategory, StyleType>::new(
            "搜索DNS记录...",
            &self.search_query,
        )
        .on_input(|string: String| MessageCategory::Dns(DnsMessage::DnsSearchChanged(string)))
        .width(Length::Fixed(200.0));

        iced::widget::Row::<'_, MessageCategory, StyleType>::new()
            .push(
                iced::widget::Row::<'_, MessageCategory, StyleType>::with_children(filter_buttons)
                    .spacing(5),
            )
            .push(search_input)
            .push(
                iced::widget::Button::<'_, MessageCategory, StyleType>::new(iced::widget::Text::<
                    '_,
                    StyleType,
                >::new(
                    "添加记录"
                ))
                .on_press(MessageCategory::Navigation(
                    NavigationMessage::PageChanged(Page::AddRecord),
                )),
            )
            .align_y(Alignment::Center)
            .spacing(10)
            .padding(Padding::from([10, 0]))
            .into()
    }

    /// 渲染DNS记录项
    fn render_record_item_simple<'a>(
        &'a self,
        record: &'a DnsRecordModal,
        is_selected: bool,
        index: usize,
    ) -> Element<'a, MessageCategory, StyleType> {
        let mut content = iced::widget::Column::<'_, MessageCategory, StyleType>::new();

        // 主要信息行
        let mut main_row = iced::widget::Row::<'_, MessageCategory, StyleType>::new()
            .push(iced::widget::Text::<'_, StyleType>::new(&record.name).size(14))
            .align_y(Alignment::Center)
            .spacing(10);

        // 记录类型
        let _type_color = match record.record_type.as_str() {
            "A" => iced::Color::from_rgb(0.2, 0.7, 0.3),
            "AAAA" => iced::Color::from_rgb(0.3, 0.6, 0.8),
            "CNAME" => iced::Color::from_rgb(0.8, 0.6, 0.2),
            "MX" => iced::Color::from_rgb(0.7, 0.3, 0.7),
            "TXT" => iced::Color::from_rgb(0.6, 0.4, 0.2),
            _ => iced::Color::from_rgb(0.5, 0.5, 0.5),
        };

        main_row = main_row.push(
            iced::widget::Container::<'_, MessageCategory, StyleType>::new(
                iced::widget::Text::<'_, StyleType>::new(&record.record_type).size(10),
            )
            .padding(Padding::from([2, 6]))
            .class(ContainerType::Hoverable),
        );

        // 记录值
        main_row = main_row.push(iced::widget::Text::<'_, StyleType>::new(&record.value).size(12));

        // TTL
        main_row = main_row.push(
            iced::widget::Text::<'_, StyleType>::new(format!("TTL: {}", record.ttl)).size(10),
        );

        content = content.push(main_row);

        // 操作按钮
        let actions = iced::widget::Row::<'_, MessageCategory, StyleType>::new()
            .push(
                iced::widget::Button::<'_, MessageCategory, StyleType>::new(
                    iced::widget::Text::<'_, StyleType>::new("编辑").size(10),
                )
                .padding(Padding::from([4, 8]))
                .on_press(MessageCategory::Navigation(
                    NavigationMessage::PageChanged(Page::AddRecord),
                )),
            )
            .push(
                iced::widget::Button::<'_, MessageCategory, StyleType>::new(
                    iced::widget::Text::<'_, StyleType>::new("删除").size(10),
                )
                .padding(Padding::from([4, 8]))
                .on_press(MessageCategory::Dns(DnsMessage::Delete(index))),
            )
            .spacing(5);

        content = content.push(actions);

        iced::widget::Container::<'_, MessageCategory, StyleType>::new(content)
            .padding(10)
            .class(if is_selected {
                ContainerType::Selected
            } else {
                ContainerType::Hoverable
            })
            .into()
    }

    fn render_record_item<'a>(
        &'a self,
        record: &'a DnsRecordModal,
        is_selected: bool,
        config: &'a DnsRecordDisplayConfig,
    ) -> Element<'a, MessageCategory, StyleType> {
        let mut content = iced::widget::Column::<'_, MessageCategory, StyleType>::new();

        // 主要信息行
        let mut main_row = row![text(&record.name).size(14)]
            .align_y(Alignment::Center)
            .spacing(10);

        // 记录类型
        if config.show_type {
            let _type_color = match record.record_type.as_str() {
                "A" => iced::Color::from_rgb(0.2, 0.7, 0.3),
                "AAAA" => iced::Color::from_rgb(0.3, 0.6, 0.8),
                "CNAME" => iced::Color::from_rgb(0.8, 0.6, 0.2),
                "MX" => iced::Color::from_rgb(0.7, 0.3, 0.7),
                "TXT" => iced::Color::from_rgb(0.6, 0.4, 0.2),
                _ => iced::Color::from_rgb(0.5, 0.5, 0.5),
            };

            main_row = main_row
                .push(container(text(&record.record_type).size(10)).padding(Padding::from([2, 6])));
        }

        // 状态指示器
        if config.show_status {
            let _status_color = if record.enabled {
                iced::Color::from_rgb(0.2, 0.7, 0.3)
            } else {
                iced::Color::from_rgb(0.6, 0.6, 0.6)
            };

            main_row = main_row.push(text(if record.enabled { "●" } else { "○" }).size(12));
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

                details_row = details_row.push(text(format!("值: {}", value_text)).size(12));
            }

            // TTL
            if config.show_ttl {
                details_row = details_row.push(text(format!("TTL: {}", record.ttl)).size(12));
            }

            // 优先级（MX记录）
            if config.show_priority && record.record_type == "MX" {
                if let Some(priority) = record.priority {
                    details_row = details_row.push(text(format!("优先级: {}", priority)).size(12));
                }
            }

            content = content.push(details_row);
        }

        // 操作按钮行
        if config.show_actions {
            let mut actions_row = row![].spacing(5);

            if config.editable {
                actions_row = actions_row.push(button("编辑").on_press(
                    MessageCategory::Navigation(PageChanged(Page::EditRecord(DnsRecord {
                        name: "".to_string(),
                        record_type: "".to_string(),
                        value: "".to_string(),
                        ttl: "".to_string(),
                    }))),
                ));
            }

            if config.deletable {
                actions_row = actions_row.push(
                    button("删除")
                        .on_press(MessageCategory::Dns(DnsMessage::Delete(record.id as usize))),
                );
            }

            actions_row = actions_row.push(
                button(if record.enabled { "禁用" } else { "启用" }).on_press(
                    MessageCategory::Dns(DnsMessage::DnsToggleRecord(record.id as usize)),
                ),
            );

            content = content.push(actions_row);
        }

        // 包装在容器中
        let mut item_container = container(content)
            .padding(Padding::from([10, 12]))
            .width(Length::Fill);

        // 设置选中状态的样式
        if is_selected {
            item_container = item_container;
        }

        button(item_container)
            .on_press(MessageCategory::Dns(DnsMessage::DnsRecordSelected(
                record.id as usize,
            )))
            .width(Length::Fill)
            .into()
    }

    /// 渲染空状态
    fn render_empty_state(&self, domain: Option<&str>) -> Element<'_, MessageCategory, StyleType> {
        let message = if let Some(domain) = domain {
            format!("域名 {} 暂无DNS记录", domain)
        } else {
            "请先选择一个域名".to_string()
        };

        container(
            column![
                text(message).size(16),
                text("点击添加按钮来添加DNS记录").size(12),
                button("添加DNS记录").on_press(MessageCategory::Navigation(
                    NavigationMessage::PageChanged(Page::AddRecord)
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
        container(text("正在加载DNS记录...").size(16))
            .width(Length::Fill)
            .height(Length::Fixed(100.0))
            .into()
    }
}

impl Component<AppState> for DnsRecordsComponent {
    fn name(&self) -> &'static str {
        "dns_records"
    }

    fn view<'a>(&'a self, state: &'a State) -> Element<'a, MessageCategory, StyleType> {
        let mut content = iced::widget::Column::<'_, MessageCategory, StyleType>::new();

        // 添加过滤器栏
        content = content.push(self.render_filter_bar(state));

        // 检查是否选择了域名
        let selected_domain = match &state.data.selected_domain {
            Some(domain) => domain,
            None => {
                content = content.push(self.render_empty_state(None));
                return iced::widget::Container::<'_, MessageCategory, StyleType>::new(content)
                    .into();
            }
        };

        // 检查加载状态
        if state.ui.is_loading {
            content = content.push(self.render_loading_state());
            return iced::widget::Container::<'_, MessageCategory, StyleType>::new(content).into();
        }

        // 获取DNS记录
        let records = state
            .data
            .dns_records_cache
            .get(&(selected_domain.id as usize))
            .map(|records| records.as_slice())
            .unwrap_or(&[]);

        // 检查是否有记录
        if records.is_empty() {
            let domain_name = selected_domain.name.clone();
            content = content.push(self.render_empty_state(Some(&domain_name)));
            return iced::widget::Container::<'_, MessageCategory, StyleType>::new(content).into();
        }

        // 过滤记录
        let filtered_records = self.filter_records(records);

        if filtered_records.is_empty() {
            content = content.push(
                iced::widget::Container::<'_, MessageCategory, StyleType>::new(
                    iced::widget::Text::<'_, StyleType>::new("没有匹配的DNS记录").size(14),
                )
                .center_x(iced::Length::Fill)
                .padding(Padding::from([20, 0])),
            );
            return iced::widget::Container::<'_, MessageCategory, StyleType>::new(content).into();
        }

        // 渲染记录列表
        let record_items: Vec<Element<MessageCategory, StyleType>> = filtered_records
            .into_iter()
            .enumerate()
            .map(|(index, record)| {
                let is_selected = self
                    .selected_record
                    .as_ref()
                    .map(|selected| selected == &record.id.to_string())
                    .unwrap_or(false);
                self.render_record_item_simple(record, is_selected, index)
            })
            .collect();

        let records_list =
            iced::widget::Column::<'_, MessageCategory, StyleType>::with_children(record_items)
                .spacing(2)
                .width(Length::Fill);

        content = content.push(
            iced::widget::Scrollable::<'_, MessageCategory, StyleType>::new(records_list)
                .width(Length::Fill)
                .height(Length::Fill),
        );

        iced::widget::Container::<'_, MessageCategory, StyleType>::new(content).into()
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

impl iced::widget::container::Catalog for TypeBadgeStyle {
    type Class<'a> = iced::Theme;

    fn default<'a>() -> Self::Class<'a> {
        <iced::Theme as std::default::Default>::default()
    }

    fn style(&self, _class: &Self::Class<'_>) -> iced::widget::container::Style {
        iced::widget::container::Style {
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

impl container::Catalog for SelectedRecordStyle {
    type Class<'a> = iced::Theme;

    fn default<'a>() -> Self::Class<'a> {
        <iced::Theme as Default>::default()
    }

    fn style(&self, _class: &Self::Class<'_>) -> container::Style {
        container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.2, 0.4, 0.8, 0.1,
            ))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.2, 0.4, 0.8),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}
