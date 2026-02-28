use crate::gui::handlers::message_handler::{MessageCategory, ProviderMessage};
use crate::gui::model::domain::DnsProvider;
use crate::gui::pages::domain::{DomainProvider, ProviderStatus, VerificationStatus};
use crate::gui::state::pages::provider_state::ProviderPageState;
use crate::gui::styles::button::ButtonType;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::text::TextType;
use crate::utils::i18_utils::get_text;
use crate::StyleType;
use iced::widget::{
    button, horizontal_space, mouse_area, pick_list, scrollable, text_input, Column, Container,
    Row, Space, Text,
};
use iced::{Alignment, Color, Element, Font, Length, Padding};

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
                VerificationStatus::Pending => Text::new("正在验证...").class(TextType::Standard),
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
    let mut list_content = Column::new().spacing(15); // 增加间距

    if state.is_loading {
        list_content = list_content.push(Text::new("正在加载数据...").size(16));
    } else {
        for provider in &state.providers {
            list_content = list_content.push(provider_row_view(
                provider,
                state.deleting_provider_id == Some(provider.account_id),
                state.hovered_provider_id == Some(provider.account_id),
            ));
        }
    }

    content = content.push(scrollable(list_content));

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}

/// 渲染单个服务商行
fn provider_row_view(
    provider: &DomainProvider,
    is_deleting: bool,
    is_hovered: bool,
) -> Element<'_, MessageCategory, StyleType> {
    let id = provider.account_id;
    let name = &provider.provider_name;
    let type_name = provider.provider.name();
    let is_expanded = provider.is_expanded;

    // 状态指示灯颜色
    let status_color = match provider.status {
        ProviderStatus::Active => Color::from_rgb(0.0, 0.8, 0.0), // 绿色
        ProviderStatus::Inactive => Color::from_rgb(0.6, 0.6, 0.6), // 灰色
        ProviderStatus::Error => Color::from_rgb(0.9, 0.1, 0.1),  // 红色
    };

    // 侧边栏颜色 (根据服务商类型)
    let side_color = get_provider_color(&provider.provider);

    // 服务商信息部分
    let info_column = Column::new()
        .push(
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    Container::new(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
                        .class(ContainerType::CustomRound(status_color))
                        .width(Length::Fixed(8.0))
                        .height(Length::Fixed(8.0)),
                )
                .push(Text::new(format!("{} ({})", name, type_name)).size(16)),
        )
        .push(
            Text::new(format!(
                "已管理 {} 个域名 • 最后同步于 {}",
                provider.domain_count,
                provider.last_synced_at.as_deref().unwrap_or("从未同步")
            ))
            .size(12)
            .class(TextType::Dimmed),
        )
        .spacing(4);

    // 主行内容
    let mut row = Row::new()
        .align_y(Alignment::Center)
        .spacing(10)
        // 侧边颜色条
        .push(
            Container::new(Space::new(Length::Fixed(4.0), Length::Fill))
                .class(ContainerType::Custom(side_color))
                .height(Length::Fixed(40.0)),
        )
        // 点击区域包裹信息
        .push(
            button(info_column)
                .on_press(MessageCategory::Provider(ProviderMessage::ToggleExpand(id)))
                .class(ButtonType::Transparent)
                .width(Length::Fill),
        );

    // 操作按钮区域
    if is_deleting {
        row = row
            .push(Text::new("确认删除?").class(TextType::Danger))
            .push(
                button(Text::new("确认").size(14).align_x(Alignment::Center))
                    .on_press(MessageCategory::Provider(ProviderMessage::ConfirmDelete(
                        id,
                    )))
                    .class(ButtonType::Alert)
                    .padding([5, 10]),
            )
            .push(
                button(Text::new("取消").size(14).align_x(Alignment::Center))
                    .on_press(MessageCategory::Provider(ProviderMessage::CancelDelete))
                    .class(ButtonType::Neutral)
                    .padding([5, 10]),
            );
    } else if is_hovered || is_expanded {
        // 悬停或展开时显示按钮
        row = row
            .push(
                button(Text::new("添加域名").size(14).align_x(Alignment::Center))
                    .on_press(MessageCategory::Provider(ProviderMessage::ToggleAddDomain(
                        id, true,
                    )))
                    .class(ButtonType::Primary)
                    .padding([5, 10]),
            )
            .push(
                button(Text::new("编辑").size(14).align_x(Alignment::Center))
                    .on_press(MessageCategory::Provider(ProviderMessage::Edit(id)))
                    .class(ButtonType::Primary)
                    .padding([5, 10]),
            )
            .push(
                button(Text::new("删除").size(14).align_x(Alignment::Center))
                    .on_press(MessageCategory::Provider(ProviderMessage::Delete(id)))
                    .class(ButtonType::Alert)
                    .padding([5, 10]),
            );
    } else {
        // 占位符以保持布局稳定，或者直接留空
        row = row.push(horizontal_space().width(Length::Fixed(200.0))); // 预留大概宽度
    }

    // 整个服务商容器
    let mut provider_container = Column::new().push(
        Container::new(
            mouse_area(row)
                .on_enter(MessageCategory::Provider(ProviderMessage::ProviderHovered(
                    Some(id),
                )))
                .on_exit(MessageCategory::Provider(ProviderMessage::ProviderHovered(
                    None,
                ))),
        )
        .padding(5)
        .class(if is_hovered {
            ContainerType::HoveredRow
        } else {
            ContainerType::Standard
        }),
    );

    // 域名列表区域（展开时显示）
    if is_expanded {
        let mut domains_column = Column::new().spacing(5).padding(Padding {
            top: 10.0,
            right: 0.0,
            bottom: 0.0,
            left: 24.0, // 与上方对齐调整
        });

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
                    button(Text::new("确认").size(14).align_x(Alignment::Center))
                        .on_press(MessageCategory::Provider(
                            ProviderMessage::ConfirmAddDomain(id),
                        ))
                        .class(ButtonType::Success)
                        .padding([5, 10]),
                )
                .push(
                    button(Text::new("取消").size(14).align_x(Alignment::Center))
                        .on_press(MessageCategory::Provider(ProviderMessage::ToggleAddDomain(
                            id, false,
                        )))
                        .class(ButtonType::Neutral)
                        .padding([5, 10]),
                );

            domains_column = domains_column.push(
                Container::new(add_domain_form)
                    .padding(10)
                    .class(ContainerType::Bordered),
            );
        }

        if provider.domains.is_empty() {
            if !provider.is_adding_domain {
                domains_column = domains_column.push(
                    Text::new("暂无域名或未同步")
                        .size(14)
                        .class(TextType::Dimmed),
                );
            }
        } else {
            for domain in &provider.domains {
                let domain_row = Row::new()
                    .align_y(Alignment::Center)
                    .spacing(10)
                    .push(Text::new(&domain.name).width(Length::Fill).size(14))
                    .push(
                        button(Text::new("同步").size(12).align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::SyncDomainInfo(
                                id,
                            )))
                            .class(ButtonType::Neutral)
                            .padding([2, 8]),
                    )
                    .push(
                        button(Text::new("删除").size(12).align_x(Alignment::Center))
                            .on_press(MessageCategory::Provider(ProviderMessage::DeleteDomain(
                                id, domain.id,
                            )))
                            .class(ButtonType::Alert)
                            .padding([2, 8]),
                    );
                domains_column = domains_column.push(
                    Container::new(domain_row)
                        .padding(5)
                        .class(ContainerType::Standard), // 使用更轻量的样式
                );
            }
        }

        // 边框包裹
        let domains_container = Container::new(domains_column)
            .padding(10)
            .class(ContainerType::Standard)
            .width(Length::Fill);

        provider_container = provider_container.push(domains_container);
    }

    Container::new(provider_container)
        .padding(0) // 移除内边距，让背景色充满
        .class(ContainerType::Bordered)
        .into()
}

/// 获取服务商对应的颜色
fn get_provider_color(provider: &DnsProvider) -> Color {
    match provider {
        DnsProvider::Aliyun => Color::from_rgb8(255, 106, 0), // 阿里云橙
        DnsProvider::TencentCloud => Color::from_rgb8(0, 164, 255), // 腾讯蓝
        DnsProvider::CloudFlare => Color::from_rgb8(243, 128, 32), // Cloudflare 橙
        DnsProvider::Aws => Color::from_rgb8(35, 47, 62),     // AWS 深蓝
        DnsProvider::Google => Color::from_rgb8(66, 133, 244), // Google 蓝
        _ => Color::from_rgb8(100, 100, 100),                 // 默认灰
    }
}
