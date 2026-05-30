//! Agent 心跳检测
//! 
//! 管理Agent心跳和超时检测

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 心跳配置
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// 心跳间隔（秒）
    pub interval_seconds: u64,
    /// 超时阈值（秒）
    pub timeout_seconds: u64,
    /// 最大重连次数
    pub max_retries: u32,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 30,
            timeout_seconds: 60,
            max_retries: 3,
        }
    }
}

/// Agent心跳状态
#[derive(Debug, Clone)]
pub struct HeartbeatStatus {
    pub agent_id: Uuid,
    pub last_heartbeat: DateTime<Utc>,
    pub timeout_count: u32,
    pub status: HeartbeatState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeartbeatState {
    Active,
    Timeout,
    Disconnected,
}

/// 心跳管理器
pub struct HeartbeatManager {
    config: HeartbeatConfig,
    statuses: Arc<RwLock<HashMap<Uuid, HeartbeatStatus>>>,
}

impl HeartbeatManager {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            config,
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册Agent心跳
    pub async fn register(&self, agent_id: Uuid) {
        let status = HeartbeatStatus {
            agent_id,
            last_heartbeat: Utc::now(),
            timeout_count: 0,
            status: HeartbeatState::Active,
        };
        let mut statuses = self.statuses.write().await;
        statuses.insert(agent_id, status);
    }
    
    /// 更新心跳
    pub async fn heartbeat(&self, agent_id: Uuid) {
        let mut statuses = self.statuses.write().await;
        if let Some(status) = statuses.get_mut(&agent_id) {
            status.last_heartbeat = Utc::now();
            status.timeout_count = 0;
            status.status = HeartbeatState::Active;
        }
    }
    
    /// 检查超时
    pub async fn check_timeouts(&self) -> Vec<Uuid> {
        let mut timed_out = Vec::new();
        let now = Utc::now();
        
        let mut statuses = self.statuses.write().await;
        for (agent_id, status) in statuses.iter_mut() {
            let elapsed = now.signed_duration_since(status.last_heartbeat);
            if elapsed.num_seconds() as u64 > self.config.timeout_seconds {
                status.timeout_count += 1;
                if status.timeout_count >= self.config.max_retries {
                    status.status = HeartbeatState::Timeout;
                    timed_out.push(*agent_id);
                }
            }
        }
        
        timed_out
    }
    
    /// 获取Agent心跳状态
    pub async fn get_status(&self, agent_id: &Uuid) -> Option<HeartbeatState> {
        let statuses = self.statuses.read().await;
        statuses.get(agent_id).map(|s| s.status.clone())
    }
    
    /// 注销Agent
    pub async fn unregister(&self, agent_id: &Uuid) {
        let mut statuses = self.statuses.write().await;
        statuses.remove(agent_id);
    }
}

impl Default for HeartbeatManager {
    fn default() -> Self {
        Self::new(HeartbeatConfig::default())
    }
}
