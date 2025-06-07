use crate::gui::manager::DomainManager;
use crate::gui::model::domain::Domain;
use crate::gui::types::message::Message;
use crate::StyleType;
use iced::widget::text::LineHeight;
use iced::widget::{button, column, row, scrollable, text, Column, Container, Row, Text};
use iced::{alignment, Alignment, Length, Pixels};
use rust_i18n::t;

/// 域名管理界面
pub fn scrollables(app: &DomainManager) -> Container<Message, StyleType> {
    // let element = domain_list_view(&app.domain_names);
    // let scrollable = optimized_list(&app.domain_names);
    println!("yyy{:?}", &app.current_domain_name);

    // 生成一个列表
    let column: Column<Message, StyleType> = column![button("I am to the top!")]
        .align_x(Alignment::Center)
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .align_y(alignment::Vertical::Center)
                .push(column!("Domain List 特殊的一列"))
                .push(column!("Domain Lis222t"))
                .width(Length::Fill)
                .spacing(20),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(button("I am to the bottom!"))
        .into();

    // 生成一个列表
    let column_content: Column<Message, StyleType> = column![button("I am to the top!")]
        .align_x(Alignment::Center)
        .push(
            Row::new()
                .push(column!("Domain List 第一列"))
                .push(column!("Domain Lis222t")),
        )
        .push(button("I am to the bottom!"))
        .into();

    let domain_names: &[Domain] = &app.domain_names;

    let container: Column<Message, StyleType> = domain_list_view(&domain_names);

    let column_content_internal = Column::new()
        .push(
            scrollable(column)
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .push(
            scrollable(column_content)
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .push(
            Column::new()
                .push(
                    Row::new()
                        .push(text!("这里是Title{}", "标题信息").width(Length::Fill))
                        .height(Length::Fixed(12f32)),
                )
                .push(row!(scrollable(container)))
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        );

    // 优化后的列表
    Container::new(column_content_internal)
        .height(Length::Fixed(20f32))
        .width(Length::Fill)
}

// 列表组件实现
fn domain_list_view<'a>(_domain_names: &[Domain]) -> Column<'a, Message, StyleType> {
    let domain_name = Domain::default();
    // row![
    //     text!("{}", &domain_name.name).width(Length::Fill),
    //     text!("{}", &domain_name.provider)
    //         .width(Length::Fill)
    //         .line_height(LineHeight::default())
    //         .style(|_theme: &Theme| { text::Style::default() })
    //         .align_x(Alignment::Start),
    //     button(Text::new(t!("dns_record")).center())
    //         .on_press(Message::QueryDomainDnsRecord(domain_name.clone()))
    //         .width(Length::Fixed(100.0)),
    //     button(Text::new(t!("delete")).center())
    //         .on_press(Message::DomainDeleted(domain_name.clone()))
    //         .width(Length::Fixed(100.0))
    //         .style(|theme: &Theme, status: Status| { danger_btn(theme, status) })
    // ]
    // .spacing(5)
    // .align_y(Alignment::Center);

    // 将每个域名转换为行元素
    let mut container: Column<Message, StyleType> =
        Column::new().push(Row::new().push(text(t!("domain_name")).width(Length::Fill)));

    // 循环二十次
    for index in 0..20 {
        container = container.push(
            Row::new()
                .push(text!("域名名称：{}", &domain_name.name).width(Length::Fill))
                .push(
                    Text::new(format!("域名名称：{}", &domain_name.name))
                        .width(Length::Fill)
                        .line_height(LineHeight::default()),
                )
                .push(
                    button(Text::new(t!("dns_record")).center())
                        .on_press(Message::QueryDomainDnsRecord(domain_name.clone()))
                        .width(Length::Fixed(100.0)),
                )
                .push(
                    button(Text::new("DNS记录").line_height(LineHeight::Absolute(Pixels(32.0))))
                        .width(Length::Fixed(100.0))
                        .on_press(Message::QueryDomain),
                )
                // .push(
                //     text!("服务提供商：{}", &domain_name.provider)
                //         .width(Length::Fill)
                //         .line_height(LineHeight::default())
                //         .style(|_theme: &Theme| text::Style::default())
                //         .align_x(Alignment::Start),
                // )
                // .push(
                //     button(Text::new(t!("delete")).center())
                //         .on_press(Message::DomainDeleted(domain_name.clone()))
                //         .width(Length::Fixed(100.0))
                //         .style(|theme: &Theme, status: Status| danger_btn(theme, status)),
                // )
                .push(text(format!("标题-{}", index)).width(Length::Fill))
                .push("域名信息")
                .width(Length::Fill),
        )
    }

    // 优化后的列表
    container
}
