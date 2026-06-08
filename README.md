# NetOps - 网络运维综合管理平台

[![Mulan PSL v2](https://img.shields.io/badge/license-Mulan%20PSL%20v2-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://rust-lang.org)

NetOps 是一个用 Rust 编写的**网络运维综合管理平台**，专注于 DDNS、组网、Agent 管理、网站和容器管理。

## 核心功能

| 功能               | 说明                                      |
|------------------|-----------------------------------------|
| **DDNS 动态域名**    | 支持阿里云 DNS、Cloudflare 等多平台，自动更新域名解析      |
| **Agent 生命周期管理** | WebSocket/gRPC/REST 多协议支持，涵盖注册、审批、心跳、监控 |
| **NAT 穿透**       | STUN/TURN 服务器支持，P2P 连接和反向隧道             |
| **网站管理**         | 域名状态监控、SSL 证书管理、 DNS 记录批量操作             |
| **容器管理**         | Docker 容器状态监控和生命周期管理（规划中）               |
| **网络拓扑**         | 节点发现、连接管理、流量可视化（规划中）                    |

## 组件架构

```
┌─────────────────────────────────────────────────────────────┐
│                      NetOps Platform                        │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │Domain Manager│  │Agent Manager │  │  Domain Agent │    │
│  │   (GUI)      │  │   (Server)   │  │   (Client)    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│  ┌──────────────┐  ┌──────────────┐                       │
│  │Domain STUN   │  │Domain Server │                       │
│  │   (TURN)     │  │   (HTTP)     │                       │
│  └──────────────┘  └──────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

### 核心模块

| Crate                                              | 说明                                  |
|----------------------------------------------------|-------------------------------------|
| [domain-manager](crates/domain-manager)            | GUI 域名管理工具，支持阿里云 DNS、Cloudflare 等   |
| [domain-agent](crates/domain-agent)                | 轻量级客户端代理，支持反向隧道和 P2P NAT 打洞         |
| [domain-stun](crates/domain-stun)                  | STUN/TURN 服务器，用于 NAT 穿透和流量中继        |
| [domain-server](crates/domain-server)              | 通用服务器框架                             |
| [domain-http-server](crates/domain-http-server)    | HTTP 服务器实现                          |
| [domain-clients](crates/domain-clients)            | DNS API 客户端库                        |
| [agent-management](crates/domain-agent-management) | Agent 生命周期管理服务（gRPC/REST/WebSocket） |
| [agent-protocol](crates/domain-agent-protocol)     | Agent 通信协议定义                        |

## 快速开始

### 环境要求

- Rust 1.75+
- PostgreSQL 15+ (可选，默认使用 SQLite)
- Docker (可选，用于 STUN/TURN 服务)

### 构建

```bash
# 构建所有组件
cargo build --workspace

# 构建特定组件
cargo build -p domain-manager    # GUI 管理工具
cargo build -p agent-management  # Agent 管理服务
cargo build -p domain-stun       # STUN/TURN 服务
cargo build -p domain-agent      # Agent 客户端
```

### 运行

```bash
# 运行域名管理 GUI
cargo run -p domain-manager

# 运行 Agent 管理服务
cargo run -p agent-management

# 运行 STUN/TURN 服务器
cargo run -p domain-stun
```

## 技术栈

| 层级         | 技术                           |
|------------|------------------------------|
| **语言**     | Rust (2021 edition)          |
| **GUI**    | Iced                         |
| **数据库**    | SQLite / PostgreSQL (SeaORM) |
| **异步**     | Tokio                        |
| **Web 框架** | Axum, Actix-web              |
| **RPC**    | gRPC (Tonic)                 |
| **序列化**    | Prost, Serde                 |

## 项目状态

查看 [ROADMAP](crates/domain-manager/ROADMAP.md) 了解功能规划和开发进度。

## 文档

- [Domain Manager 使用指南](crates/domain-manager/README.md)
- [STUN/TURN 服务部署](crates/domain-stun/DEPLOY.md)
- [Agent 使用手册](crates/domain-agent/README.md)
- [Agent Management API](crates/domain-agent-management/API.md)

## 许可证

本项目基于 [Mulan PSL v2](LICENSE) 开源。
