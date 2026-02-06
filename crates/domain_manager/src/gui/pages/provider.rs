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
use iced::{Alignment, Element, Font, Length};

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

            let mut row = Row::new()
                .align_y(Alignment::Center)
                .spacing(10)
                .push(Text::new(format!("{} ({})", name, type_name)).width(Length::Fill));

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
                        button(Text::new("编辑").align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::Edit(id))),
                    )
                    .push(
                        button(Text::new("删除").align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::Delete(id)))
                            .class(ButtonType::Alert),
                    );
            }

            list_content = list_content.push(
                Container::new(row)
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
