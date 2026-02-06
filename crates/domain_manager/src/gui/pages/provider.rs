use crate::gui::handlers::message_handler::{MessageCategory, ProviderMessage};
use crate::gui::model::domain::DnsProvider;
use crate::gui::pages::domain::VerificationStatus;
use crate::gui::state::pages::provider_state::ProviderPageState;
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::text::TextType;
use crate::utils::i18_utils::get_text;
use crate::StyleType;
use iced::widget::{
    button, horizontal_space, pick_list, scrollable, text_input, Column, Container, Row, Text,
};
use iced::{Alignment, Element, Font, Length, Padding};

pub fn provider_page(state: &ProviderPageState) -> Element<'_, MessageCategory, StyleType> {
    let form = &state.form;

    // 1. 顶部操作栏
    let toggle_text = if state.form_visible {
        "-收起"
    } else {
        "+添加"
    };

    let header_row = Row::new()
        .push(Text::new("域名服务商管理").size(24))
        .push(horizontal_space())
        .push(
            button(Text::new(toggle_text).align_x(Alignment::Center))
                .on_press(MessageCategory::Provider(ProviderMessage::ToggleForm(
                    !state.form_visible,
                )))
                .width(Length::Shrink),
        )
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let mut content = Column::new().push(header_row).spacing(20);

    // 2. 表单区域 (可折叠)
    if state.form_visible {
        // 动态生成凭证表单
        let dyn_form: Option<Element<MessageCategory, StyleType>> =
            form.credential.as_ref().and_then(|credential| {
                Some(credential.view().map(|credential_message| {
                    MessageCategory::Provider(ProviderMessage::AddFormCredentialChanged(
                        credential_message,
                    ))
                }))
            });

        let form_content = Column::new()
            .push(
                pick_list(&DnsProvider::ALL[..], form.provider.clone(), |provider| {
                    MessageCategory::Provider(ProviderMessage::Selected(provider))
                })
                .width(Length::Fill)
                .placeholder("选择域名托管商..."),
            )
            .push(
                text_input("服务商名称 (自动生成，可修改)", &form.provider_name)
                    .font(Font::with_name("Maple Mono NF CN"))
                    .on_input(|name| {
                        MessageCategory::Provider(ProviderMessage::AddFormNameChanged(name))
                    })
                    .padding(10),
            )
            .push_maybe(dyn_form) // 动态添加凭证表单
            .push(match &form.verification_status {
                VerificationStatus::None => Text::new(""),
                VerificationStatus::Pending => {
                    Text::new("正在验证...").class(TextType::Standard)
                }
                VerificationStatus::Success => Text::new("验证通过").class(TextType::Success),
                VerificationStatus::Failed(err) => {
                    Text::new(format!("验证失败: {}", err)).class(TextType::Danger)
                }
            })
            .push(
                Row::new()
                    .push(
                        button(
                            Text::new(get_text("provider.validate_credential"))
                                .align_x(Alignment::Center),
                        )
                        .on_press(MessageCategory::Provider(
                            ProviderMessage::ValidateCredential,
                        ))
                        .width(Length::FillPortion(1)),
                    )
                    .push(
                        button(
                            Text::new(if state.editing_provider_id.is_some() {
                                "保存修改"
                            } else {
                                "添加"
                            })
                            .align_x(Alignment::Center),
                        )
                        .on_press(MessageCategory::Provider(ProviderMessage::AddCredential))
                        .width(Length::FillPortion(1)),
                    )
                    .spacing(20),
            )
            .spacing(10);

        content = content.push(
            Container::new(form_content)
                .padding(15)
                .class(ContainerType::Bordered),
        );
    }

    // 3. 列表区域
    let mut list_content = Column::new().spacing(10);

    if state.is_loading {
        list_content = list_content.push(Text::new("正在加载数据...").size(16));
    } else {
        for provider in &state.providers {
            let id = provider.account_id;
            let name = &provider.provider_name;
            let type_name = provider.provider.name();
            let is_expanded = provider.is_expanded;

            // 服务商主行
            let mut row = Row::new()
                .align_y(Alignment::Center)
                .spacing(10)
                // 展开/收起按钮（或点击整行）
                .push(
                    button(Text::new(format!("{} ({})", name, type_name)))
                        .on_press(MessageCategory::Provider(ProviderMessage::ToggleExpand(id)))
                        .class(ButtonType::Link)
                        .width(Length::Fill),
                );

            // 如果处于删除确认状态
            if state.deleting_provider_id == Some(id) {
                row = row
                    .push(Text::new("确认删除此服务商及其所有相关数据?").class(TextType::Danger))
                    .push(
                        button(Text::new("确认").align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::ConfirmDelete(
                                id,
                            )))
                            .class(ButtonType::Alert),
                    )
                    .push(
                        button(Text::new("取消").align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::CancelDelete))
                            .class(ButtonType::Neutral),
                    );
            } else {
                row = row
                    .push(
                        button(Text::new("添加域名").align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::ToggleAddDomain(
                                id, true,
                            )))
                            .class(ButtonType::Success),
                    )
                    .push(
                        button(Text::new("编辑").align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::Edit(id))),
                    )
                    .push(
                        button(Text::new("删除").align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::Delete(id)))
                            .class(ButtonType::Alert),
                    );
            }

            let mut provider_container = Column::new().push(row);

            // 域名列表区域（展开时显示）
            if is_expanded {
                let mut domains_column = Column::new()
                    .spacing(5)
                    .padding(Padding {
                        top: 10.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 20.0,
                    }); // 缩进

                // 添加域名表单
                if provider.is_adding_domain {
                    let add_domain_form = Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(
                            text_input("输入域名 (例如: example.com)", &provider.new_domain_name)
                                .on_input(move |name| {
                                    MessageCategory::Provider(ProviderMessage::NewDomainNameChanged(
                                        id, name,
                                    ))
                                })
                                .padding(5)
                                .width(Length::Fill),
                        )
                        .push(
                            button(Text::new("确认添加").align_x(Alignment::Center))
                                .on_press(MessageCategory::Provider(
                                    ProviderMessage::ConfirmAddDomain(id),
                                ))
                                .class(ButtonType::Success),
                        )
                        .push(
                            button(Text::new("取消").align_x(Alignment::Center))
                                .on_press(MessageCategory::Provider(
                                    ProviderMessage::ToggleAddDomain(id, false),
                                ))
                                .class(ButtonType::Neutral),
                        );

                    domains_column = domains_column.push(
                        Container::new(add_domain_form)
                            .padding(10)
                            .class(ContainerType::Bordered)
                    );
                }

                if provider.domains.is_empty() {
                    if !provider.is_adding_domain {
                        domains_column = domains_column.push(Text::new("暂无域名或未同步"));
                    }
                } else {
                    for domain in &provider.domains {
                        let domain_row = Row::new()
                            .align_y(Alignment::Center)
                            .spacing(10)
                            .push(
                                Container::new(Text::new(&domain.name))
                                    .padding(5)
                                    .class(ContainerType::Bordered)
                                    .width(Length::Fill),
                            )
                            .push(
                                button(Text::new("同步").align_x(Alignment::Center))
                                    .on_press(MessageCategory::Provider(
                                        ProviderMessage::SyncDomainInfo(id),
                                    ))
                                    .width(Length::Fixed(80.0)),
                            )
                            .push(
                                button(Text::new("删除").align_x(Alignment::Center))
                                    .on_press(MessageCategory::Provider(
                                        ProviderMessage::DeleteDomain(id, domain.id),
                                    ))
                                    .class(ButtonType::Alert)
                                    .width(Length::Fixed(80.0)),
                            );
                        domains_column = domains_column.push(domain_row);
                    }
                }

                // 边框包裹
                let domains_container = Container::new(domains_column)
                    .padding(10)
                    .class(ContainerType::Standard) // 或者是其他样式，表示嵌套区域
                    .width(Length::Fill);

                provider_container = provider_container.push(domains_container);
            }

            list_content = list_content.push(
                Container::new(provider_container)
                    .padding(10)
                    .class(ContainerType::Bordered),
            );
        }
    }

    content = content.push(scrollable(list_content));

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}
