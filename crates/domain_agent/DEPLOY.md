# Domain Agent 部署指南

## 概述

Domain Agent 是一个轻量级的客户端代理程序，通过 WebSocket 连接到 Domain Manager Hub。

## 功能特性

- **自动重连**: 网络断开后自动重连，采用指数退避策略
- **代理支持**: 支持 SOCKS5 和 HTTP CONNECT 代理
- **反向隧道**: 支持反向隧道，让 Hub 可以主动连接内网服务
- **P2P连接**: 支持 NAT 打洞，实现 Agent 之间的直接连接

## 系统要求

### 硬件要求
- CPU: 1 核+
- 内存: 256MB+
- 磁盘: 50MB+

### 软件要求

#### 直接部署
- Rust 1.75+
- Linux/Unix/macOS/Windows

#### Docker 部署
- Docker 20.10+
- Docker Compose 2.0+

---

## 部署方式一：直接部署

### 1. 编译构建

```bash
# 克隆代码后，进入 domain_agent 目录
cd crates/domain_agent

# 编译 release 版本
cargo build --release

# 二进制文件位于
# target/release/domain_agent
```

### 2. 配置文件

创建配置文件 `agent.toml`：

```toml
[agent]
hub = "hub.example.com:8080"
name = "my-agent"
key = "your-secret-key"

# 可选：代理配置
[proxy]
type = "socks5"
host = "proxy.example.com"
port = 1080
# auth = { username = "user", password = "pass" }

# 可选：重连配置
[reconnection]
base_delay_ms = 1000
max_delay_ms = 300000
max_retries = 0
jitter = 0.1

# 可选：反向隧道
[tunnel]
port = 8081

# 可选：P2P
[p2p]
port = 9000
```

### 3. 运行

#### 使用配置文件启动

```bash
./target/release/domain_agent --config agent.toml
```

#### 使用命令行参数

```bash
./target/release/domain_agent \
    --hub hub.example.com:8080 \
    --name my-agent \
    --key your-secret-key
```

#### 使用环境变量

```bash
export DOMAIN_AGENT_HUB=hub.example.com:8080
export DOMAIN_AGENT_NAME=my-agent
export DOMAIN_AGENT_KEY=your-secret-key
./target/release/domain_agent
```

### 4. 后台运行（systemd）

创建服务文件 `/etc/systemd/system/domain_agent.service`：

```ini
[Unit]
Description=Domain Agent
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/domain_agent
ExecStart=/opt/domain_agent/domain_agent --config /opt/domain_agent/agent.toml
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable domain_agent
sudo systemctl start domain_agent

# 查看状态
sudo systemctl status domain_agent

# 查看日志
sudo journalctl -u domain_agent -f
```

---

## 部署方式二：Docker 部署

### 前置条件

确保已安装 Docker 和 Docker Compose：

```bash
docker --version
docker compose version
```

### 1. 构建镜像

```bash
cd crates/domain_agent
docker build -t domain_agent:latest .
```

### 2. 使用 Docker Compose

创建 `docker-compose.yml`：

```yaml
version: "3.8"

services:
  domain_agent:
    build: .
    image: domain_agent:latest
    container_name: domain_agent
    restart: unless-stopped
    environment:
      - DOMAIN_AGENT_HUB=hub.example.com:8080
      - DOMAIN_AGENT_NAME=my-agent
      - DOMAIN_AGENT_KEY=your-secret-key
      - DOMAIN_AGENT_TUNNEL_PORT=8081
      - DOMAIN_AGENT_P2P_PORT=9000
      - RUST_LOG=info
    volumes:
      - ./agent.toml:/app/agent.toml:ro
    networks:
      - agent_network

networks:
  agent_network:
    driver: bridge
```

创建配置文件 `agent.toml`：

```toml
[agent]
hub = "hub.example.com:8080"
name = "my-agent"
key = "your-secret-key"

[tunnel]
port = 8081

[p2p]
port = 9000
```

启动：

```bash
# 前台运行（查看日志）
docker compose up

# 后台运行
docker compose up -d

# 查看状态
docker compose ps

# 查看日志
docker compose logs -f
```

### 3. 仅使用环境变量（无需配置文件）

```bash
docker run -d \
  --name domain_agent \
  --restart unless-stopped \
  -e DOMAIN_AGENT_HUB=hub.example.com:8080 \
  -e DOMAIN_AGENT_NAME=my-agent \
  -e DOMAIN_AGENT_KEY=your-secret-key \
  -e DOMAIN_AGENT_TUNNEL_PORT=8081 \
  -e RUST_LOG=info \
  domain_agent:latest
```

### 4. 停止服务

```bash
docker compose down

# 删除容器和镜像
docker compose down --rmi local
```

---

## 配置说明

### 配置优先级

配置加载顺序（后者覆盖前者）：

1. **默认配置**
2. **配置文件** (`--config` 或默认路径)
3. **环境变量**
4. **命令行参数**

### 配置文件格式 (TOML)

```toml
[agent]
hub = "localhost:8080"      # Hub 地址 (必填)
name = "my-agent"           # Agent 名称 (必填)
key = "secret-key"          # 认证密钥 (必填)

[proxy]
type = "none"               # none | socks5 | http

[socks5]
host = "proxy.example.com"
port = 1080
auth = { username = "user", password = "pass" }

[http]
host = "proxy.example.com"
port = 8080
auth = { username = "user", password = "pass" }

[reconnection]
base_delay_ms = 1000        # 初始重连延迟 (毫秒)
max_delay_ms = 300000       # 最大重连延迟 (毫秒)
max_retries = 0             # 最大重试次数 (0=无限)
jitter = 0.1                # 抖动比例 (0.0-1.0)

[tunnel]
port = 8081                 # 反向隧道端口 (0=禁用)

[p2p]
port = 9000                 # P2P 监听端口 (0=禁用)
```

### 环境变量

| 变量 | 说明 | 示例 |
|------|------|------|
| `DOMAIN_AGENT_HUB` | Hub 地址 | `hub.example.com:8080` |
| `DOMAIN_AGENT_NAME` | Agent 名称 | `my-agent` |
| `DOMAIN_AGENT_KEY` | 认证密钥 | `secret-key` |
| `DOMAIN_AGENT_TUNNEL_PORT` | 反向隧道端口 | `8081` |
| `DOMAIN_AGENT_P2P_PORT` | P2P 端口 | `9000` |
| `RUST_LOG` | 日志级别 | `info`, `debug` |

### 命令行参数

| 参数 | 简写 | 说明 |
|------|------|------|
| `--config` | `-c` | 配置文件路径 |
| `--hub` | `-h` | Hub 地址 |
| `--name` | `-n` | Agent 名称 |
| `--key` | `-k` | 认证密钥 |
| `--tunnel-port` | `-t` | 反向隧道端口 |
| `--p2p-port` | `-p` | P2P 端口 |

---

## 使用示例

### 基本连接

```bash
# 命令行参数
./domain_agent --hub localhost:8080 --name agent-1 --key secret

# 环境变量
export DOMAIN_AGENT_HUB=localhost:8080
export DOMAIN_AGENT_NAME=agent-1
export DOMAIN_AGENT_KEY=secret
./domain_agent

# 配置文件
./domain_agent --config agent.toml
```

### 使用代理

```bash
# SOCKS5 代理
./domain_agent --config agent.toml

# agent.toml:
# [proxy]
# type = "socks5"
# [socks5]
# host = "proxy.example.com"
# port = 1080
# auth = { username = "user", password = "pass" }
```

### 反向隧道

```bash
# 启动 Agent，监听本地端口 8081
# Hub 可通过此端口连接 Agent 后面的内网服务
./domain_agent --hub hub.example.com:8080 --name agent-1 --key secret --tunnel-port 8081

# 或在配置文件中:
# [tunnel]
# port = 8081
```

### P2P 连接

```bash
# 启用 P2P 监听
./domain_agent --hub hub.example.com:8080 --name agent-1 --key secret --p2p-port 9000
```

---

## Docker 部署示例

### 基本部署

```bash
docker run -d \
  --name domain_agent \
  --restart unless-stopped \
  -e DOMAIN_AGENT_HUB=hub.example.com:8080 \
  -e DOMAIN_AGENT_NAME=my-agent \
  -e DOMAIN_AGENT_KEY=secret \
  domain_agent:latest
```

### 带反向隧道的部署

```bash
docker run -d \
  --name domain_agent \
  --restart unless-stopped \
  -e DOMAIN_AGENT_HUB=hub.example.com:8080 \
  -e DOMAIN_AGENT_NAME=my-agent \
  -e DOMAIN_AGENT_KEY=secret \
  -e DOMAIN_AGENT_TUNNEL_PORT=8081 \
  -p 8081:8081 \
  domain_agent:latest
```

### 使用 Docker Compose 完整部署

创建 `docker-compose.yml`：

```yaml
version: "3.8"

services:
  domain_agent:
    build: .
    image: domain_agent:latest
    container_name: domain_agent
    restart: unless-stopped
    environment:
      - DOMAIN_AGENT_HUB=${HUB_ADDRESS}
      - DOMAIN_AGENT_NAME=${AGENT_NAME}
      - DOMAIN_AGENT_KEY=${AGENT_KEY}
      - DOMAIN_AGENT_TUNNEL_PORT=${TUNNEL_PORT:-0}
      - DOMAIN_AGENT_P2P_PORT=${P2P_PORT:-0}
      - RUST_LOG=${RUST_LOG:-info}
    volumes:
      - ./agent.toml:/app/agent.toml:ro
    ports:
      - "8081:8081"  # Only if tunnel_port > 0
    networks:
      - agent_network

networks:
  agent_network:
    driver: bridge
```

创建 `.env` 文件：

```bash
HUB_ADDRESS=hub.example.com:8080
AGENT_NAME=my-agent
AGENT_KEY=secret
TUNNEL_PORT=8081
P2P_PORT=0
RUST_LOG=info
```

启动：

```bash
docker compose up -d
docker compose logs -f
```

---

## 验证测试

### 检查 Agent 状态

连接 Hub 后，Agent 会自动发送注册消息。如果连接成功，日志会显示：

```
Starting Domain Agent: name=my-agent, hub=hub.example.com:8080
Connected to Hub successfully
```

### Docker 健康检查

```bash
# 检查容器是否运行
docker ps | grep domain_agent

# 检查健康状态
docker inspect --format='{{.State.Health.Status}}' domain_agent
```

### 日志分析

```bash
# 直接部署
sudo journalctl -u domain_agent -f

# Docker 部署
docker compose logs -f domain_agent
```

---

## 常见问题

### Q1: 连接失败

```bash
# 检查 Hub 地址是否正确
# 检查网络连通性
telnet hub.example.com 8080

# 检查防火墙
sudo ufw allow 8080/tcp
```

### Q2: 配置文件不生效

```bash
# 确认配置文件路径正确
./domain_agent --config /path/to/agent.toml

# 检查配置文件格式
cat agent.toml | grep -v "^#" | grep -v "^$"
```

### Q3: Docker 容器启动失败

```bash
# 查看详细日志
docker compose logs domain_agent

# 检查环境变量
docker compose config
```

### Q4: 代理连接失败

```bash
# 确认代理服务器可用
curl --socks5 proxy.example.com:1080 http://example.com

# 检查代理认证信息
```

---

## 相关文档

- [README.md](./README.md) - 功能和使用说明
- [Domain STUN 部署](../domain_stun/DEPLOY.md) - STUN/TURN 服务器部署
