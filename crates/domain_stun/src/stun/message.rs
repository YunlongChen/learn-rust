//! STUN message parsing and encoding

use bytes::{Buf, BufMut, BytesMut};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StunError {
    #[error("Invalid STUN header")]
    InvalidHeader,
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u16),
    #[error("Message too short: {0}")]
    MessageTooShort(usize),
    #[error("Invalid attribute: {0}")]
    InvalidAttribute(u16),
    #[error("Invalid fingerprint")]
    InvalidFingerprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StunMessageType {
    BindingRequest,
    BindingResponse,
    BindingErrorResponse,
    AllocateRequest,
    AllocateResponse,
    AllocateErrorResponse,
    RefreshRequest,
    RefreshResponse,
    RefreshErrorResponse,
    ChannelBindRequest,
    ChannelBindResponse,
    ChannelBindErrorResponse,
    DataIndication,
    SendIndication,
}

impl StunMessageType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0001 => Some(StunMessageType::BindingRequest),
            0x0101 => Some(StunMessageType::BindingResponse),
            0x0111 => Some(StunMessageType::BindingErrorResponse),
            0x0003 => Some(StunMessageType::AllocateRequest),
            0x0103 => Some(StunMessageType::AllocateResponse),
            0x0113 => Some(StunMessageType::AllocateErrorResponse),
            0x0004 => Some(StunMessageType::RefreshRequest),
            0x0104 => Some(StunMessageType::RefreshResponse),
            0x0114 => Some(StunMessageType::RefreshErrorResponse),
            0x0009 => Some(StunMessageType::ChannelBindRequest),
            0x0109 => Some(StunMessageType::ChannelBindResponse),
            0x0119 => Some(StunMessageType::ChannelBindErrorResponse),
            0x0015 => Some(StunMessageType::DataIndication),
            0x0006 => Some(StunMessageType::SendIndication),
            _ => None,
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            StunMessageType::BindingRequest => 0x0001,
            StunMessageType::BindingResponse => 0x0101,
            StunMessageType::BindingErrorResponse => 0x0111,
            StunMessageType::AllocateRequest => 0x0003,
            StunMessageType::AllocateResponse => 0x0103,
            StunMessageType::AllocateErrorResponse => 0x0113,
            StunMessageType::RefreshRequest => 0x0004,
            StunMessageType::RefreshResponse => 0x0104,
            StunMessageType::RefreshErrorResponse => 0x0114,
            StunMessageType::ChannelBindRequest => 0x0009,
            StunMessageType::ChannelBindResponse => 0x0109,
            StunMessageType::ChannelBindErrorResponse => 0x0119,
            StunMessageType::DataIndication => 0x0015,
            StunMessageType::SendIndication => 0x0006,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StunAttribute {
    pub attr_type: u16,
    pub value: Vec<u8>,
}

impl StunAttribute {
    pub fn new(attr_type: u16, value: Vec<u8>) -> Self {
        Self { attr_type, value }
    }
}

#[derive(Debug, Clone)]
pub struct StunMessage {
    pub message_type: StunMessageType,
    pub transaction_id: [u8; 12],
    pub attributes: Vec<StunAttribute>,
}

impl StunMessage {
    pub fn parse(data: &[u8]) -> Result<Self, StunError> {
        if data.len() < 20 {
            return Err(StunError::MessageTooShort(data.len()));
        }

        let mut buf = data;
        let first_word = buf.get_u16();
        let msg_type = StunMessageType::from_u16(first_word & 0x3FFF)
            .ok_or(StunError::InvalidMessageType(first_word))?;
        let msg_length = buf.get_u16() as usize;

        if msg_length > data.len() - 20 {
            return Err(StunError::MessageTooShort(data.len()));
        }

        let mut transaction_id = [0u8; 12];
        buf.copy_to_slice(&mut transaction_id);

        let mut attributes = Vec::new();
        let remaining = &data[20..20 + msg_length];
        let mut attr_buf = remaining;

        while attr_buf.len() >= 4 {
            let attr_type = attr_buf.get_u16();
            let attr_length = attr_buf.get_u16() as usize;
            let padding = (4 - (attr_length % 4)) % 4;

            if attr_buf.len() < attr_length + padding {
                break;
            }

            let mut attr_value = vec![0u8; attr_length];
            attr_value.copy_from_slice(&attr_buf[..attr_length]);
            attr_buf = &attr_buf[attr_length + padding..];

            attributes.push(StunAttribute::new(attr_type, attr_value));
        }

        Ok(StunMessage {
            message_type: msg_type,
            transaction_id,
            attributes,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(20 + self.attributes.len() * 64);

        let type_and_class = (self.message_type.to_u16() & 0x3FFF) | 0x0000;
        buf.put_u16(type_and_class);

        let attr_bytes: usize = self.attributes.iter()
            .map(|a| 4 + a.value.len() + (4 - (a.value.len() % 4)) % 4)
            .sum();
        buf.put_u16(attr_bytes as u16);

        buf.put_slice(&self.transaction_id);

        for attr in &self.attributes {
            buf.put_u16(attr.attr_type);
            buf.put_u16(attr.value.len() as u16);
            buf.put_slice(&attr.value);
            let padding = (4 - (attr.value.len() % 4)) % 4;
            for _ in 0..padding {
                buf.put_u8(0);
            }
        }

        buf.to_vec()
    }

    pub fn get_attribute(&self, attr_type: u16) -> Option<&StunAttribute> {
        self.attributes.iter().find(|a| a.attr_type == attr_type)
    }

    pub fn add_attribute(&mut self, attr: StunAttribute) {
        self.attributes.push(attr);
    }
}

pub fn make_error_response(msg: &StunMessage, code: u16, reason: &str) -> Vec<u8> {
    let error_class = (code / 100) as u8;
    let error_number = (code % 100) as u8;

    let mut reason_bytes = reason.as_bytes().to_vec();
    while reason_bytes.len() < 128 {
        reason_bytes.push(0);
    }

    let mut error_attr = vec![0u8; 4 + reason_bytes.len()];
    error_attr[0] = 0;
    error_attr[1] = 0;
    error_attr[2] = error_class;
    error_attr[3] = error_number;
    error_attr[4..].copy_from_slice(&reason_bytes);

    let response_type = match msg.message_type {
        StunMessageType::BindingRequest => StunMessageType::BindingErrorResponse,
        StunMessageType::AllocateRequest => StunMessageType::AllocateErrorResponse,
        StunMessageType::RefreshRequest => StunMessageType::RefreshErrorResponse,
        StunMessageType::ChannelBindRequest => StunMessageType::ChannelBindErrorResponse,
        _ => StunMessageType::BindingErrorResponse,
    };

    let mut response = StunMessage {
        message_type: response_type,
        transaction_id: msg.transaction_id,
        attributes: vec![
            StunAttribute::new(0x0009, error_attr),
        ],
    };

    response.to_bytes()
}

pub fn make_binding_response(msg: &StunMessage, mapped_addr: SocketAddr) -> Vec<u8> {
    let mut addr_bytes = vec![0u8; 8];
    addr_bytes[0] = 0x00;
    addr_bytes[1] = 0x01;
    addr_bytes[2..4].copy_from_slice(&mapped_addr.port().to_be_bytes());

    let ip_octets = match mapped_addr.ip() {
        IpAddr::V4(ipv4) => ipv4.octets(),
        IpAddr::V6(_) => [0, 0, 0, 0],
    };
    addr_bytes[4..8].copy_from_slice(&ip_octets);

    let response = StunMessage {
        message_type: StunMessageType::BindingResponse,
        transaction_id: msg.transaction_id,
        attributes: vec![
            StunAttribute::new(0x0020, addr_bytes),
        ],
    };

    response.to_bytes()
}
