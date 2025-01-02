use crate::model::dns_operate::RecordLog;
use crate::model::dns_record_response::{Record, Type};
use crate::{get_text, App, Message, Page};
use iced::border::Radius;
use iced::widget::button::{danger, primary};
use iced::widget::text::{LineHeight, Wrapping};
use iced::widget::{
    button, container, pick_list, row, slider, text, text_input, Column, Container, Row, Text,
};
use iced::{Alignment, Border, Color, Element, Length, Padding, Theme};

pub fn dns_record(app: &App) -> Element<'static, Message> {
    // 展示dns列表
    let page: Element<'static, Message> = match &app.current_domain_name {
        // 选中了域名
        Some(domain_name) => {
            // 返回到解析界面
            let dns_content: Column<Message> =
                Column::from_iter(app.dns_list.iter().map(|record: &Record| {
                    // 这里是一行数据
                    row![
                        text!("{}", record.record_id).width(Length::Fixed(200.0)),
                        text!("{}", record.rr).width(Length::Fixed(200.0)),
                        text!("{}", record.record_type)
                            .width(Length::Fixed(100.0))
                            .align_x(Alignment::Start),
                        text!("{}", record.value)
                            .width(Length::Fill)
                            .line_height(LineHeight::default())
                            .style(|_theme: &Theme| { text::Style::default() })
                            .align_x(Alignment::Start),
                        button(Text::new(get_text("edit")).align_x(Alignment::Center))
                            .style(|theme: &Theme, status| {
                                match status {
                                    button::Status::Hovered => button::Style::default()
                                        .with_background(Color::from_rgb(255.0, 50.0, 50.0)),
                                    _ => primary(theme, status),
                                }
                            })
                            .on_press(Message::AddDnsFormSubmit)
                            .width(Length::Fixed(100.0)),
                        button(Text::new(get_text("delete")).align_x(Alignment::Center))
                            .style(|theme: &Theme, status| {
                                match status {
                                    button::Status::Hovered => button::Style::default()
                                        .with_background(Color::from_rgb(255.0, 50.0, 50.0)),
                                    _ => danger(theme, status),
                                }
                            })
                            .on_press(Message::DnsDelete(record.record_id.clone()))
                            .width(Length::Fixed(100.0))
                    ]
                    .align_y(Alignment::Center)
                    .into()
                }));

            let title: String = match app.in_query {
                true => format!(
                    "{}：{}({})",
                    get_text("dns_record"),
                    domain_name.name,
                    get_text("in_query")
                ),
                false => format!("{}：{}", get_text("dns_record"), domain_name.name),
            };

            // 返回到解析界面
            let dns_log_content: Column<Message> =
                Column::from_iter(app.dns_log_list.iter().map(|record: &RecordLog| {
                    // 这里是一行数据
                    row![
                        text!("{}", record.action).width(Length::Fixed(100.0)),
                        text!("{}", record.action_time)
                            .width(Length::Fixed(200.0))
                            .align_x(Alignment::Start),
                        text!("{}", record.message)
                            .width(Length::Fill)
                            .line_height(LineHeight::default())
                            .style(|_theme: &Theme| { text::Style::default() })
                            .align_x(Alignment::Start),
                    ]
                    .align_y(Alignment::Center)
                    .into()
                }));

            Column::new()
                .push(
                    row![
                        button(Text::new(get_text("return")).center())
                            .on_press(Message::ChangePage(Page::DomainPage)),
                        row!(text!("{}", title).width(Length::Fill).center(),).width(Length::Fill),
                        button(Text::new("Help").center())
                            .on_press(Message::ToHelp)
                            .width(Length::Fixed(100.0)),
                        button(Text::new(get_text("reload")))
                            .on_press(Message::ToHelp)
                            .width(Length::Fixed(100.0)),
                        button(Text::new(get_text("add_dns_record")).center())
                            .on_press(Message::AddDnsRecord)
                            .width(Length::Fixed(200.0))
                    ]
                    .padding(Padding {
                        bottom: 20.0,
                        ..Default::default()
                    })
                    .align_y(Alignment::Center),
                )
                .push(
                    // 选中了域名
                    text!("Dns Record list for domain：{}", domain_name.name).width(Length::Fill),
                )
                .push_maybe(match app.in_query {
                    true => Some(text!("{}", get_text("in_query")).width(Length::Fill)),
                    false => None,
                })
                // dns 列表
                .push(
                    row![
                        text!("记录标识").width(Length::Fixed(200.0)),
                        text!("主机记录")
                            .width(Length::Fixed(200.0))
                            .wrapping(Wrapping::Word),
                        text!("记录类型")
                            .width(Length::Fixed(100.0))
                            .align_x(Alignment::Start),
                        text!("记录值")
                            .width(Length::Fill)
                            .line_height(LineHeight::default())
                            .style(|_theme: &Theme| { text::Style::default() })
                            .align_x(Alignment::Start),
                        text("操作")
                            .align_x(Alignment::Center)
                            .width(Length::Fixed(200.0))
                    ]
                    .align_y(Alignment::Center),
                )
                .push(dns_content)
                .push(
                    Container::new(row![
                        text!("Dns解析操作记录").width(Length::Fill),
                        button(Text::new(get_text("reload")))
                            .width(Length::Fixed(100.0))
                            .on_press(Message::ToHelp)
                    ])
                    .padding(Padding {
                        bottom: 20.0,
                        ..Default::default()
                    })
                    .style(|_theme: &Theme| {
                        // 北京颜色
                        container::Style {
                            text_color: Some(Color::WHITE),
                            border: Border {
                                color: Color::from_rgb(255.0, 100.2, 0.0),
                                radius: Radius::from(5),
                                ..Default::default()
                            },
                            ..container::Style::default()
                        }
                    })
                    .width(Length::Fill)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Start),
                )
                .push(row![
                    text("操作方式")
                        .width(Length::Fixed(100.0))
                        .line_height(LineHeight::default()),
                    text!("操作时间")
                        .width(Length::Fixed(200.0))
                        .align_x(Alignment::Start),
                    text("详细信息")
                        .width(Length::Fill)
                        .line_height(LineHeight::default())
                ])
                .push(dns_log_content)
                .padding(10)
                .spacing(5)
                .into()
        }
        None => {
            // 没有选择域名，返回到域名列表(这里除非是除了BUG，应该不会走到这里来）
            text!("No Domain Name selected!").width(Length::Fill).into()
        }
    };
    page.into()
}

pub fn add_dns_record(app: &App) -> Element<'static, Message> {
    {
        let record_id_column = match &app.add_dns_form.record_id {
            Some(record_id) => text!("修改Dns记录：{}", record_id)
                .width(Length::Fill)
                .into(),
            None => text!("No record id").width(Length::Fill).into(),
        };

        // 添加 dns 记录
        Container::new(
            Column::new()
                .width(Length::Fill)
                .align_x(Alignment::Start)
                .push_maybe(record_id_column)
                .push(
                    Text::new(get_text("add_dns_record"))
                        .width(Length::Fill)
                        .center(),
                )
                .push(
                    Column::new()
                        .push(text!("主机记录").width(Length::Fill))
                        .push(
                            text_input("Type something here...", &app.add_dns_form.record_name)
                                .on_input(Message::DnsFormNameChanged),
                        )
                        .push(text!("记录类型").width(Length::Fill))
                        .push(pick_list(
                            &Type::ALL[..],
                            app.add_dns_form.record_type.clone(),
                            Message::DnsFormRecordTypeChanged,
                        ))
                        .push(text!("记录值").width(Length::Fill))
                        .push(
                            text_input("Type something here...", &app.add_dns_form.value)
                                .on_input(Message::DnsFormValueChanged),
                        )
                        .push(text!("TTL：{}", app.add_dns_form.ttl).width(Length::Fill))
                        .push(slider(
                            600..=1000,
                            app.add_dns_form.ttl,
                            Message::DnsFormTtlChanged,
                        ))
                        .width(Length::Fill),
                )
                .push(
                    Row::new()
                        .push(
                            button(Text::new(get_text("cancel")))
                                .on_press(Message::AddDnsFormCancelled)
                                .width(Length::Fixed(200.0)),
                        )
                        .push(
                            button(Text::new(get_text("confirm")))
                                .on_press(Message::AddDnsFormSubmit)
                                .width(Length::Fixed(200.0)),
                        )
                        .spacing(20)
                        .width(Length::Fill)
                        .align_y(Alignment::Center),
                ),
        )
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
