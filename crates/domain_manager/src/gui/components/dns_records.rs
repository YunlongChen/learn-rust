//! DNS记录组件
//!
//! 显示和管理DNS记录的可重用组件

use super::{Component, ComponentConfig, State};
use crate::gui::handlers::message_handler::NavigationMessage::PageChanged;
use crate::gui::handlers::message_handler::{DnsMessage, MessageCategory, NavigationMessage};
use crate::gui::model::domain::DnsRecord;
use crate::gui::pages::Page;
use crate::gui::state::AppState;
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::ContainerType as ContainerClass;
use crate::model::dns_record_response::Type as RecordType;
use crate::storage::DnsRecordModal;
use crate::StyleType;
use iced::widget::{button, column, container, mouse_area, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length, Padding};

/// DNS记录组件
#[derive(Debug, Clone)]
pub struct DnsRecordsComponent {
    config: ComponentConfig,
    selected_record: Option<String>,
    show_details: bool,
    edit_mode: bool,
    editing_record: Option<DnsRecordModal>,
    is_adding: bool, // 新增：控制是否显示添加表单
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
            show_details: true,
            edit_mode: false,
            editing_record: None,
            is_adding: false,
        }
    }

    /// 切换添加模式
    pub fn toggle_add_mode(&mut self) {
        self.is_adding = !self.is_adding;
    }

    /// 设置选中的记录
    pub fn set_selected_record(&mut self, record_id: Option<String>) {
        self.selected_record = record_id;
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
    fn filter_records<'a>(
        &self,
        records: &'a [DnsRecordModal],
        filter: &crate::gui::state::data_state::Filter,
    ) -> Vec<&'a DnsRecordModal> {
        records
            .iter()
            .filter(|record| {
                // 类型过滤
                if let Some(filter_type) = &filter.record_type {
                    if &record.record_type != filter_type {
                        return false;
                    }
                }

                // 搜索过滤
                if !filter.search_keyword.is_empty() {
                    let query = filter.search_keyword.to_lowercase();
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

    /// 渲染内联添加表单
    fn render_add_form<'a>(&'a self, state: &'a State) -> Element<'a, MessageCategory, StyleType> {
        let form_state = &state.data.add_dns_form;
        let is_edit = form_state.record_id.is_some();

        let record_types = vec![
            RecordType::A,
            RecordType::AAAA,
            RecordType::Cname,
            RecordType::MX,
            RecordType::TXT,
            RecordType::NS,
            RecordType::SRV,
            RecordType::PTR,
            RecordType::SOA,
        ];

        let content = column![
            text(if is_edit {
                "编辑DNS记录"
            } else {
                "添加DNS记录"
            })
            .size(14),
            row![
                // 记录类型
                column![
                    text("类型").size(12),
                    pick_list(record_types, form_state.record_type.clone(), |t| {
                        MessageCategory::Dns(DnsMessage::FormRecordTypeChanged(t))
                    })
                    .width(Length::Fixed(100.0))
                ]
                .spacing(5),
                // 主机记录
                column![
                    text("主机记录").size(12),
                    text_input("如 www", &form_state.record_name)
                        .on_input(|s| MessageCategory::Dns(DnsMessage::FormNameChanged(s)))
                        .width(Length::Fixed(150.0))
                ]
                .spacing(5),
                // 记录值
                column![
                    text("记录值").size(12),
                    text_input("如 1.2.3.4", &form_state.value)
                        .on_input(|s| MessageCategory::Dns(DnsMessage::FormValueChanged(s)))
                        .width(Length::Fill)
                ]
                .spacing(5),
                // TTL
                column![
                    text("TTL").size(12),
                    text_input("600", &form_state.ttl.to_string())
                        .on_input(|s| {
                            if let Ok(ttl) = s.parse::<i32>() {
                                MessageCategory::Dns(DnsMessage::FormTtlChanged(ttl))
                            } else if s.is_empty() {
                                MessageCategory::Dns(DnsMessage::FormTtlChanged(0))
                            } else {
                                MessageCategory::Dns(DnsMessage::FormTtlChanged(form_state.ttl))
                            }
                        })
                        .width(Length::Fixed(80.0))
                ]
                .spacing(5),
            ]
            .spacing(10)
            .align_y(Alignment::End),
            // 操作按钮
            row![
                button(if is_edit { "更新" } else { "保存" })
                    .on_press(MessageCategory::Dns(DnsMessage::FormSubmit))
                    .class(ButtonType::Primary),
                button("取消")
                    .on_press(MessageCategory::Dns(DnsMessage::FormCancelled))
                    .class(ButtonType::Standard),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        ]
        .spacing(15)
        .padding(15);

        container(content)
            .class(ContainerType::BorderedRound)
            .width(Length::Fill)
            .into()
    }

    /// 渲染过滤器栏
    fn render_filter_bar(&self, state: &State) -> Element<'_, MessageCategory, StyleType> {
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
                let is_active = match &state.data.dns_record_filter.record_type {
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
                .class(if is_active {
                    ButtonType::Primary
                } else {
                    ButtonType::Standard
                })
                .into()
            })
            .collect();

        let search_input = iced::widget::TextInput::<'_, MessageCategory, StyleType>::new(
            "搜索DNS记录...",
            &state.data.dns_record_filter.search_keyword,
        )
        .on_input(|string: String| MessageCategory::Dns(DnsMessage::DnsSearchChanged(string)))
        .width(Length::Fixed(200.0));

        let mut row = iced::widget::Row::<'_, MessageCategory, StyleType>::new()
            .push(
                iced::widget::Row::<'_, MessageCategory, StyleType>::with_children(filter_buttons)
                    .spacing(5),
            )
            .push(search_input);

        // 仅当选中域名时才显示添加按钮
        if state.data.selected_domain.is_some() {
            let is_visible = state.data.add_dns_form.is_visible;
            row =
                row.push(
                    iced::widget::Button::<'_, MessageCategory, StyleType>::new(
                        iced::widget::Text::<'_, StyleType>::new(if is_visible {
                            "隐藏表单"
                        } else {
                            "添加记录"
                        }),
                    )
                    .on_press(MessageCategory::Dns(DnsMessage::ProviderSelected(99999))), // 临时使用此消息触发切换
                );
        }

        row.align_y(Alignment::Center)
            .spacing(10)
            .padding(Padding::from([10, 0]))
            .into()
    }

    /// 渲染DNS记录项
    fn render_record_item_simple<'a>(
        &'a self,
        state: &'a State,
        record: &'a DnsRecordModal,
        is_selected: bool,
        _index: usize,
    ) -> Element<'a, MessageCategory, StyleType> {
        let is_hovered = state.ui.hovered_dns_record == Some(record.id as usize);

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

        // 推送间隔，将按钮推到右边
        main_row = main_row.push(iced::widget::Space::with_width(Length::Fill));

        // 操作按钮 (仅在悬停或选中时显示)
        if is_hovered || is_selected {
            let mut actions = iced::widget::Row::<'_, MessageCategory, StyleType>::new()
                .spacing(5)
                .height(Length::Shrink);

            // 检查是否正在删除
            let is_deleting = state.data.deleting_dns_record_id == Some(record.id as usize);

            if is_deleting {
                actions = actions
                    .push(
                        iced::widget::Button::<'_, MessageCategory, StyleType>::new(
                            iced::widget::Text::<'_, StyleType>::new("确认删除?").size(10),
                        )
                        .padding(Padding::from([2, 8]))
                        .on_press(MessageCategory::Dns(DnsMessage::Delete(record.id as usize)))
                        .class(ButtonType::Alert),
                    )
                    .push(
                        iced::widget::Button::<'_, MessageCategory, StyleType>::new(
                            iced::widget::Text::<'_, StyleType>::new("取消").size(10),
                        )
                        .padding(Padding::from([2, 8]))
                        .on_press(MessageCategory::Dns(DnsMessage::DeleteCancel)),
                    );
            } else {
                actions = actions
                    .push(
                        iced::widget::Button::<'_, MessageCategory, StyleType>::new(
                            iced::widget::Text::<'_, StyleType>::new("编辑").size(10),
                        )
                        .padding(Padding::from([2, 8]))
                        .on_press(MessageCategory::Dns(DnsMessage::EditRecord(record.clone()))),
                    )
                    .push(
                        iced::widget::Button::<'_, MessageCategory, StyleType>::new(
                            iced::widget::Text::<'_, StyleType>::new("删除").size(10),
                        )
                        .padding(Padding::from([2, 8]))
                        .on_press(MessageCategory::Dns(
                            DnsMessage::DeleteRequest(record.id as usize),
                        )),
                    );
            }

            main_row = main_row.push(actions);
        }

        let item_container =
            iced::widget::Container::<'_, MessageCategory, StyleType>::new(main_row)
                .padding(10)
                .width(Length::Fill)
                .class(if is_selected {
                    ContainerType::Selected
                } else {
                    ContainerType::Hoverable
                });

        mouse_area(
            button(item_container)
                .on_press(MessageCategory::Dns(DnsMessage::DnsRecordSelected(
                    record.id as usize,
                )))
                .width(Length::Fill)
                .class(ButtonType::Transparent),
        )
        .on_enter(MessageCategory::Dns(DnsMessage::RecordHovered(Some(
            record.id as usize,
        ))))
        .on_exit(MessageCategory::Dns(DnsMessage::RecordHovered(None)))
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

        // 如果处于添加模式，显示添加表单
        if state.data.add_dns_form.is_visible {
            content = content.push(self.render_add_form(state));
        }

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
        let filtered_records = self.filter_records(records, &state.data.dns_record_filter);

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
                self.render_record_item_simple(state, record, is_selected, index)
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
