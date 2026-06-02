# Domain Agent 使用手册

Domain Agent 是一个轻量级的客户端代理程序，通过 WebSocket 连接到 Domain Manager Hub。

## 功能特性

- **自动重连**: 网络断开后自动重连，采用指数退避策略
- **代理支持**: 支持 SOCKS5 和 HTTP CONNECT 代理
- **反向隧道**: 支持反向隧道，让 Hub 可以主动连接内网服务
- **P2P连接**: 支持 NAT 打洞，实现 Agent 之间的直接连接

## 快速开始

### 1. 编译

```bash
cargo build --release
```

### 2. 配置

创建配置文件 `agent.toml`：

```toml
[agent]
hub = "localhost:8080"
name = "my-agent"
key = "my-secret-key"
```

### 3. 运行

```bash
./target/release/domain_agent --config agent.toml
```

## 启动方式

### 配置文件 (推荐)

```toml
[agent]
hub = "hub.example.com:8080"
name = "my-agent"
key = "secret-key"

[proxy]
type = "none"  # socks5 | http | none

[reconnection]
base_delay_ms = 1000
max_delay_ms = 300000
max_retries = 0
jitter = 0.1

[tunnel]
port = 0  # 0 = disabled

[p2p]
port = 0  # 0 = disabled
```

启动：
```bash
./domain_agent --config agent.toml
```

### 环境变量

```bash
export DOMAIN_AGENT_HUB=hub.example.com:8080
export DOMAIN_AGENT_NAME=my-agent
export DOMAIN_AGENT_KEY=secret

./domain_agent
```

### 命令行参数

```bash
./domain_agent \
    --hub hub.example.com:8080 \
    --name my-agent \
    --key secret \
    --tunnel-port 8081 \
    --p2p-port 9000
```

### 配置优先级

CLI args > 环境变量 > 配置文件 > 默认值

## Docker 部署

### Docker Compose

```yaml
version: "3.8"

services:
  domain_agent:
    image: domain_agent:latest
    restart: unless-stopped
    environment:
      - DOMAIN_AGENT_HUB=hub.example.com:8080
      - DOMAIN_AGENT_NAME=my-agent
      - DOMAIN_AGENT_KEY=secret
      - DOMAIN_AGENT_TUNNEL_PORT=8081
    volumes:
      - ./agent.toml:/app/agent.toml:ro
```

```bash
docker compose up -d
```

### 仅用环境变量

```bash
docker run -d \
  --name domain_agent \
  -e DOMAIN_AGENT_HUB=hub.example.com:8080 \
  -e DOMAIN_AGENT_NAME=my-agent \
  -e DOMAIN_AGENT_KEY=secret \
  domain_agent:latest
```

## 使用示例

### 基本连接

```bash
./domain_agent --hub localhost:8080 --name my-agent --key secret
```

### 通过代理连接

```toml
[proxy]
type = "socks5"

[socks5]
host = "proxy.example.com"
port = 1080
auth = { username = "user", password = "pass" }
```

### 反向隧道

让 Hub 主动连接 Agent 后面的内网服务：

```bash
./domain_agent --hub hub.example.com:8080 --name my-agent --key secret --tunnel-port 8081
```

### P2P 连接

```bash
./domain_agent --hub hub.example.com:8080 --name my-agent --key secret --p2p-port 9000
```

## 配置详解

### agent 部分

| 字段 | 必填 | 说明 |
|------|------|------|
| hub | 是 | Hub 地址 (host:port) |
| name | 是 | Agent 名称 |
| key | 是 | 认证密钥 |

### proxy 部分

```toml
# 无代理 (默认)
[proxy]
type = "none"

# SOCKS5 代理
[proxy]
type = "socks5"
[socks5]
host = "proxy.example.com"
port = 1080
auth = { username = "user", password = "pass" }

# HTTP CONNECT 代理
[proxy]
type = "http"
[http]
host = "proxy.example.com"
port = 8080
auth = { username = "user", password = "pass" }
```

### reconnection 部分

```toml
[reconnection]
base_delay_ms = 1000    # 初始重连延迟 (毫秒)
max_delay_ms = 300000   # 最大重连延迟 (毫秒)
max_retries = 0         # 最大重试次数 (0=无限)
jitter = 0.1            # 抖动比例 (0.0-1.0)
```

### tunnel 部分

```toml
[tunnel]
port = 8081  # 0 = 禁用反向隧道
```

### p2p 部分

```toml
[p2p]
port = 9000  # 0 = 禁用 P2P
```

## 环境变量

| 变量 | 说明 |
|------|------|
| `DOMAIN_AGENT_HUB` | Hub 地址 |
| `DOMAIN_AGENT_NAME` | Agent 名称 |
| `DOMAIN_AGENT_KEY` | 认证密钥 |
| `DOMAIN_AGENT_TUNNEL_PORT` | 反向隧道端口 |
| `DOMAIN_AGENT_P2P_PORT` | P2P 端口 |
| `RUST_LOG` | 日志级别 (info, debug) |

## 网络架构

### 直接连接

```
┌─────────┐         ┌─────────┐
│  Agent  │────────▶│   Hub   │
└─────────┘  WebSocket  └─────────┘
```

### 通过代理连接

```
┌─────────┐     ┌──────────┐     ┌─────────┐
│  Agent  │────▶│  Proxy   │────▶│   Hub   │
└─────────┘     └──────────┘     └─────────┘
```

### 反向隧道

```
┌─────────┐         ┌─────────┐         ┌─────────┐
│  内网   │◀───────│  Agent  │───────▶│   Hub   │
│  服务   │   TCP   └─────────┘ WebSocket  └─────────┘
                                      │
                                      ▼
                               ┌─────────┐
                               │  外网   │
                               │  客户端  │
                               └─────────┘
```

### P2P 连接

```
┌─────────┐              ┌─────────┐              ┌─────────┐
│ Agent A │◀── signaling ───▶│   Hub   │◀── signaling ───▶│ Agent B │
│         │     (relay)       │         │     (relay)       │         │
│    ◀═══════════════════════════════════════════════════════▶      │
│                      P2P 直接连接                            │
└─────────┘                                                 └─────────┘
```

## 状态机

Agent 连接状态：

```
Disconnected ──[connect()]──> Connecting
    ▲                           │
    │                           ▼
    │                   Connected ──[Register]──> Registered
    │                       │              │
    │                       │              ▼
    │                       │        PendingApproval (if requires_approval)
    │                       │
    │<──────[disconnect()]──┴──[error]──> Error
    │                           │
    │<────────[reconnect()]──────┘
```

## 故障排查

### 连接失败

1. 检查 Hub 地址是否正确
2. 检查网络连通性
3. 检查防火墙是否阻止

### 代理问题

1. 确认代理服务器可用
2. 检查代理认证信息
3. 确认代理协议版本

### Docker 部署问题

```bash
# 查看日志
docker compose logs

# 检查配置
docker compose config

# 重建镜像
docker compose build --no-cache
```

## 部署文档

详细部署说明见 [DEPLOY.md](./DEPLOY.md)。

## 许可证

Mulan PSL v2
