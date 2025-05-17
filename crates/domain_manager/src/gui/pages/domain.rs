use crate::gui::manager::DomainManager;
use crate::gui::model::domain::DomainName;
use crate::gui::model::form::AddDomainField;
use crate::gui::pages::names::Page;
use crate::gui::styles::container::ContainerType;
use crate::gui::types::message::Message;
use crate::utils::i18_utils::get_text;
use crate::StyleType;
use iced::widget::{button, row, text, Column, Container, Row, Space, Text};
use iced::{Alignment, Length, Padding};

use iced::widget::{lazy, Scrollable};

// 正确关联生命周期的版本
fn optimized_list<'a>(items: &'a [DomainName]) -> Scrollable<'a, Message> {
    Scrollable::new(
        lazy(items.len(), |index| {
            let index = *index;

            let item = &items[index];
            row![
                text(&item.name).width(Length::Fill),
                button("Query").on_press(Message::ToHelp)
            ]
                .spacing(10)
        }
        )
    )
}

// // 列表组件实现
// fn domain_list_view<'a>(domain_names: &[DomainName]) -> Element<'a, Message, StyleType> {
//     let domain_name = DomainName::default();
//
//     row![
//         text!("{}", &domain_name.name).width(Length::Fill),
//         text!("{}", &domain_name.provider)
//             .width(Length::Fill)
//             .line_height(LineHeight::default())
//             .style(|_theme: &Theme| { text::Style::default() })
//             .align_x(Alignment::Start),
//         button(Text::new(t!("dns_record")).center())
//             .on_press(Message::QueryDomainDnsRecord(domain_name.clone()))
//             .width(Length::Fixed(100.0)),
//         button(Text::new(t!("delete")).center())
//             .on_press(Message::DomainDeleted(domain_name.clone()))
//             .width(Length::Fixed(100.0))
//             .style(|theme: &Theme, status: Status| { danger_btn(theme, status) })
//     ]
//     .spacing(5)
//     .align_y(Alignment::Center);
//
//     // 将每个域名转换为行元素
//     let mut container = Column::new();
//     for domain_name in domain_names {
//         println!("测试参数绑定：{:?}", domain_name.name);
//         container = container.push(Row::new())
//     }
//     container.width(Length::Fill).into()
// }

/// 域名管理界面
pub fn domain_page(app: &DomainManager) -> Container<Message, StyleType> {
    // let element = domain_list_view(&app.domain_names);
    let scrollable = optimized_list(&app.domain_names);
    Container::new(
        Column::new()
            .push(scrollable)
            .push(
                Row::new().push(Space::with_height(5)).push(
                    text(get_text("domain_manage"))
                        .align_x(Alignment::Start)
                        .width(Length::Fill),
                ),
            )
            .push(
                Row::new()
                    .push(Space::with_height(5))
                    .push(
                        text(get_text("domain_manage"))
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
                    .align_y(Alignment::Center),
            )
            .push(
                Row::new()
                    .push(Column::new().push(text(get_text("domain_manage")).width(Length::Fill)))
                    .push(Space::with_width(30)),
            )
            .push(text(get_text("add_domain")))
            .push(Space::with_height(5))
            .push(Row::new().push(Space::with_width(30))),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .class(ContainerType::Gradient(app.config.color_gradient))
}

/// 添加域名页面
pub fn add_domain_page<'a>(app: &DomainManager) -> Container<'a, Message, StyleType> {
    let state = AddDomainField {
        domain_name: String::from("www.example.com"),
        provider: app.add_domain_field.provider.clone(),
    };
    //
    // let container = Container::new(iced::widget::column![
    //     text("add domain")
    //         .color(color!(0x0000ff))
    //         .size(20)
    //         .style(|_theme: &Theme| {
    //             text::Style {
    //                 color: Some(color!(0xff00ff)),
    //             }
    //         })
    //         .width(Length::Fill),
    //     // TextInput::new("domain name", &app.add_domain_field.domain_name)
    //     //     .on_input(Message::AddDomainFormChanged),
    //     pick_list(
    //         &DnsProvider::ALL[..],
    //         state.provider,
    //         Message::DnsProviderSelected
    //     )
    //     .placeholder("Select your favorite fruit..."),
    //     Button::new(text("confirm")).on_press(Message::SubmitDomainForm),
    //     button(Text::new(get_text("return"))).on_press(Message::ChangePage(Page::DomainPage)),
    // ])
    // .width(Length::Fill)
    // .height(Length::Fill)
    // .padding(10)
    // .align_top(0)
    // .center_x(Length::Fill)
    // .center_y(Length::Fill);

    let body = Column::new()
        .push(text(get_text("add_domain")))
        .push(Space::with_height(5))
        .push(Row::new().push(Space::with_width(30)));
    Container::new(body).height(Length::Fill)
}
