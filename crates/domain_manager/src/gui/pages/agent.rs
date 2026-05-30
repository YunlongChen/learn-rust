//! Agent 管理页面
//!
//! 显示和管理已连接的 Agent

use crate::gui::handlers::message_handler::{AgentMessage, MessageCategory};
use crate::gui::state::pages::agent_state::AgentPageState;
use crate::gui::styles::container::ContainerType;
use crate::gui::styles::text::TextType;
use crate::utils::i18_utils::get_text;
use crate::StyleType;
use iced::widget::{button, container, row, text, text_input, Column, Container, Row, Space};
use iced::{Alignment, Element, Font, Length, Padding};

/// Agent 管理页面
pub fn agent_page(state: &AgentPageState) -> Element<'_, MessageCategory, StyleType> {
    let is_adding = state.is_adding;

    let add_button_text = if is_adding { "取消添加" } else { "+ 添加" };

    let title_row = row!(
        text(get_text("agent_manage"))
            .size(20)
            .align_x(Alignment::Start),
        Space::with_width(Length::Fill),
        button(text(get_text("reload")).center())
            .on_press(MessageCategory::Agent(AgentMessage::LoadAgents))
            .width(Length::Fixed(80.0)),
        Space::with_width(Length::Fixed(4.0)),
        button(text(add_button_text).center())
            .on_press(MessageCategory::Agent(AgentMessage::ToggleAddMode))
            .width(Length::Fixed(100.0)),
    )
    .align_y(Alignment::Center)
    .padding(Padding {
        bottom: 10.0,
        ..Default::default()
    });

    let mut content = Column::new()
        .push(title_row)
        .push(Space::with_height(Length::Fixed(10.0)));

    if is_adding {
        content = content.push(render_add_agent_form(state));
    }

    content = content.push(agent_list_view(state));

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .into()
}

/// 渲染添加 Agent 表单
fn render_add_agent_form<'a>(
    state: &'a AgentPageState,
) -> Element<'a, MessageCategory, StyleType> {
    let name_input = text_input(
        "Agent 名称",
        &state.new_agent_name,
    )
    .font(Font::with_name("Maple Mono NF CN"))
    .on_input(|name| MessageCategory::Agent(AgentMessage::AddFormNameChanged(name)))
    .padding(8)
    .width(Length::Fill);

    let endpoint_input = text_input(
        "Hub地址 (host:port, 例如: localhost:8081)",
        &state.new_agent_endpoint,
    )
    .font(Font::with_name("Maple Mono NF CN"))
    .on_input(|endpoint| MessageCategory::Agent(AgentMessage::AddFormEndpointChanged(endpoint)))
    .padding(8)
    .width(Length::Fill);

    let key_input = text_input(
        "Agent密钥 (自动生成，可复制)",
        &state.new_agent_key,
    )
    .font(Font::with_name("Maple Mono NF CN"))
    .on_input(|key| MessageCategory::Agent(AgentMessage::AddFormKeyChanged(key)))
    .padding(8)
    .width(Length::Fill);

    // 测试结果消息
    let test_result_text = if let Some(ref result) = state.test_result {
        if result.contains("成功") || result.contains("验证通过") {
            text(result).class(TextType::Success)
        } else if result.contains("失败") || result.contains("错误") || result.contains("无效") {
            text(result).class(TextType::Danger)
        } else {
            text(result).size(12)
        }
    } else {
        text("")
    };

    let form = Column::new()
        .padding(15)
        .spacing(12)
        .push(text("添加新 Agent").size(16))
        // 名称行：标签 + 输入框
        .push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    Container::new(text("名称:").size(14))
                        .width(Length::Fixed(80.0))
                        .align_x(Alignment::End),
                )
                .push(name_input)
        )
        // Hub地址行：标签 + 输入框
        .push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    Container::new(text("Hub地址:").size(14))
                        .width(Length::Fixed(80.0))
                        .align_x(Alignment::End),
                )
                .push(endpoint_input)
        )
        // 密钥行：标签 + 输入框
        .push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    Container::new(text("密钥:").size(14))
                        .width(Length::Fixed(80.0))
                        .align_x(Alignment::End),
                )
                .push(key_input)
        )
        // 能力说明
        .push(text("能力: DDNS客户端, Shell执行, SSL验证").size(12))
        // 一键部署命令提示
        .push(
            text("提示: 将此密钥配置到Agent程序，Agent启动后将自动连接Hub").size(11)
        )
        // 测试结果
        .push(test_result_text)
        // 按钮行
        .push(
            Row::new()
                .spacing(10)
                .push(
                    button("测试连接")
                        .on_press(MessageCategory::Agent(AgentMessage::TestConnection)),
                )
                .push(Space::with_width(Length::Fixed(10.0)))
                .push(
                    button("保存")
                        .on_press(MessageCategory::Agent(AgentMessage::SaveAgent)),
                )
                .push(Space::with_width(Length::Fixed(10.0)))
                .push(
                    button("取消")
                        .on_press(MessageCategory::Agent(AgentMessage::CancelAdd)),
                ),
        );

    Container::new(form)
        .width(Length::Fill)
        .class(ContainerType::Standard)
        .into()
}

/// 渲染 Agent 列表
fn agent_list_view<'a>(state: &'a AgentPageState) -> Element<'a, MessageCategory, StyleType> {
    let agents = &state.agents;

    let header_row = row!(
        text("状态").size(12).width(Length::Fixed(80.0)),
        text("名称").size(12).width(Length::Fixed(150.0)),
        text("Hub地址").size(12).width(Length::Fixed(150.0)),
        text("能力").size(12).width(Length::Fill),
        text("最后心跳").size(12).width(Length::Fixed(100.0)),
        text("操作").size(12).width(Length::Fixed(80.0)),
    )
    .spacing(10)
    .padding(5);

    let empty_state: Element<'a, MessageCategory, StyleType> = container(
        Column::new()
            .spacing(10)
            .align_x(Alignment::Center)
            .push(text("暂无已连接的 Agent").size(14))
            .push(text("Agent 连接后将在此处显示").size(12)),
    )
    .width(Length::Fill)
    .height(Length::Fixed(200.0))
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into();

    let list_content = if agents.is_empty() {
        empty_state
    } else {
        Column::new()
            .spacing(5)
            .push(
                agents.iter().map(|agent| {
                    let status_color = match agent.status {
                        crate::agent::model::AgentStatus::Online => "🟢",
                        crate::agent::model::AgentStatus::Offline => "⚫",
                        crate::agent::model::AgentStatus::Busy => "🟡",
                        crate::agent::model::AgentStatus::Maintenance => "🔵",
                    };
                    let capabilities = agent
                        .capabilities
                        .iter()
                        .map(|c| format!("{}", c))
                        .collect::<Vec<_>>()
                        .join(", ");

                    row!(
                        text(format!("{} {}", status_color, agent.status))
                            .size(12)
                            .width(Length::Fixed(80.0)),
                        text(&agent.name).size(12).width(Length::Fixed(150.0)),
                        text(&agent.endpoint).size(12).width(Length::Fixed(150.0)),
                        text(capabilities).size(12).width(Length::Fill),
                        text(agent.last_heartbeat
                            .map(|t| t.format("%H:%M:%S").to_string())
                            .unwrap_or_else(|| "-".to_string()))
                        .size(12)
                        .width(Length::Fixed(100.0)),
                        button("删除")
                            .on_press(MessageCategory::Agent(AgentMessage::DeleteAgent(
                                agent.id.to_string()
                            )))
                            .width(Length::Fixed(80.0)),
                    )
                    .spacing(10)
                    .padding(5)
                    .into()
                })
                .collect::<Column<'a, MessageCategory, StyleType>>()
            )
            .into()
    };

    Column::new()
        .spacing(5)
        .push(header_row)
        .push(list_content)
        .into()
}
