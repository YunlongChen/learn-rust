//! Agent 页面状态

use crate::agent::model::{Agent, Capability, AgentStatus};
use rand::Rng;

/// Agent 页面状态
#[derive(Debug, Clone)]
pub struct AgentPageState {
    /// 是否处于添加模式
    pub is_adding: bool,
    /// 是否处于编辑模式
    pub is_editing: bool,
    /// 当前编辑的Agent ID
    pub editing_agent_id: Option<String>,
    /// 新Agent名称
    pub new_agent_name: String,
    /// 新Agent端点 (host:port)
    pub new_agent_endpoint: String,
    /// Agent密钥
    pub new_agent_key: String,
    /// Agent列表
    pub agents: Vec<Agent>,
    /// 是否正在测试连接
    pub is_testing: bool,
    /// 测试结果消息
    pub test_result: Option<String>,
    /// 选中的Agent ID（用于详情视图）
    pub selected_agent_id: Option<String>,
    /// 是否显示详情视图
    pub showing_detail: bool,
}

impl Default for AgentPageState {
    fn default() -> Self {
        Self {
            is_adding: false,
            is_editing: false,
            editing_agent_id: None,
            new_agent_name: String::new(),
            new_agent_endpoint: String::from("localhost:8081"),
            new_agent_key: String::new(),
            agents: Vec::new(),
            is_testing: false,
            test_result: None,
            selected_agent_id: None,
            showing_detail: false,
        }
    }
}

impl AgentPageState {
    /// 切换添加模式
    pub fn toggle_add_mode(&mut self) {
        self.is_adding = !self.is_adding;
        self.test_result = None;
        if self.is_adding {
            self.new_agent_name = String::new();
            self.new_agent_endpoint = String::from("localhost:8081");
            self.new_agent_key = Self::generate_key();
            self.is_editing = false;
            self.editing_agent_id = None;
        }
    }

    /// 生成随机密钥
    fn generate_key() -> String {
        let key: [u8; 16] = rand::thread_rng().gen();
        base64::encode(key)
    }

    /// 取消添加
    pub fn cancel_add(&mut self) {
        self.is_adding = false;
        self.is_editing = false;
        self.editing_agent_id = None;
        self.new_agent_name = String::new();
        self.new_agent_endpoint = String::from("localhost:8081");
        self.new_agent_key = String::new();
        self.test_result = None;
    }

    /// 保存Agent
    pub fn save_agent(&mut self) {
        if self.new_agent_name.is_empty() {
            return;
        }
        // 创建新Agent并添加到列表
        let mut agent = Agent::new(
            self.new_agent_name.clone(),
            self.new_agent_endpoint.clone(),
        );
        // 设置默认能力
        agent.capabilities = vec![
            Capability::DdnsClient,
            Capability::ShellExecutor,
            Capability::SslValidator,
        ];
        // 默认设置为离线（因为只是配置了，还未实际连接）
        agent.status = AgentStatus::Offline;
        // 设置密钥哈希
        agent.agent_key_hash = Some(Self::hash_key(&self.new_agent_key));

        self.agents.push(agent);

        // 重置表单
        self.is_adding = false;
        self.is_editing = false;
        self.new_agent_name = String::new();
        self.new_agent_endpoint = String::from("localhost:8081");
        self.new_agent_key = String::new();
        self.test_result = None;
    }

    /// 创建Agent对象（用于保存到数据库）
    pub fn create_agent(&self) -> Option<Agent> {
        if self.new_agent_name.is_empty() {
            return None;
        }
        let mut agent = Agent::new(
            self.new_agent_name.clone(),
            self.new_agent_endpoint.clone(),
        );
        agent.capabilities = vec![
            Capability::DdnsClient,
            Capability::ShellExecutor,
            Capability::SslValidator,
        ];
        agent.status = AgentStatus::Offline;
        agent.agent_key_hash = Some(Self::hash_key(&self.new_agent_key));
        Some(agent)
    }

    /// 测试连接
    pub fn test_connection(&mut self) {
        self.is_testing = true;
        self.test_result = Some("正在测试连接...".to_string());

        // 验证输入
        if self.new_agent_endpoint.is_empty() {
            self.test_result = Some("错误: 请输入Hub地址".to_string());
            self.is_testing = false;
            return;
        }
        if self.new_agent_key.is_empty() {
            self.test_result = Some("错误: 请输入Agent密钥".to_string());
            self.is_testing = false;
            return;
        }

        // 实际测试连接需要在异步环境中进行
        // 这里设置一个待测试状态，GUI会显示正在连接
        // TODO: 实现真正的异步连接测试
        self.test_result = Some("测试功能开发中...".to_string());
        self.is_testing = false;
    }

    /// 对密钥进行哈希
    fn hash_key(key: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 删除Agent
    pub fn delete_agent(&mut self, id: &str) {
        self.agents.retain(|a| a.id.to_string() != id);
    }

    /// 选择Agent查看详情
    pub fn select_agent(&mut self, id: &str) {
        self.selected_agent_id = Some(id.to_string());
        self.showing_detail = true;
    }

    /// 关闭详情视图
    pub fn close_detail(&mut self) {
        self.selected_agent_id = None;
        self.showing_detail = false;
    }

    /// 获取选中的Agent
    pub fn get_selected_agent(&self) -> Option<&Agent> {
        if let Some(ref id) = self.selected_agent_id {
            self.agents.iter().find(|a| a.id.to_string() == *id)
        } else {
            None
        }
    }
}

mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(data: impl AsRef<[u8]>) -> String {
        let bytes = data.as_ref();
        let mut result = String::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            result.push(HEX_CHARS[(byte >> 4) as usize] as char);
            result.push(HEX_CHARS[(byte & 0xf) as usize] as char);
        }
        result
    }
}
