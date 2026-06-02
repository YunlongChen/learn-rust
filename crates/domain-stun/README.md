# Domain STUN 服务设计

## 概述

Domain STUN 是一个用于 NAT 穿透的服务器，提供：
1. **STUN Server** - 帮助客户端发现公网 IP 和 NAT 类型
2. **TURN Relay** - 当 P2P 直接连接失败时，提供流量中继

## 架构

```
┌─────────┐         ┌───────────┐         ┌─────────┐
│ Agent A │────────▶│ STUN/TURN  │◀────────│ Agent B │
│         │◀────────│  Server   │────────▶│         │
└─────────┘   TURN   └───────────┘  TURN   └─────────┘
              Relay         ▲
                            │
                            │
                     ┌─────────────┐
                     │   Web UI   │
                     │  (管理界面) │
                     └─────────────┘
```

## 功能模块

### 1. STUN 模块
- 监听 UDP 3478 端口（STUN 标准端口）
- 响应 Binding Request
- 返回公网 IP 和端口
- 支持 Binding Indication（保活）

### 2. TURN 模块
- 监听 UDP 3478 端口（与 STUN 共用）
- Allocation 管理（创建/刷新/删除）
- 数据转发（Relay）

### 3. Web API 模块
- Agent 注册
- 连接状态查询
- TURN 中继分配管理

### 4. Web UI 模块
- 仪表盘（连接数、流量统计）
- Agent 列表
- 实时日志

## 接口设计

### Web API Endpoints

#### 1. Agent 注册
```
POST /api/v1/agent/register
Content-Type: application/json

{
  "agent_id": "uuid",
  "name": "agent-1",
  "public_addr": "1.2.3.4:5678",
  "nat_type": "symmetric"
}

Response:
{
  "code": 0,
  "message": "success",
  "data": {
    "relay_token": "xxx"
  }
}
```

#### 2. 获取 STUN 服务器信息
```
GET /api/v1/stun/info

Response:
{
  "code": 0,
  "data": {
    "stun_addr": "stun.example.com:3478",
    "turn_addr": "turn.example.com:3478",
    "public_ip": "5.6.7.8"
  }
}
```

#### 3. 创建 TURN 分配
```
POST /api/v1/turn/allocation
Content-Type: application/json
X-Relay-Token: xxx

{
  "agent_id": "uuid",
  "requested_lifetime": 600
}

Response:
{
  "code": 0,
  "data": {
    "allocation_id": "xxx",
    "mapped_addr": "5.6.7.8:12345",
    "relayed_addr": "5.6.7.8:54321",
    "lifetime": 600
  }
}
```

#### 4. 刷新分配
```
REFRESH /api/v1/turn/allocation/{id}

Response:
{
  "code": 0,
  "data": {
    "lifetime": 600
  }
}
```

#### 5. 获取 Agent 列表
```
GET /api/v1/agents

Response:
{
  "code": 0,
  "data": {
    "agents": [
      {
        "id": "uuid",
        "name": "agent-1",
        "public_addr": "1.2.3.4:5678",
        "nat_type": "symmetric",
        "connected_at": "2024-01-01T00:00:00Z",
        "last_seen": "2024-01-01T00:05:00Z"
      }
    ]
  }
}
```

### UDP Protocol (STUN/TURN)

#### STUN Message Format (RFC 5389)
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|0 0|     STUN Message Type        |         Message Length    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                     Transaction ID (96 bits)                   |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                      Attributes (variable)                    |
|                                                               |
```

#### Message Types
- 0x0001: Binding Request
- 0x0101: Binding Response
- 0x0111: Binding Error Response
- 0x0002: Allocate Request (TURN)
- 0x0102: Allocate Response (TURN)
- 0x0112: Allocate Error Response (TURN)

#### Attributes
- 0x0001: MAPPED-ADDRESS
- 0x0002: XOR-MAPPED-ADDRESS
- 0x0003: RESPONSE-ADDRESS
- 0x0004: CHANGE-REQUEST
- 0x0005: SOURCE-ADDRESS
- 0x0006: USERNAME
- 0x0008: MESSAGE-INTEGRITY
- 0x0009: ERROR-CODE
- 0x000C: REALM
- 0x000D: NONCE
- 0x0010: XOR-RELAYED-ADDRESS
- 0x0018: REQUESTED-ADDRESS-FAMILY
- 0x0013: LIFETIME
- 0x0015: CHANNEL-NUMBER
- 0x0016: BANDWIDTH

## Web UI 设计

### 页面结构

```
/
├── Dashboard (/)
│   ├── 连接统计
│   ├── 流量统计
│   └── 在线 Agent 地图
│
├── Agents (/agents)
│   ├── Agent 列表
│   ├── Agent 详情
│   └── Agent 操作
│
├── Relay (/relay)
│   ├── 活动分配
│   ├── 带宽使用
│   └── 分配历史
│
├── Logs (/logs)
│   ├── 实时日志
│   └── 日志搜索
│
└── Settings (/settings)
    ├── 服务器配置
    ├── STUN/TURN 配置
    └── 日志级别
```

### Dashboard 页面

```
┌─────────────────────────────────────────────────────────────┐
│  [Logo] Domain STUN                    [Agent] [Settings]   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ Online   │ │ Total    │ │ Bandwidth│ │ Allocations│   │
│  │ Agents   │ │ Requests  │ │   Used   │ │  Active   │   │
│  │   12     │ │  1,234   │ │ 125 MB   │ │    5      │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │
│                                                             │
│  STUN/TURN Status                                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ STUN Server: Active (UDP:3478)                     │   │
│  │ TURN Relay: Active (5 allocations)                │   │
│  │ Public IP: 5.6.7.8                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Recent Activity                                           │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 10:30:45  Agent-3 registered                        │   │
│  │ 10:30:12  Allocation created for Agent-5             │   │
│  │ 10:29:58  Agent-2 heartbeat                        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 技术栈

- **Web 框架**: Actix-web
- **前端**: Tera 模板 (与 domain_server 保持一致)
- **数据库**: SQLite (轻量，与 domain_manager 保持一致)
- **异步运行时**: Tokio

## 项目结构

```
crates/domain-stun/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── stun/
│   │   ├── mod.rs
│   │   ├── message.rs      # STUN 消息编解码
│   │   ├── handler.rs      # STUN 请求处理
│   │   └── attributes.rs    # STUN 属性定义
│   ├── turn/
│   │   ├── mod.rs
│   │   ├── allocation.rs   # TURN 分配管理
│   │   └── relay.rs        # 数据中继
│   ├── api/
│   │   ├── mod.rs
│   │   ├── agent.rs        # Agent API
│   │   ├── stun.rs         # STUN API
│   │   └── turn.rs         # TURN API
│   ├── db/
│   │   ├── mod.rs
│   │   └── models.rs       # 数据模型
│   ├── handlers/
│   │   ├── mod.rs
│   │   └── ui.rs            # Web UI handlers
│   └── templates/          # Tera 模板
│       ├── base.html
│       ├── dashboard.html
│       ├── agents.html
│       └── ...
└── templates/               # 模板目录
```

## 配置

```yaml
server:
  host: "0.0.0.0"
  port: 3479  # Web UI 端口

stun:
  bind: "0.0.0.0:3478"
  realm: "domain-stun"

turn:
  bind: "0.0.0.0:3478"
  max_allocations: 1000
  default_lifetime: 600

database:
  url: "sqlite:domain-stun.db"
```

## 验证步骤

1. 启动 `domain-stun`
2. 访问 Web UI `http://localhost:3479`
3. 启动 Agent 连接 STUN 服务器
4. 在 Web UI 查看 Agent 列表
5. 测试 TURN 中继功能
