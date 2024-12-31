// YApi QuickType插件生成，具体参考文档:https://plugins.jetbrains.com/plugin/18847-yapi-quicktype/documentation

use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsOperateResponse {
    #[serde(rename = "TotalCount")]
    total_count: i64,

    #[serde(rename = "PageSize")]
    page_size: i64,

    #[serde(rename = "RequestId")]
    request_id: String,

    #[serde(rename = "PageNumber")]
    page_number: i64,

    #[serde(rename = "RecordLogs")]
    pub record_logs: RecordLogs,
}

impl From<String> for DnsOperateResponse {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordLogs {
    #[serde(rename = "RecordLog")]
    pub record_log: Vec<RecordLog>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordLog {
    #[serde(rename = "ActionTime")]
    pub action_time: String,

    #[serde(rename = "Action")]
    pub action: Action,

    #[serde(rename = "Message")]
    pub message: String,

    #[serde(rename = "ActionTimestamp")]
    pub action_timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    #[serde(rename = "ADD")]
    Add,

    #[serde(rename = "DEL")]
    Del,

    #[serde(rename = "UPDATE")]
    Update,

    #[serde(rename = "RESUME")]
    Resume,

    #[serde(rename = "PAUSE")]
    PAUSE,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Add => write!(f, "添加"),
            Action::Del => write!(f, "删除"),
            Action::Update => write!(f, "更新"),
            Action::Resume => write!(f, "恢复"),
            Action::PAUSE => write!(f, "暂停解析"),
        }
    }
}
