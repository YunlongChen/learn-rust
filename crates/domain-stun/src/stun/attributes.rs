//! STUN attribute types (RFC 5389)

pub const MAPPED_ADDRESS: u16 = 0x0001;
pub const XOR_MAPPED_ADDRESS: u16 = 0x0020;
pub const RESPONSE_ADDRESS: u16 = 0x0002;
pub const CHANGE_REQUEST: u16 = 0x0003;
pub const SOURCE_ADDRESS: u16 = 0x0004;
pub const USERNAME: u16 = 0x0006;
pub const MESSAGE_INTEGRITY: u16 = 0x0008;
pub const ERROR_CODE: u16 = 0x0009;
pub const REALM: u16 = 0x0014;
pub const NONCE: u16 = 0x0015;
pub const XOR_RELAYED_ADDRESS: u16 = 0x0016;
pub const REQUESTED_ADDRESS_FAMILY: u16 = 0x0017;
pub const LIFETIME: u16 = 0x000D;
pub const CHANNEL_NUMBER: u16 = 0x000C;
pub const BANDWIDTH: u16 = 0x0010;
pub const XOR_PEER_ADDRESS: u16 = 0x0012;
pub const DATA: u16 = 0x0013;
pub const MAGIC_COOKIE: u32 = 0x2112A442;

#[derive(Debug, Clone)]
pub struct MappedAddressAttr {
    pub family: u8,
    pub port: u16,
    pub ip: [u8; 4],
}

impl MappedAddressAttr {
    pub fn new(port: u16, ip: [u8; 4]) -> Self {
        Self { family: 0x01, port, ip }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = vec![0u8; 8];
        buf[0] = 0x00;
        buf[1] = self.family;
        buf[2..4].copy_from_slice(&self.port.to_be_bytes());
        buf[4..8].copy_from_slice(&self.ip);
        buf
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        Some(Self {
            family: data[1],
            port: u16::from_be_bytes([data[2], data[3]]),
            ip: [data[4], data[5], data[6], data[7]],
        })
    }
}

#[derive(Debug, Clone)]
pub struct XorMappedAddressAttr {
    pub family: u8,
    pub port: u16,
    pub ip: [u8; 4],
}

impl XorMappedAddressAttr {
    pub fn new(port: u16, ip: [u8; 4]) -> Self {
        Self { family: 0x01, port, ip }
    }

    pub fn encode(&self, _transaction_id: &[u8; 12]) -> Vec<u8> {
        let mut buf = vec![0u8; 8];
        buf[0] = 0x00;
        buf[1] = self.family;

        let xor_port = self.port ^ ((MAGIC_COOKIE >> 16) as u16);
        buf[2..4].copy_from_slice(&xor_port.to_be_bytes());

        let mut xor_ip = [0u8; 4];
        let cookie_bytes = (MAGIC_COOKIE as u32).to_be_bytes();
        for i in 0..4 {
            xor_ip[i] = self.ip[i] ^ cookie_bytes[i];
        }
        buf[4..8].copy_from_slice(&xor_ip);
        buf
    }
}

#[derive(Debug, Clone)]
pub struct ErrorCodeAttr {
    pub error_class: u8,
    pub number: u8,
    pub reason: String,
}

impl ErrorCodeAttr {
    pub fn new(error_class: u8, number: u8, reason: String) -> Self {
        Self { error_class, number, reason }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut reason_bytes = self.reason.as_bytes().to_vec();
        while reason_bytes.len() < 128 {
            reason_bytes.push(0);
        }

        let mut buf = vec![0u8; 4 + reason_bytes.len()];
        buf[2] = self.error_class;
        buf[3] = self.number;
        buf[4..].copy_from_slice(&reason_bytes);
        buf
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }
        let reason = String::from_utf8_lossy(&data[4..]).trim_end_matches('\0').to_string();
        Some(Self {
            error_class: data[2],
            number: data[3],
            reason,
        })
    }

    pub fn code(&self) -> u16 {
        (self.error_class as u16) * 100 + self.number as u16
    }
}

#[derive(Debug, Clone)]
pub struct LifetimeAttr(pub u32);

impl LifetimeAttr {
    pub fn new(seconds: u32) -> Self {
        Self(seconds)
    }

    pub fn encode(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    pub fn decode(data: &[u8]) -> Option<u32> {
        if data.len() < 4 {
            return None;
        }
        Some(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }
}

#[derive(Debug, Clone)]
pub struct ChannelNumberAttr(pub u16);

impl ChannelNumberAttr {
    pub fn new(channel: u16) -> Self {
        Self(channel)
    }

    pub fn encode(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}
