use crate::gui::manager::DomainManager;
use crate::gui::model::domain::{DnsProvider, Domain};
use crate::gui::model::form::AddDomainField;
use crate::gui::pages::names::Page;
use crate::gui::types::message::Message;
use crate::utils::i18_utils::get_text;
use crate::StyleType;
use iced::widget::horizontal_space;
use iced::widget::text::LineHeight;
use iced::widget::{
    button, pick_list, scrollable, text, Button, Column, Container, Row, Space, Text,
};
use iced::{Alignment, Length, Padding};
use rust_i18n::t;

// 列表组件实现
fn domain_list_view<'a>(domain_names: &[Domain]) -> Column<'a, Message, StyleType> {
    // 将每个域名转换为行元素
    let mut container: Column<Message, StyleType> = Column::new().spacing(5);
    for domain_name in domain_names {
        let domain_line: Row<Message, StyleType> = Row::new()
            .push(text!("选择").width(Length::Fill))
            .push(text!("{}", &domain_name.name).width(Length::Fill))
            .push(
                text!("{}", &domain_name.provider)
                    .width(Length::Fill)
                    .line_height(LineHeight::default()),
            )
            .push(text!("正常").width(Length::Fill))
            .push(text!("到期时间").width(Length::Fill))
            .push(text!("标签").width(Length::Fill))
            .push(
                button(Text::new(t!("dns_record")).center())
                    .on_press(Message::QueryDomainDnsRecord(domain_name.clone()))
                    .width(Length::Fixed(100.0)),
            )
            .push(horizontal_space().width(Length::Fixed(4f32)).height(4))
            .push(
                button(Text::new(t!("delete")).center())
                    .on_press(Message::DomainDeleted(domain_name.clone()))
                    .width(Length::Fixed(100.0)),
            );
        container = container.push(domain_line);
    }
    container.width(Length::Fill).into()
}

/// 域名管理界面
pub fn domain_page(app: &DomainManager) -> Container<Message, StyleType> {
    // let scrollable = optimized_list(&app.domain_names);
    Container::new(
        Column::new()
            .push(horizontal_space().width(Length::Fill).height(4))
            .spacing(5)
            .push(
                Row::new()
                    .push(
                        text(get_text("domain_manage"))
                            .size(20)
                            .align_x(Alignment::Start)
                            .width(Length::Fill),
                    )
                    .push_maybe(match app.in_query {
                        true => Some(Text::new(get_text("in_query")).width(Length::Fill)),
                        false => None,
                    })
                    .push(
                        button(text(get_text("reload")).center())
                            .on_press(Message::QueryDomain)
                            .width(Length::Fixed(100.0)),
                    )
                    .push(horizontal_space().width(Length::Fixed(4f32)).height(4))
                    .push(
                        button(Text::new(get_text("add_domain")).center())
                            .on_press(Message::ChangePage(Page::AddDomain))
                            .width(Length::Fixed(100.0)),
                    )
                    .push(
                        button(Text::new(get_text("add_provider")).center())
                            .on_press(Message::ChangePage(Page::AddDomain))
                            .width(Length::Fixed(100.0)),
                    )
                    .width(Length::Fill)
                    .padding(Padding {
                        bottom: 10.0,
                        ..Default::default()
                    })
                    .align_y(Alignment::Center),
            )
            .push(
                // 域名列表
                scrollable(domain_list_view(&app.domain_names))
                    .height(Length::Fill)
                    .width(Length::Fill),
            ),
    )
}

/// 添加域名页面
pub fn add_domain_page<'a>(app: &DomainManager) -> Container<'a, Message, StyleType> {
    let state = AddDomainField {
        domain_name: String::from("www.example.com"),
        provider: app.add_domain_field.provider.clone(),
    };

    let content: Container<Message, StyleType> = Container::new(iced::widget::column![
        text("add domain").size(20).width(Length::Fill),
        // TextInput::new("domain name", &app.add_domain_field.domain_name)
        //     .on_input(Message::AddDomainFormChanged),
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
    .center_y(Length::Fill);

    let body = Column::new()
        .push(content)
        .push(Space::with_height(5))
        .push(Row::new().push(Space::with_width(30)));
    Container::new(body).height(Length::Fill)
}
