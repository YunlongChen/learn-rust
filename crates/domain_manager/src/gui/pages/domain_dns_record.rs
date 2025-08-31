use crate::gui::handlers::message_handler::{DnsMessage, MessageCategory, NavigationMessage};
use crate::gui::manager_v2::DomainManagerV2;
use crate::gui::pages::Page;
use crate::model::dns_record_response::Type;
use crate::utils::i18_utils::get_text;
use crate::StyleType;
use iced::widget::text::{LineHeight, Wrapping};
use iced::widget::{
    button, horizontal_space, pick_list, row, scrollable, slider, text, text_input, Column,
    Container, Row, Text,
};
use iced::{Alignment, Length, Padding};

pub fn dns_record<'a>(app: &DomainManagerV2) -> Container<'a, MessageCategory, StyleType> {
    // 展示dns列表
    let page: Column<MessageCategory, StyleType> = match app.current_domain_name() {
        // 选中了域名
        Some(domain) => {
            // 返回到解析界面
            let mut dns_content: Column<MessageCategory, StyleType> = Column::new().spacing(10);
            for record in app.dns_list() {
                let row: Row<MessageCategory, StyleType> = row![
                    text!("{}", record.id).width(Length::Fixed(200.0)),
                    text!("{}", record.name).width(Length::Fixed(200.0)),
                    text!("{}", record.record_type)
                        .width(Length::Fixed(100.0))
                        .align_x(Alignment::Start),
                    // 记录值
                    text!("{}", record.value)
                        .width(Length::Fill)
                        .line_height(LineHeight::default())
                        .align_x(Alignment::Start),
                    // TTL
                    text!("10分钟")
                        .width(Length::Fill)
                        .line_height(LineHeight::default())
                        .align_x(Alignment::Start),
                    // 状态
                    text!("启用")
                        .width(Length::Fill)
                        .line_height(LineHeight::default())
                        .align_x(Alignment::Start),
                    // 创建时间
                    text!("2023-08-20 07:28:59")
                        .width(Length::Fill)
                        .line_height(LineHeight::default())
                        .align_x(Alignment::Start),
                    button(Text::new(get_text("edit")).align_x(Alignment::Center))
                        .on_press(MessageCategory::Dns(DnsMessage::EditRecord(record.clone())))
                        .width(Length::Fixed(80.0)),
                    horizontal_space().width(Length::Fixed(5f32)),
                    button(Text::new(get_text("stop")).align_x(Alignment::Center))
                        .on_press(MessageCategory::Dns(DnsMessage::FormSubmit))
                        .width(Length::Fixed(80.0)),
                    horizontal_space().width(Length::Fixed(5f32)),
                    button(Text::new(get_text("delete")).align_x(Alignment::Center))
                        .on_press(MessageCategory::Dns(DnsMessage::Delete(record.id as usize)))
                        .width(Length::Fixed(80.0)),
                    horizontal_space().width(Length::Fixed(5f32)),
                    button(Text::new(get_text("test")).align_x(Alignment::Center))
                        .on_press(MessageCategory::Dns(DnsMessage::TestRecord(
                            record.id as usize
                        )))
                        .width(Length::Fixed(80.0))
                ];
                dns_content = dns_content.push(row)
            }

            let title: String = match app.in_query() {
                true => format!(
                    "{}：{}({})",
                    get_text("dns_record"),
                    domain.name,
                    get_text("in_query")
                ),
                false => format!("{}：{}", get_text("dns_record"), domain.name),
            };

            // 返回到解析界面
            let mut dns_log_content: Column<MessageCategory, StyleType> = Column::new();

            for record_log in &app.dns_log_list() {
                let record_log_row: Row<MessageCategory, StyleType> = row![
                    text!("{}", record_log.action).width(Length::Fixed(100.0)),
                    text!("{}", record_log.action_time)
                        .width(Length::Fixed(200.0))
                        .align_x(Alignment::Start),
                    text!("{}", record_log.message)
                        .width(Length::Fill)
                        .line_height(LineHeight::default())
                        .align_x(Alignment::Start),
                ]
                .align_y(Alignment::Center);
                dns_log_content = dns_log_content.push(record_log_row);
            }

            Column::new()
                .push(
                    row![
                        row!(text!("{}", title).width(Length::Fill).center(),).width(Length::Fill),
                        button(Text::new(get_text("reload")))
                            .on_press(MessageCategory::Dns(DnsMessage::ReloadDnsRecord(
                                domain.id as usize
                            )))
                            .width(Length::Fixed(100.0)),
                        button(Text::new(get_text("add_dns_record")).center())
                            .on_press(MessageCategory::Navigation(NavigationMessage::PageChanged(
                                Page::AddRecord
                            )))
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
                    text!("Dns Record list for domain：{}", domain.name).width(Length::Fill),
                )
                .push_maybe(match app.in_query() {
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
                            .align_x(Alignment::Start),
                        text!("TTL")
                            .width(Length::Fill)
                            .line_height(LineHeight::default())
                            .align_x(Alignment::Start),
                        text!("状态")
                            .width(Length::Fill)
                            .line_height(LineHeight::default())
                            .align_x(Alignment::Start),
                        text!("创建时间")
                            .width(Length::Fill)
                            .line_height(LineHeight::default())
                            .align_x(Alignment::Start),
                        text("操作")
                            .align_x(Alignment::Center)
                            .width(Length::Fixed(335.0))
                    ]
                    .align_y(Alignment::Center),
                )
                .push(
                    horizontal_space()
                        .width(Length::Fill)
                        .height(Length::Fixed(3.0)),
                )
                .push(scrollable(dns_content).height(Length::FillPortion(2)))
                .push(
                    Container::new(row![
                        text!("Dns解析操作记录").width(Length::Fill),
                        button(Text::new(get_text("reload")))
                            .width(Length::Fixed(100.0))
                            // todo 更新解析记录
                            .on_press(MessageCategory::Dns(DnsMessage::ReloadDnsRecord(1)))
                    ])
                    .padding(Padding {
                        bottom: 20.0,
                        ..Default::default()
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
                .push(scrollable(dns_log_content).height(Length::FillPortion(1)))
                .padding(10)
                .spacing(5)
        }
        None => {
            // 没有选择域名，返回到域名列表(这里除非是除了BUG，应该不会走到这里来）
            Column::new()
                .width(Length::Fill)
                .push(text!("No Domain Name selected!"))
        }
    };
    Container::new(page)
        .width(Length::Shrink)
        .center_x(Length::Fill)
}

pub fn add_dns_record<'a>(app: &DomainManagerV2) -> Container<'a, MessageCategory, StyleType> {
    {
        let record_id_column = match &app.add_dns_form().record_id {
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
                            text_input("Type something here...", &app.add_dns_form().record_name)
                                .on_input(|name| {
                                    MessageCategory::Dns(DnsMessage::FormNameChanged(name))
                                }),
                        )
                        .push(text!("记录类型").width(Length::Fill))
                        .push(pick_list(
                            &Type::ALL[..],
                            app.add_dns_form().record_type.clone(),
                            |record_type| {
                                MessageCategory::Dns(DnsMessage::FormRecordTypeChanged(record_type))
                            },
                        ))
                        .push(text!("记录值").width(Length::Fill))
                        .push(
                            text_input("Type something here...", &app.add_dns_form().value)
                                .on_input(|record_value| {
                                    MessageCategory::Dns(DnsMessage::FormValueChanged(record_value))
                                }),
                        )
                        .push(text!("TTL：{}", app.add_dns_form().ttl).width(Length::Fill))
                        .push(slider(600..=1000, app.add_dns_form().ttl, |ttl| {
                            MessageCategory::Dns(DnsMessage::FormTtlChanged(ttl))
                        }))
                        .width(Length::Fill),
                )
                .push(
                    Row::new()
                        .push(
                            button(Text::new(get_text("cancel")))
                                .on_press(MessageCategory::Dns(DnsMessage::FormCancelled))
                                .width(Length::Fixed(200.0)),
                        )
                        .push(
                            button(Text::new(get_text("confirm")))
                                .on_press(MessageCategory::Dns(DnsMessage::FormSubmit))
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
