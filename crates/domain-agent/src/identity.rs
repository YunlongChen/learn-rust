//! Agent 持久化身份管理模块
//!
//! 管理 Agent 的持久化 UUID 标识，存储在本地文件中

use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use anyhow::Result;

/// Agent 持久化身份
#[derive(Debug)]
pub struct AgentIdentity {
    /// Agent 的持久化 UUID
    id: Uuid,
    /// 身份文件路径
    path: PathBuf,
}

impl AgentIdentity {
    /// 从配置目录加载或创建新的身份标识
    pub fn load_or_create(config_dir: &PathBuf) -> Result<Self> {
        let path = config_dir.join("id");

        if path.exists() {
            // 读取已有的 UUID
            let content = fs::read_to_string(&path)?;
            let id = Uuid::parse_str(content.trim())?;
            Ok(Self { id, path })
        } else {
            // 生成新的 UUID
            let id = Uuid::new_v4();

            // 确保目录存在
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            // 写入文件
            fs::write(&path, id.to_string())?;

            Ok(Self { id, path })
        }
    }

    /// 获取 Agent 的持久化 ID
    pub fn id(&self) -> Uuid {
        self.id
    }
}