# Domain 项目

Rust 编写的分布式域名管理和服务穿透工具集。

## 组件

| Crate                                           | 说明                               |
|-------------------------------------------------|----------------------------------|
| [domain-manager](crates/domain-manager)         | GUI 域名管理工具，支持阿里云DNS、Cloudflare 等 |
| [domain-agent](crates/domain-agent)             | 轻量级客户端代理，支持反向隧道和 P2P NAT 打洞      |
| [domain-stun](crates/domain-stun)               | STUN/TURN 服务器，用于 NAT 穿透和流量中继     |
| [domain-server](crates/domain-server)           | 通用服务器框架                          |
| [domain-http-server](crates/domain-http-server) | HTTP 服务器实现                       |
| [domain-clients](crates/domain-clients)         | DNS API 客户端库                     |

## 快速开始

```bash
# 构建所有组件
cargo build --workspace

# 构建特定组件
cargo build -p domain-manager
cargo build -p domain-stun
cargo build -p domain-agent

# 开发检查
cargo check --workspace
```

## 技术栈

- **GUI**: Iced
- **数据库**: SQLite / PostgreSQL (SeaORM)
- **异步**: Tokio
- **序列化**: Serde

## 文档

- [Domain Manager 使用指南](crates/domain-manager/README.md)
- [STUN/TURN 服务部署](crates/domain-stun/DEPLOY.md)
- [Agent 使用手册](crates/domain-agent/README.md)

## 许可证

Mulan PSL v2
