# Agent Lifecycle & Management Service Design

## 1. System Architecture

```
┌─────────────────┐     ┌──────────────────────┐     ┌─────────────────┐
│  domain-agent   │────▶│  agent-management     │◀────│  domain-manager  │
│  (Agent 端)      │     │  (独立服务)            │     │  (作为客户端)    │
└─────────────────┘     └──────────────────────┘     └─────────────────┘
                                    │
                                    ▼
                          ┌──────────────────────┐
                          │     PostgreSQL        │
                          │  (事件/Agent数据)      │
                          └──────────────────────┘

协议通信：
- Agent ↔ agent-management : WebSocket (现有协议扩展)
- domain-manager ↔ agent-management : gRPC + REST
- 事件推送 : WebSocket 流 (通过 agent-management 中转)
```

**核心组件：**
- **agent-management**：独立进程，托管 Agent 注册审批、生命追踪、诊断数据收集
- **domain-agent-client**：供 domain-manager 使用的客户端库，封装 gRPC/REST 调用
- **agent-protocol**：共用协议定义 crate，Agent 和 management 服务共享

---

## 2. Agent Lifecycle State Machine

```
                    ┌─────────────┐
                    │  Created    │
                    └──────┬──────┘
                           │ 注册请求
                           ▼
               ┌────────────────────────┐
               │      Pending           │◀────────┐
               │   (待审批/待认证)        │         │ 拒绝/认证失败
               └───────────┬────────────┘         │
                         审批/认证通过             │
                           ▼                      │
               ┌────────────────────────┐         │
               │      Authorized        │─────────┘
               └───────────┬────────────┘
                         连接建立
                           ▼
               ┌────────────────────────┐
               │      Connected         │
               └───────────┬────────────┘
                         认证完成
                           ▼
               ┌────────────────────────┐
               │      Registered        │──────────┐
               └───────────┬────────────┘          │
                    正常运行中 │                    │ 断开/错误
                             ▼                    │
               ┌────────────────────────┐         │
               │      Reconnecting      │─────────┘
               └────────────────────────┘
                             │
                    重连成功 / 重连失败
                   /                  \
                  ▼                    ▼
           ┌───────────┐       ┌───────────┐
           │ Registered │       │  Closed   │
           └───────────┘       └───────────┘
```

**生命周期事件（完整审计日志）：**
- `AgentCreated` - Agent 记录被创建
- `AgentRegistering` - 注册请求接收
- `AgentAuthenticating` - 认证中
- `AgentPendingApproval` - 等待审批
- `AgentApproved` / `AgentDenied` - 审批结果
- `AgentConnected` - WebSocket 连接建立
- `AgentRegistered` - 注册完成
- `AgentReconnecting` - 尝试重连
- `AgentDisconnected` - 连接断开
- `AgentClosed` - 完全关闭
- `AgentError` - 发生错误

每条事件包含：
```json
{
  "event_id": "uuid",
  "agent_id": "uuid",
  "event_type": "AgentRegistered",
  "timestamp": "2026-06-04T10:30:00Z",
  "source": "agent|manager|system",
  "reason": "string",
  "metadata": { /* 事件相关数据 */ },
  "triggered_by": "user_id|system"
}
```

---

## 3. Diagnostic Information Collection Protocol

**Agent → management（定期上报，30s 间隔）：**
```json
{
  "type": "SystemInfoReport",
  "agent_id": "uuid",
  "timestamp": "2026-06-04T10:30:00Z",
  "data": {
    "os": {
      "type": "linux",
      "distro": "Ubuntu 22.04",
      "kernel": "5.15.0-generic",
      "architecture": "x86_64",
      "hostname": "worker-node-01"
    },
    "environment": {
      "env_vars": ["PATH=...", "HOME=..."],
      "current_working_dir": "/opt/agent",
      "user": "agent",
      "uid_gid": "1001/1001"
    },
    "process": {
      "pid": 1234,
      "parent_pid": 1,
      "command": "/usr/local/bin/agent",
      "start_time": "2026-06-04T08:00:00Z",
      "uptime_seconds": 9000
    },
    "network": {
      "interfaces": [
        {"name": "eth0", "ip": "10.0.0.5", "mac": "..."}
      ],
      "connections": [
        {"proto": "tcp", "local": "10.0.0.5:8080", "remote": "10.0.0.1:443", "state": "ESTABLISHED"}
      ]
    },
    "resources": {
      "cpu": { "usage_percent": 12.5, "cores": 4 },
      "memory": { "used_bytes": 1073741824, "total_bytes": 8589934592 },
      "disk": { "used_bytes": 5368709120, "total_bytes": 21474836480 }
    }
  }
}
```

**management → Agent（按需查询）：**
```json
{
  "type": "SystemInfoQuery",
  "request_id": "uuid",
  "query": {
    "info_types": ["process_list", "open_files", "network_connections", "env_vars"]
  }
}
```

**Agent → management（查询响应）：**
```json
{
  "type": "SystemInfoResponse",
  "request_id": "uuid",
  "data": { /* 完整诊断信息 */ }
}
```

---

## 4. gRPC/REST API Design

**服务定义（gRPC）：**
```protobuf
service AgentManagement {
  // Agent 注册（WebSocket 升级前预处理）
  rpc RegisterAgent(RegisterRequest) returns (RegisterResponse);

  // Agent CRUD
  rpc GetAgent(GetAgentRequest) returns (Agent);
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse);
  rpc UpdateAgent(UpdateAgentRequest) returns (Agent);
  rpc DeleteAgent(DeleteAgentRequest) returns (Empty);

  // 审批管理
  rpc ApproveAgent(ApproveRequest) returns (Empty);
  rpc DenyAgent(DenyRequest) returns (Empty);

  // 诊断信息
  rpc GetAgentSystemInfo(GetSystemInfoRequest) returns (SystemInfo);
  rpc QueryAgentSystemInfo(QuerySystemInfoRequest) returns (SystemInfo);
  rpc StreamAgentEvents(StreamEventsRequest) returns (stream AgentEvent);

  // 连接健康
  rpc GetAgentHealth(GetAgentHealthRequest) returns (HealthScore);
  rpc StreamAgentHealth(StreamHealthRequest) returns (stream HealthScore);

  // 生命周期
  rpc GetAgentLifecycleEvents(GetLifecycleRequest) returns (LifecycleEventsResponse);
  rpc StreamLifecycleEvents(StreamLifecycleRequest) returns (stream LifecycleEvent);
}
```

**REST API 端点：**
```
POST   /api/v1/agents/register     # 注册新 Agent
GET    /api/v1/agents               # 列表查询
GET    /api/v1/agents/{id}          # 获取详情
PATCH  /api/v1/agents/{id}          # 更新信息
DELETE /api/v1/agents/{id}          # 删除

POST   /api/v1/agents/{id}/approve  # 审批通过
POST   /api/v1/agents/{id}/deny     # 审批拒绝

GET    /api/v1/agents/{id}/system-info       # 获取系统信息
POST   /api/v1/agents/{id}/system-info/query # 按需查询
GET    /api/v1/agents/{id}/health            # 健康评分
WS     /ws/v1/agents/{id}/events            # 事件流

GET    /api/v1/agents/{id}/lifecycle         # 生命周期事件
WS     /ws/v1/agents/{id}/lifecycle         # 生命周期事件流
```

---

## 5. Data Models

**agents 表：**
```sql
CREATE TABLE agents (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  endpoint VARCHAR(512),
  status VARCHAR(32) NOT NULL,
  capabilities JSONB,
  tags JSONB,
  cert_fingerprint VARCHAR(128),

  approval_state VARCHAR(32) NOT NULL,
  approved_at TIMESTAMP,
  approved_by VARCHAR(255),
  denied_reason TEXT,

  auth_method VARCHAR(32),
  agent_key_hash VARCHAR(128),

  last_heartbeat TIMESTAMP,
  last_connected_at TIMESTAMP,
  connection_count INT DEFAULT 0,

  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

**agent_lifecycle_events 表：**
```sql
CREATE TABLE agent_lifecycle_events (
  id UUID PRIMARY KEY,
  agent_id UUID NOT NULL REFERENCES agents(id),
  event_type VARCHAR(64) NOT NULL,
  timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
  source VARCHAR(32),
  reason TEXT,
  metadata JSONB,
  triggered_by VARCHAR(255),
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_events_agent_time ON agent_lifecycle_events(agent_id, timestamp DESC);
```

**agent_system_info 表：**
```sql
CREATE TABLE agent_system_info (
  id UUID PRIMARY KEY,
  agent_id UUID NOT NULL REFERENCES agents(id),
  reported_at TIMESTAMP NOT NULL DEFAULT NOW(),
  info_type VARCHAR(32),
  os_info JSONB,
  environment_info JSONB,
  process_info JSONB,
  network_info JSONB,
  resource_info JSONB,
  raw_data JSONB
);
CREATE INDEX idx_sysinfo_agent_time ON agent_system_info(agent_id, reported_at DESC);
```

**agent_health_scores 表：**
```sql
CREATE TABLE agent_health_scores (
  id UUID PRIMARY KEY,
  agent_id UUID NOT NULL REFERENCES agents(id),
  scored_at TIMESTAMP NOT NULL DEFAULT NOW(),
  overall_score INT NOT NULL,
  latency_ms INT,
  jitter_ms INT,
  packet_loss_percent DECIMAL(5,2),
  bandwidth_kbps INT,
  component_scores JSONB
);
CREATE INDEX idx_health_agent_time ON agent_health_scores(agent_id, scored_at DESC);
```

---

## 6. Storage Strategy (Hot/Warm/Cold)

```
热数据 (最近 1 小时):
  agent_lifecycle_events → 内存队列 + Redis
  agent_health_scores   → 内存缓存

温数据 (7 天):
  agent_lifecycle_events → PostgreSQL
  agent_system_info     → PostgreSQL (定期报告)
  agent_health_scores   → PostgreSQL (采样保留)

冷数据 (30 天+):
  agent_system_info     → 可选压缩归档到对象存储
  清理策略: 30天前报告数据降采样或删除
```

---

## 7. Security (mTLS)

```
认证流程：
1. Agent 启动时加载 mTLS 证书（agent.crt + agent.key）
2. 建立 WebSocket 连接时进行 TLS 双向认证
3. 证书指纹在注册时与预存记录匹配
4. 可选：注册后颁发短期 session token 用于 API 访问

证书管理（后续迭代）：
- CA 签发 Agent 证书
- 证书撤销列表 (CRL)
- 自动续期机制
```

---

## 8. Implementation Phases

### Phase 1: Core Infrastructure
- 创建 `agent-management` crate
- 定义 protobuf gRPC 接口
- 实现基础 REST API 端点
- 实现 PostgreSQL schema migrations

### Phase 2: Lifecycle Events
- 实现 Agent 状态机
- 实现事件记录机制
- 实现 WebSocket 事件流
- 实现生命周期事件查询 API

### Phase 3: Diagnostic Collection
- 扩展 Agent 协议（SystemInfoReport, SystemInfoQuery, SystemInfoResponse）
- 实现 Agent 端诊断信息采集
- 实现 management 端按需查询
- 实现定期上报处理

### Phase 4: Health Scoring
- 实现心跳增强（含 RTT/抖动/丢包）
- 实现健康评分算法
- 实现健康数据存储和查询

### Phase 5: Security (mTLS)
- 实现 mTLS 证书握手
- 实现证书指纹验证
- 实现可选 session token
