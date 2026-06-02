//! STUN message handler

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use super::{StunMessage, StunMessageType, StunAttribute, make_binding_response};

pub async fn handle_binding_request(
    msg: &StunMessage,
    from: &SocketAddr,
    _agents: &Arc<RwLock<HashMap<Uuid, crate::AgentInfo>>>,
) -> Option<Vec<u8>> {
    tracing::debug!("Handling binding request from {}", from);
    let response = make_binding_response(msg, *from);
    Some(response)
}

pub async fn handle_binding_indication(_msg: &StunMessage, from: &SocketAddr) {
    tracing::debug!("Binding indication from {}", from);
}

pub fn make_success_response(msg: &StunMessage, attrs: Vec<StunAttribute>) -> Vec<u8> {
    let response_type = match msg.message_type {
        StunMessageType::BindingRequest => StunMessageType::BindingResponse,
        StunMessageType::AllocateRequest => StunMessageType::AllocateResponse,
        StunMessageType::RefreshRequest => StunMessageType::RefreshResponse,
        StunMessageType::ChannelBindRequest => StunMessageType::ChannelBindResponse,
        _ => StunMessageType::BindingResponse,
    };

    let response = StunMessage {
        message_type: response_type,
        transaction_id: msg.transaction_id,
        attributes: attrs,
    };

    response.to_bytes()
}
