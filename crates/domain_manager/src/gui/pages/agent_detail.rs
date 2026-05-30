//! Agent 详情页面
//!
//! 显示 Agent 的详细信息

use crate::agent::model::{Agent, AgentApprovalState, AgentStatus};
use crate::gui::handlers::message_handler::{AgentMessage, MessageCategory};
use crate::gui::styles::container::ContainerType;
use crate::StyleType;
use iced::widget::{button, container, row, text, Column, Container, Row, Space};
use iced::{Alignment, Element, Length, Padding};

/// Agent 详情页面
pub fn agent_detail_page(
    agent: &Agent,
) -> Element<'_, MessageCategory, StyleType> {
    let status_color = match agent.status {
        AgentStatus::Online => "🟢",
        AgentStatus::Offline => "⚫",
        AgentStatus::Busy => "🟡",
        AgentStatus::Maintenance => "🔵",
    };

    let approval_text = match agent.approval_state {
        AgentApprovalState::Pending => "⏳ 待审批",
        AgentApprovalState::Approved => "✅ 已批准",
        AgentApprovalState::Denied => "❌ 已拒绝",
    };

    let hostname = agent.hostname.clone().unwrap_or_else(|| "-".to_string());
    let version = agent.version.clone().unwrap_or_else(|| "-".to_string());
    let last_heartbeat = agent.last_heartbeat
        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "-".to_string());
    let created_at = agent.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at = agent.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let key_hash = agent.agent_key_hash.clone().unwrap_or_else(|| "-".to_string());
    let approved_at = agent.approved_at
        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "-".to_string());
    let approved_by = agent.approved_by.clone().unwrap_or_else(|| "-".to_string());
    let capabilities = agent.capabilities.iter().map(|c| format!("{}", c)).collect::<Vec<_>>().join(", ");

    // 基本信息卡片内容
    let basic_card_content: Element<'_, MessageCategory, StyleType> = Column::new()
        .spacing(8)
        .push(text("基本信息").size(16))
        .push(Space::with_height(Length::Fixed(4.0)))
        // 名称行
        .push({
            let label = "名称";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(&agent.name).size(14))
        })
        // ID行
        .push({
            let label = "ID";
            let value = agent.id.to_string();
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(value.clone()).size(14))
        })
        // 状态行
        .push({
            let label = "状态";
            let value = format!("{} {}", status_color, agent.status);
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(value.clone()).size(14))
        })
        // Hub地址行
        .push({
            let label = "Hub地址";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(agent.endpoint.clone()).size(14))
        })
        // 审批状态行
        .push({
            let label = "审批状态";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(approval_text).size(14))
        })
        // 能力行
        .push({
            let label = "能力";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(capabilities.clone()).size(14))
        })
        .into();

    // 系统信息卡片内容
    let system_card_content: Element<'_, MessageCategory, StyleType> = Column::new()
        .spacing(8)
        .push(text("系统信息").size(16))
        .push(Space::with_height(Length::Fixed(4.0)))
        .push({
            let label = "主机名";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(hostname.clone()).size(14))
        })
        .push({
            let label = "版本";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(version.clone()).size(14))
        })
        .into();

    // 连接信息卡片内容
    let connection_card_content: Element<'_, MessageCategory, StyleType> = Column::new()
        .spacing(8)
        .push(text("连接信息").size(16))
        .push(Space::with_height(Length::Fixed(4.0)))
        .push({
            let label = "最后心跳";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(last_heartbeat.clone()).size(14))
        })
        .push({
            let label = "创建时间";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(created_at.clone()).size(14))
        })
        .push({
            let label = "更新时间";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(updated_at.clone()).size(14))
        })
        .into();

    // 密钥信息卡片内容
    let key_card_content: Element<'_, MessageCategory, StyleType> = Column::new()
        .spacing(8)
        .push(text("密钥信息").size(16))
        .push(Space::with_height(Length::Fixed(4.0)))
        .push({
            let label = "密钥哈希";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(key_hash.clone()).size(14))
        })
        .push({
            let label = "批准时间";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(approved_at.clone()).size(14))
        })
        .push({
            let label = "批准人";
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(container(text(label).size(14)).width(Length::Fixed(100.0)).align_x(Alignment::End))
                .push(text(approved_by.clone()).size(14))
        })
        .into();

    // 主内容
    Column::new()
        .spacing(15)
        .push(
            row!(
                text("Agent 详情").size(20),
                Space::with_width(Length::Fill),
                button("返回")
                    .on_press(MessageCategory::Agent(AgentMessage::CloseAgentDetail))
                    .width(Length::Fixed(80.0)),
            )
            .align_y(Alignment::Center),
        )
        .push(Space::with_height(Length::Fixed(10.0)))
        .push(Container::new(basic_card_content).width(Length::Fill).class(ContainerType::Standard).padding(15))
        .push(Container::new(system_card_content).width(Length::Fill).class(ContainerType::Standard).padding(15))
        .push(Container::new(connection_card_content).width(Length::Fill).class(ContainerType::Standard).padding(15))
        .push(Container::new(key_card_content).width(Length::Fill).class(ContainerType::Standard).padding(15))
        .push(Space::with_height(Length::Fixed(10.0)))
        .push(action_buttons(agent))
        .into()
}

/// 操作按钮行
fn action_buttons(agent: &Agent) -> Element<'_, MessageCategory, StyleType> {
    Row::new()
        .spacing(10)
        .push(
            if agent.approval_state == AgentApprovalState::Pending {
                button("批准")
                    .on_press(MessageCategory::Agent(AgentMessage::ApproveAgent(agent.id.to_string())))
            } else {
                button("已批准").width(Length::Fixed(80.0))
            },
        )
        .push(Space::with_width(Length::Fixed(10.0)))
        .push(
            if agent.approval_state == AgentApprovalState::Pending {
                button("拒绝")
                    .on_press(MessageCategory::Agent(AgentMessage::DenyAgent(agent.id.to_string())))
            } else {
                button("已拒绝").width(Length::Fixed(80.0))
            },
        )
        .push(Space::with_width(Length::Fixed(10.0)))
        .push(
            button("删除")
                .on_press(MessageCategory::Agent(AgentMessage::DeleteAgent(agent.id.to_string()))),
        )
        .into()
}
