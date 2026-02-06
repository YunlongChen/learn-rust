use crate::gui::handlers::message_handler::AddDomainFormMessage::ProviderChanged;
use crate::gui::handlers::message_handler::{
    AddDomainFormMessage, DomainMessage, MessageCategory, NavigationMessage,
};
use crate::gui::manager_v2::DomainManagerV2;
use crate::gui::model::domain::{DnsProvider, Domain};
use crate::gui::model::form::AddDomainField;
use crate::gui::pages::names::Page;
use crate::gui::types::credential::Credential;
use crate::models::account::Account;
use crate::utils::i18_utils::get_text;
use crate::StyleType;
use iced::widget::horizontal_space;
use iced::widget::text::LineHeight;
use iced::widget::{
    button, pick_list, scrollable, text, Button, Column, Container, Row, Space, Text,
};
use iced::{Alignment, Length, Padding};
use rust_i18n::t;
use std::default::Default;

// 列表组件实现
fn domain_list_view<'a>(domains: &[Domain]) -> Column<'a, MessageCategory, StyleType> {
    // 将每个域名转换为行元素
    let mut container: Column<MessageCategory, StyleType> = Column::new().spacing(5);
    for domain in domains {
        let domain_line: Row<MessageCategory, StyleType> = Row::new()
            .push(text!("选择").width(Length::Fill))
            .push(text!("{}", &domain.name).width(Length::Fill))
            .push(
                text!("{}", &domain.provider)
                    .width(Length::Fill)
                    .line_height(LineHeight::default()),
            )
            .push(text!("正常").width(Length::Fill))
            .push(text!("到期时间").width(Length::Fill))
            .push(text!("标签").width(Length::Fill))
            .push(
                button(Text::new(t!("dns_record")).center())
                    .on_press(MessageCategory::Domain(DomainMessage::Selected(
                        domain.clone(),
                    )))
                    .width(Length::Fixed(100.0)),
            )
            .push(horizontal_space().width(Length::Fixed(4f32)).height(4))
            .push(
                button(Text::new(t!("delete")).center())
                    .on_press(MessageCategory::Domain(DomainMessage::Delete(
                        domain.id as usize,
                    )))
                    .width(Length::Fixed(100.0)),
            );
        container = container.push(domain_line);
    }
    container.width(Length::Fill).into()
}

/// 域名管理界面
pub fn domain_page(app: &DomainManagerV2) -> Container<'_, MessageCategory, StyleType> {
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
                    .push_maybe(match app.in_query() {
                        true => Some(Text::new(get_text("in_query")).width(Length::Fill)),
                        false => None,
                    })
                    .push(
                        button(text(get_text("reload")).center())
                            .on_press(MessageCategory::Domain(DomainMessage::Reload))
                            .width(Length::Fixed(100.0)),
                    )
                    .push(horizontal_space().width(Length::Fixed(4f32)).height(4))
                    .push(
                        button(Text::new(get_text("add_domain")).center())
                            .on_press(MessageCategory::Navigation(NavigationMessage::PageChanged(
                                Page::AddDomain,
                            )))
                            .width(Length::Fixed(100.0)),
                    )
                    .push(
                        button(Text::new(get_text("add_provider")).center())
                            .on_press(MessageCategory::Navigation(NavigationMessage::PageChanged(
                                Page::Providers,
                            )))
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
                scrollable(domain_list_view(&app.domain_list()))
                    .height(Length::Fill)
                    .width(Length::Fill),
            ),
    )
}

/// 添加域名页面
pub fn add_domain_page<'a>(_app: &DomainManagerV2) -> Container<'a, MessageCategory, StyleType> {
    let state = AddDomainField {
        domain_name: String::from("www.example.com"),
        provider: None, // TODO: 需要从DomainManagerV2中获取
    };

    let content: Container<MessageCategory, StyleType> = Container::new(iced::widget::column![
        text("add domain").size(20).width(Length::Fill),
        pick_list(&DnsProvider::ALL[..], state.provider, |pro| {
            MessageCategory::Domain(DomainMessage::AddFormChanged(ProviderChanged(Some(pro))))
        })
        .placeholder("Select your favorite fruit..."),
        Button::new(text("confirm")).on_press(MessageCategory::Domain(
            DomainMessage::AddFormChanged(AddDomainFormMessage::Submit)
        )),
        button(Text::new(get_text("return"))).on_press(MessageCategory::Navigation(
            NavigationMessage::PageChanged(Page::DomainPage,)
        )),
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DomainProvider {
    pub account_id: i64,
    pub provider_name: String,
    pub provider: DnsProvider,
    pub credential: Credential,
    pub is_expanded: bool,
    pub is_adding_domain: bool,
    pub new_domain_name: String,
    pub domains: Vec<Domain>,
    pub status: ProviderStatus,
    pub last_synced_at: Option<String>,
    pub domain_count: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum ProviderStatus {
    Active,
    Inactive,
    Error,
}

impl Default for ProviderStatus {
    fn default() -> Self {
        ProviderStatus::Inactive
    }
}

impl From<Account> for DomainProvider {
    fn from(input_account: Account) -> Self {
        DomainProvider {
            provider_name: input_account.username.clone(),
            provider: input_account.provider_type.clone().into(),
            // todo 这里有可能报错
            account_id: input_account.id,
            credential: input_account.try_into().unwrap(),
            is_expanded: false,
            is_adding_domain: false,
            new_domain_name: String::new(),
            domains: vec![],
            status: ProviderStatus::Inactive, // 默认为未激活，加载后更新
            last_synced_at: None,
            domain_count: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    None,
    Pending,
    Success,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct AddDomainProviderForm {
    pub provider_name: String,
    pub provider: Option<DnsProvider>,
    pub credential: Option<Credential>,
    pub verification_status: VerificationStatus,
}

impl AddDomainProviderForm {
    pub fn clear(&mut self) {
        self.provider_name = String::new();
        self.credential = None;
        self.provider = None;
        self.verification_status = VerificationStatus::None;
    }
}

impl Default for AddDomainProviderForm {
    fn default() -> Self {
        Self {
            provider_name: String::new(),
            provider: None,
            credential: None,
            verification_status: VerificationStatus::None,
        }
    }
}

// /// 添加域名托管商页面
// pub fn add_domain_provider_page(_app: &DomainManagerV2) -> Container<Message, StyleType> {
//     // TODO: 需要从DomainManagerV2中获取add_domain_provider_form
//     let state = AddDomainProviderForm::default();

//     // 动态生成凭证表单
//     let dyn_form: Option<Element<Message, StyleType>> =
//         state.credential.as_ref().and_then(|credential| {
//             // 将凭证消息转换为顶层消息
//             Some(
//                 credential
//                     .view()
//                     .map(|credential_message| credential_message.into()),
//             )
//         });

//     let content = Column::new()
//         .push(
//             TextInput::new("域名名称", &state.provider_name)
//                 .on_input(Message::AddProviderFormNameChanged)
//                 .padding(10),
//         )
//         .push(
//             pick_list(
//                 &DnsProvider::ALL[..],
//                 state.provider.clone(),
//                 Message::AddProviderFormProviderChanged,
//             )
//             .width(Length::Fill)
//             .placeholder("选择域名托管商..."),
//         )
//         .push_maybe(dyn_form) // 动态添加凭证表单
//         .push(
//             Row::new()
//                 .push(
//                     Button::new(
//                         Text::new(get_text("validate_credential")).align_x(Alignment::Center),
//                     )
//                     .on_press(Message::SubmitDomainForm)
//                     .width(Length::FillPortion(1)),
//                 )
//                 .push(
//                     button(Text::new(get_text("validate")).align_x(Alignment::Center))
//                         .on_press(Message::ValidateCredential)
//                         .width(Length::FillPortion(1)),
//                 )
//                 .push(
//                     Button::new(Text::new(get_text("add")).align_x(Alignment::Center))
//                         .on_press(Message::AddCredential)
//                         .width(Length::FillPortion(1)),
//                 )
//                 .push(
//                     Button::new(text("返回").align_x(Alignment::Center))
//                         .on_press(Message::ChangePage(Page::DomainPage))
//                         .width(Length::FillPortion(1)),
//                 )
//                 .spacing(20),
//         )
//         .spacing(20)
//         .padding(20);

//     Container::new(content).height(Length::Fill)
// }
