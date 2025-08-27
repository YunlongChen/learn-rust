//! 控制台界面组件 - 显示API请求和数据库查询日志

use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::text::TextType;
use crate::gui::types::message::Message;
use crate::StyleType;
use iced::widget::{
    button, column, container, row, scrollable, text, Column, Container, Row, Scrollable, Text,
};
use iced::{Alignment, Element, Font, Length};
use std::collections::VecDeque;

/// 控制台标签页类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleTab {
    ApiRequests,
    DatabaseQueries,
}

/// API请求日志条目
#[derive(Debug, Clone)]
pub struct ApiRequestLog {
    pub timestamp: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub duration: String,
}

/// 数据库查询日志条目
#[derive(Debug, Clone)]
pub struct DatabaseQueryLog {
    pub timestamp: String,
    pub query: String,
    pub duration: String,
    pub rows_affected: Option<u64>,
}

/// 控制台状态
#[derive(Debug, Clone)]
pub struct ConsoleState {
    pub current_tab: ConsoleTab,
    pub api_logs: VecDeque<ApiRequestLog>,
    pub db_logs: VecDeque<DatabaseQueryLog>,
    pub max_logs: usize,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            current_tab: ConsoleTab::ApiRequests,
            api_logs: VecDeque::new(),
            db_logs: VecDeque::new(),
            max_logs: 1000,
        }
    }
}

impl ConsoleState {
    /// 添加API请求日志
    pub fn add_api_log(&mut self, log: ApiRequestLog) {
        if self.api_logs.len() >= self.max_logs {
            self.api_logs.pop_front();
        }
        self.api_logs.push_back(log);
    }

    /// 添加数据库查询日志
    pub fn add_db_log(&mut self, log: DatabaseQueryLog) {
        if self.db_logs.len() >= self.max_logs {
            self.db_logs.pop_front();
        }
        self.db_logs.push_back(log);
    }

    /// 清空所有日志
    pub fn clear_logs(&mut self) {
        self.api_logs.clear();
        self.db_logs.clear();
    }
}

/// 创建控制台界面
pub fn console_view<'a>(
    console_state: &ConsoleState,
    font: Font,
) -> Container<'a, Message, StyleType> {
    let header = create_console_header(&console_state.current_tab, font);
    
    let content = match console_state.current_tab {
        ConsoleTab::ApiRequests => create_api_logs_view(&console_state.api_logs, font),
        ConsoleTab::DatabaseQueries => create_db_logs_view(&console_state.db_logs, font),
    };

    Container::new(
        Column::new()
            .spacing(10)
            .push(header)
            .push(content)
            .padding(20)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .class(ContainerType::Standard)
}

/// 创建控制台头部（标签页切换）
fn create_console_header<'a>(
    current_tab: &ConsoleTab,
    font: Font,
) -> Row<'a, Message, StyleType> {
    Row::new()
        .spacing(10)
        .push(
            button(
                Text::new("API请求")
                    .font(font)
                    .size(14)
            )
            .class(if *current_tab == ConsoleTab::ApiRequests {
                ButtonType::TabActive
            } else {
                ButtonType::TabInactive
            })
            .on_press(Message::ChangeConsoleTab(ConsoleTab::ApiRequests))
            .padding([8, 16])
        )
        .push(
            button(
                Text::new("数据库查询")
                    .font(font)
                    .size(14)
            )
            .class(if *current_tab == ConsoleTab::DatabaseQueries {
                ButtonType::TabActive
            } else {
                ButtonType::TabInactive
            })
            .on_press(Message::ChangeConsoleTab(ConsoleTab::DatabaseQueries))
            .padding([8, 16])
        )
        .push(
            button(
                Text::new("清空日志")
                    .font(font)
                    .size(14)
            )
            .class(ButtonType::Alert)
            .on_press(Message::ClearConsoleLogs)
            .padding([8, 16])
        )
        .align_y(Alignment::Center)
}

/// 创建API请求日志视图
fn create_api_logs_view<'a>(
    logs: &VecDeque<ApiRequestLog>,
    font: Font,
) -> Container<'a, Message, StyleType> {
    let mut content = Column::new().spacing(5);

    if logs.is_empty() {
        content = content.push(
            Container::new(
                Text::new("暂无API请求日志")
                    .font(font)
                    .size(16)
                    .class(TextType::Subtitle)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
        );
    } else {
        // 添加表头
        content = content.push(
            Row::new()
                .spacing(10)
                .push(
                    Text::new("时间")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fixed(150.0))
                )
                .push(
                    Text::new("方法")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fixed(80.0))
                )
                .push(
                    Text::new("URL")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fill)
                )
                .push(
                    Text::new("状态")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fixed(80.0))
                )
                .push(
                    Text::new("耗时")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fixed(100.0))
                )
                .padding([10, 15])
        );

        // 添加日志条目
        for log in logs.iter().rev() {
            content = content.push(
                Container::new(
                    Row::new()
                        .spacing(10)
                        .push(
                            Text::new(&log.timestamp)
                                .font(font)
                                .size(11)
                                .width(Length::Fixed(150.0))
                        )
                        .push(
                            Text::new(&log.method)
                                .font(font)
                                .size(11)
                                .class(match log.method.as_str() {
                                    "GET" => TextType::Standard,
                                    "POST" => TextType::Incoming,
                                    "PUT" => TextType::Outgoing,
                                    "DELETE" => TextType::Danger,
                                    _ => TextType::Standard,
                                })
                                .width(Length::Fixed(80.0))
                        )
                        .push(
                            Text::new(&log.url)
                                .font(font)
                                .size(11)
                                .width(Length::Fill)
                        )
                        .push(
                            Text::new(log.status.to_string())
                                .font(font)
                                .size(11)
                                .class(if log.status >= 400 {
                                    TextType::Danger
                                } else if log.status >= 300 {
                                    TextType::Outgoing
                                } else {
                                    TextType::Incoming
                                })
                                .width(Length::Fixed(80.0))
                        )
                        .push(
                            Text::new(&log.duration)
                                .font(font)
                                .size(11)
                                .width(Length::Fixed(100.0))
                        )
                        .padding([8, 15])
                        .align_y(Alignment::Center)
                )
                .class(ContainerType::Hoverable)
                .width(Length::Fill)
            );
        }
    }

    Container::new(
        Scrollable::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .class(ContainerType::BorderedRound)
}

/// 创建数据库查询日志视图
fn create_db_logs_view<'a>(
    logs: &VecDeque<DatabaseQueryLog>,
    font: Font,
) -> Container<'a, Message, StyleType> {
    let mut content = Column::new().spacing(5);

    if logs.is_empty() {
        content = content.push(
            Container::new(
                Text::new("暂无数据库查询日志")
                    .font(font)
                    .size(16)
                    .class(TextType::Subtitle)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
        );
    } else {
        // 添加表头
        content = content.push(
            Row::new()
                .spacing(10)
                .push(
                    Text::new("时间")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fixed(150.0))
                )
                .push(
                    Text::new("查询语句")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fill)
                )
                .push(
                    Text::new("耗时")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fixed(100.0))
                )
                .push(
                    Text::new("影响行数")
                        .font(font)
                        .size(12)
                        .class(TextType::Subtitle)
                        .width(Length::Fixed(100.0))
                )
                .padding([10, 15])
        );

        // 添加日志条目
        for log in logs.iter().rev() {
            content = content.push(
                Container::new(
                    Row::new()
                        .spacing(10)
                        .push(
                            Text::new(&log.timestamp)
                                .font(font)
                                .size(11)
                                .width(Length::Fixed(150.0))
                        )
                        .push(
                            Text::new(&log.query)
                                .font(font)
                                .size(11)
                                .width(Length::Fill)
                        )
                        .push(
                            Text::new(&log.duration)
                                .font(font)
                                .size(11)
                                .width(Length::Fixed(100.0))
                        )
                        .push(
                            Text::new(
                                log.rows_affected
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| "-".to_string())
                            )
                            .font(font)
                            .size(11)
                            .width(Length::Fixed(100.0))
                        )
                        .padding([8, 15])
                        .align_y(Alignment::Center)
                )
                .class(ContainerType::Hoverable)
                .width(Length::Fill)
            );
        }
    }

    Container::new(
        Scrollable::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .class(ContainerType::BorderedRound)
}