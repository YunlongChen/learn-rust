# NetOps - 项目路线图

> 网络运维综合管理平台 | DDNS + Agent 管理 + 组网 + 网站/容器管理

## 更新日志

| 日期         | 版本 | 变更                                   |
|------------|----|--------------------------------------|
| 2026-06-07 | -  | 重命名项目为 NetOps，整合 Agent 管理、网站/容器管理路线图 |

---

## 当前版本状态

**最新版本**: v0.2.x (开发中)

| 模块                      | 状态      | 说明                                      |
|-------------------------|---------|-----------------------------------------|
| Domain Manager (GUI)    | 🟡 活跃开发 | 阿里云/Cloudflare DNS 管理，Iced GUI          |
| Domain Agent Management | 🟢 基础完成 | gRPC/REST/WebSocket 服务，CRUD/审批/健康分/生命周期 |
| Domain Agent            | 🟡 基础完成 | WebSocket 连接、重连、反向隧道、P2P 打洞             |
| Domain STUN             | 🟡 基础完成 | STUN/TURN 服务器框架                         |
| Domain Server           | ⚪ 框架    | 通用服务器框架                                 |
| Website 管理              | ⚪ 规划中   | 域名监控、SSL 证书管理                           |

---

## 第一阶段：基础域名管理 ✅

**状态**: 基本完成

### Domain Manager GUI

- [x] **核心框架**
    - [x] 窗口大小/位置记忆
    - [x] 自适应布局基础
    - [x] 多主题系统 (Nord, Dracula, Gruvbox, Solarized, Modern, A11y)
    - [x] 国际化 (中/英)
- [x] **账户管理**
    - [x] 阿里云 API 接入
    - [x] Cloudflare API 适配
    - [x] API 密钥安全存储
- [x] **域名操作**
    - [x] 域名列表展示
    - [x] DNS 记录管理 (CRUD)
    - [x] 域名状态查询
    - [x] 分线路解析配置
- [x] **自动发布**
    - [x] Windows 打包 (WiX)
    - [x] Release 构建优化 (LTO, 裁剪)

---

## 第二阶段：Agent 生命周期管理 ✅

**状态**: 基础完成，2026-Q2

### Agent Management Server

- [x] **通信协议**
    - [x] gRPC 接口 (Tonic)
    - [x] REST API (Axum)
    - [x] WebSocket 支持
- [x] **Agent 注册与审批**
    - [x] Agent 注册请求
    - [x] 手动审批/拒绝
    - [x] 自动审批 (可选)
- [x] **生命周期管理**
    - [x] 状态机 (Pending → Approved → Active → Offline → Denied)
    - [x] 事件记录 (注册、审批、心跳、断连)
- [x] **健康监控**
    - [x] 系统信息收集 (OS、网络、进程、资源)
    - [x] 健康评分算法
    - [x] 历史评分查询

### Domain Agent Client

- [x] **连接管理**
    - [x] WebSocket 连接 Hub
    - [x] 指数退避重连
    - [x] 心跳机制
- [x] **隧道功能**
    - [x] 反向 TCP 隧道
    - [x] SOCKS5 代理
    - [x] HTTP 代理
- [x] **P2P 连接**
    - [x] NAT 类型检测
    - [x] STUN 打洞
    - [x] P2P 直连

### STUN/TURN 服务器

- [x] **基础功能**
    - [x] STUN 服务器
    - [x] TURN 中继
    - [x] SQLite 存储

---

## 第三阶段：高级功能 🚧

**目标**: 2026-Q3

### Website 管理

- [ ] **域名监控**
    - [ ] 域名到期提醒
    - [ ] DNS 传播状态
    - [ ] 健康检查 (HTTP/S)
- [ ] **SSL 证书**
    - [ ] Let's Encrypt 申请
    - [ ] 自动续期
    - [ ] 证书状态监控
- [ ] **DNS 高级**
    - [ ] 批量操作
    - [ ] DNSSEC 管理
    - [ ] TTL 优化

### Agent 增强

- [ ] **自动化运维**
    - [ ] 定时任务调度
    - [ ] 告警规则引擎
    - [ ] 远程命令执行
- [ ] **团队协作**
    - [ ] 多用户支持
    - [ ] 角色权限控制
    - [ ] 操作审计日志

---

## 第四阶段：容器与网络管理 📋

**目标**: 2026-Q4

### Container 管理

- [ ] **Docker 集成**
    - [ ] 容器列表与状态
    - [ ] 容器生命周期 (启/停/删)
    - [ ] 日志查看
    - [ ] 镜像管理
- [ ] **编排支持**
    - [ ] Docker Compose 支持
    - [ ] 批量操作
    - [ ] 健康检查配置

### 网络拓扑

- [ ] **节点发现**
    - [ ] 自动发现局域网节点
    - [ ] Agent 节点注册
    - [ ] 拓扑可视化
- [ ] **流量管理**
    - [ ] 连接状态监控
    - [ ] 流量统计
    - [ ] 带宽控制

---

## 第五阶段：企业级能力 📋

**目标**: 2027-Q1

- [ ] **高可用**
    - [ ] 多 Hub 集群
    - [ ] 故障转移
    - [ ] 负载均衡
- [ ] **安全增强**
    - [ ] mTLS 双向认证
    - [ ] 端到端加密
    - [ ] 密钥轮换
- [ ] **生态集成**
    - [ ] Webhook 通知
    - [ ] GitOps 配置同步
    - [ ] API 开放平台

---

## 技术债务与优化

| 优先级 | 项目                                | 状态  |
|-----|-----------------------------------|-----|
| 高   | Domain Manager V2 重构 (manager_v2) | 进行中 |
| 中   | 前端界面开发 (agent-management)         | 待启动 |
| 中   | 单元测试覆盖率提升                         | 待规划 |
| 低   | Linux/macOS 打包                    | 待启动 |

---

## 依赖技术栈

| 类别         | 技术                           |
|------------|------------------------------|
| **语言**     | Rust 2021                    |
| **GUI**    | Iced                         |
| **数据库**    | SQLite / PostgreSQL (SeaORM) |
| **异步**     | Tokio                        |
| **Web 框架** | Axum, Actix-web              |
| **RPC**    | gRPC (Tonic), Prost          |
| **序列化**    | Serde, JSON                  |
| **容器**     | bollard (Docker API)         |

---

## 文档计划

| 文档                   | 状态                                |
|----------------------|-----------------------------------|
| 用户快速入门指南             | ✅ README.md                       |
| Domain Manager 使用指南  | ✅ crates/domain-manager/README.md |
| STUN/TURN 部署指南       | ✅ crates/domain-stun/DEPLOY.md    |
| Agent 使用手册           | ✅ crates/domain-agent/README.md   |
| Agent Management API | ⚠️ 待完善                            |
| 管理员部署手册              | ⚪ 待编写                             |
| Docker 部署指南          | ⚪ 待编写                             |

---

> **更新策略**: 本路线图根据开发进度动态调整，欢迎 Issue 反馈和 PR 贡献。
