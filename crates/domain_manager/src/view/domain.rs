use crate::view::theme::danger_btn;
use crate::{get_text, AddDomainField, App, DnsProvider, DomainName, Message, Page};
use iced::widget::button::Status;
use iced::widget::text::LineHeight;
use iced::widget::{
    button, container, horizontal_space, pick_list, row, text, Button, Column, Container, Row,
    Text, TextInput,
};
use iced::{color, Alignment, Element, Length, Padding, Theme};
use rust_i18n::t;

/// 域名管理界面
pub fn domain_page(app: &App) -> Element<'static, Message> {
    // 返回到解析界面
    let domain_name_list: Column<Message> =
        Column::from_iter(app.domain_names.iter().map(|domain_name: &DomainName| {
            // 这里是一行数据
            row![
                text!("{}", domain_name.name).width(Length::Fill),
                text!("{}", domain_name.provider)
                    .width(Length::Fill)
                    .line_height(LineHeight::default())
                    .style(|_theme: &Theme| { text::Style::default() })
                    .align_x(Alignment::Start),
                button(Text::new(t!("dns_record")).center())
                    .on_press(Message::QueryDomainDnsRecord(domain_name.clone()))
                    .width(Length::Fixed(100.0)),
                button(Text::new(t!("delete")).center())
                    .on_press(Message::DomainDeleted(domain_name.clone()))
                    .width(Length::Fixed(100.0))
                    .style(|theme: &Theme, status: Status| { danger_btn(theme, status) })
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .into()
        }))
        .spacing(5);

    let action_row: Row<Message> = Row::new()
        .push(Text::new(get_text("domain_name")).width(Length::Fill))
        .push(
            Text::new(get_text("domain_name"))
                .width(Length::Fill)
                .style(|_theme: &Theme| text::Style::default())
                .align_x(Alignment::Start),
        )
        .push(
            Text::new(t!("operation"))
                .center()
                .width(Length::Fixed(200.0))
                .align_y(Alignment::Center),
        )
        .height(Length::Shrink)
        .spacing(5);

    let in_query_tag = if app.in_query {
        Some(Text::new(get_text("in_query")).width(Length::Fill))
    } else {
        None
    };

    let actions = Row::new()
        .push(
            text(get_text("domain_manage"))
                .align_x(Alignment::Start)
                .width(Length::Fill),
        )
        .push_maybe(in_query_tag)
        .push(
            button(text(get_text("reload")).center())
                .on_press(Message::QueryDomain)
                .width(Length::Fixed(100.0)),
        )
        .push(
            button(Text::new(get_text("add_domain")).center())
                .on_press(Message::ChangePage(Page::AddDomain))
                .width(Length::Fixed(100.0)),
        )
        .push(
            button(Text::new(get_text("change_theme")).center())
                .on_press(Message::ToggleTheme)
                .width(Length::Fixed(100.0)),
        )
        .push(
            button(Text::new(get_text("change_locale")).center())
                .on_press(Message::ChangeLocale)
                .width(Length::Fixed(100.0)),
        )
        .spacing(5)
        .width(Length::Fill)
        .padding(Padding {
            bottom: 10.0,
            ..Default::default()
        })
        .align_y(Alignment::Center);

    let row1 = Column::new().push(actions).push(action_row).push(
        Container::new(domain_name_list)
            .style(container::rounded_box)
            .width(Length::Fill)
            .height(Length::Fill),
    );

    let content2: Column<Message> = Column::new().push(row1).width(Length::Fill);

    Container::new(
        content2.push(
            Row::new()
                .push(text(get_text("domain_manager")).width(Length::Fixed(200.0)))
                .push(horizontal_space().width(Length::Fill))
                .push(text!("Made with Love by {}", app.config.author).align_x(Alignment::End)),
        ),
    )
    .style(container::rounded_box)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    .center(800)
    .center_x(Length::Fill)
    .center_y(Length::Shrink)
    .into()
}

/// 添加域名页面
pub fn add_domain_page(app: &App) -> Element<'static, Message> {
    let state = AddDomainField {
        domain_name: String::from("www.example.com"),
        provider: app.add_domain_field.provider.clone(),
    };

    Container::new(iced::widget::column![
        text("add domain")
            .color(color!(0x0000ff))
            .size(20)
            .style(|_theme: &Theme| {
                text::Style {
                    color: Some(color!(0xff00ff)),
                }
            })
            .width(Length::Fill),
        TextInput::new("domain name", &app.add_domain_field.domain_name)
            .on_input(Message::AddDomainFormChanged),
        pick_list(
            &DnsProvider::ALL[..],
            state.provider,
            Message::DnsProviderSelected
        )
        .placeholder("Select your favorite fruit..."),
        Button::new(text("confirm")).on_press(Message::SubmitDomainForm),
        button(Text::new(get_text("return"))).on_press(Message::ChangePage(Page::DomainPage)),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    .align_top(0)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}
