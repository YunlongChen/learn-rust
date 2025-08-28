# Domain Manager 重构计划

## 当前问题分析

### 1. 文件规模问题
- `manager.rs` 文件共 2398 行，过于庞大
- 单一文件承担了过多职责，违反了单一职责原则
- 代码维护困难，可读性差

### 2. 职责混合问题
通过代码分析发现，`manager.rs` 文件混合了以下职责：

#### UI 渲染职责
- `view()` 方法：主界面渲染逻辑
- `domain_detail()` 方法：域名详情页面渲染
- `get_custom_button()` 方法：自定义按钮组件
- 各种页面布局和样式处理

#### 业务逻辑职责
- `update()` 方法：处理 50+ 种不同的消息类型
- DNS 记录同步逻辑
- 域名管理逻辑
- 账户管理逻辑

#### 数据管理职责
- 数据库连接管理
- 数据加载和缓存
- 状态管理（域名列表、DNS记录、配置等）

#### 事件处理职责
- 键盘事件处理
- 窗口事件处理
- 用户交互事件处理

### 3. 具体问题点

#### 消息处理过于复杂
`update()` 方法处理了 50+ 种消息类型，包括：
- 页面导航消息
- 数据同步消息
- 表单处理消息
- 窗口管理消息
- 配置管理消息

#### 异步操作分散
多个 `handle_*` 方法分散在文件各处：
- `handle_reload()`
- `handle_sync_domain()`
- `handle_dns_reload()`
- `handle_dns_record_add()`
- `handle_dns_record_delete()`

#### 状态管理复杂
`DomainManager` 结构体包含 20+ 个字段，管理各种状态：
- UI 状态（当前页面、主题、语言等）
- 业务数据（域名列表、DNS记录、账户信息等）
- 配置信息（窗口状态、背景配置等）

## 重构方案

### 1. 模块化架构设计

```
src/gui/
├── manager.rs          # 主管理器（简化后）
├── state/              # 状态管理模块
│   ├── mod.rs
│   ├── app_state.rs    # 应用状态
│   ├── ui_state.rs     # UI状态
│   └── data_state.rs   # 数据状态
├── handlers/           # 事件处理模块
│   ├── mod.rs
│   ├── message_handler.rs    # 消息处理
│   ├── sync_handler.rs       # 同步处理
│   ├── domain_handler.rs     # 域名处理
│   ├── dns_handler.rs        # DNS处理
│   └── window_handler.rs     # 窗口处理
├── services/           # 业务服务模块
│   ├── mod.rs
│   ├── domain_service.rs     # 域名服务
│   ├── dns_service.rs        # DNS服务
│   ├── sync_service.rs       # 同步服务
│   └── config_service.rs     # 配置服务
├── components/         # UI组件模块
│   ├── mod.rs
│   ├── domain_list.rs        # 域名列表组件
│   ├── dns_records.rs        # DNS记录组件
│   ├── forms/               # 表单组件
│   │   ├── mod.rs
│   │   ├── add_domain.rs
│   │   ├── add_dns.rs
│   │   └── add_provider.rs
│   └── layout/              # 布局组件
│       ├── mod.rs
│       ├── header.rs
│       ├── sidebar.rs
│       └── footer.rs
└── pages/              # 页面模块（已存在）
    ├── mod.rs
    ├── domain_page.rs
    ├── dns_page.rs
    └── settings.rs
```

### 2. 状态管理重构

#### 2.1 分离状态结构

```rust
// state/app_state.rs
pub struct AppState {
    pub ui: UiState,
    pub data: DataState,
    pub config: Config,
}

// state/ui_state.rs
pub struct UiState {
    pub current_page: Page,
    pub last_page: Option<Page>,
    pub theme: Theme,
    pub locale: Locale,
    pub message: String,
    pub is_syncing: bool,
    pub toast_visible: bool,
    pub toast_message: Option<String>,
}

// state/data_state.rs
pub struct DataState {
    pub domain_list: Vec<Domain>,
    pub domain_providers: Vec<DnsProvider>,
    pub dns_records: Vec<DnsRecord>,
    pub filter: Filter,
    pub stats: DomainStats,
    pub connection: Option<DatabaseConnection>,
}
```

#### 2.2 状态管理方法

```rust
impl AppState {
    pub fn new() -> Self { ... }
    pub fn reset(&mut self) { ... }
    pub fn update_ui(&mut self, ui_update: UiUpdate) { ... }
    pub fn update_data(&mut self, data_update: DataUpdate) { ... }
}
```

### 3. 消息处理重构

#### 3.1 消息分类

```rust
// handlers/message_handler.rs
pub enum MessageCategory {
    Navigation(NavigationMessage),
    Domain(DomainMessage),
    Dns(DnsMessage),
    Sync(SyncMessage),
    Window(WindowMessage),
    Config(ConfigMessage),
}

pub struct MessageHandler {
    domain_handler: DomainHandler,
    dns_handler: DnsHandler,
    sync_handler: SyncHandler,
    window_handler: WindowHandler,
}

impl MessageHandler {
    pub fn handle_message(&self, state: &mut AppState, message: Message) -> Task<Message> {
        match message.categorize() {
            MessageCategory::Navigation(msg) => self.handle_navigation(state, msg),
            MessageCategory::Domain(msg) => self.domain_handler.handle(state, msg),
            MessageCategory::Dns(msg) => self.dns_handler.handle(state, msg),
            MessageCategory::Sync(msg) => self.sync_handler.handle(state, msg),
            MessageCategory::Window(msg) => self.window_handler.handle(state, msg),
            MessageCategory::Config(msg) => self.handle_config(state, msg),
        }
    }
}
```

#### 3.2 专门的处理器

```rust
// handlers/domain_handler.rs
pub struct DomainHandler {
    domain_service: DomainService,
}

impl DomainHandler {
    pub fn handle(&self, state: &mut AppState, message: DomainMessage) -> Task<Message> {
        match message {
            DomainMessage::Add(domain) => self.add_domain(state, domain),
            DomainMessage::Delete(id) => self.delete_domain(state, id),
            DomainMessage::Select(domain) => self.select_domain(state, domain),
            // ...
        }
    }
}
```

### 4. 业务服务重构

#### 4.1 域名服务

```rust
// services/domain_service.rs
pub struct DomainService {
    connection: Option<DatabaseConnection>,
}

impl DomainService {
    pub async fn load_domains(&self, filter: &Filter) -> Result<Vec<Domain>, ServiceError> { ... }
    pub async fn add_domain(&self, domain: NewDomain) -> Result<Domain, ServiceError> { ... }
    pub async fn delete_domain(&self, id: i32) -> Result<(), ServiceError> { ... }
    pub async fn sync_domains(&self, client: &DnsClient) -> Result<Vec<Domain>, ServiceError> { ... }
}
```

#### 4.2 DNS服务

```rust
// services/dns_service.rs
pub struct DnsService {
    connection: Option<DatabaseConnection>,
}

impl DnsService {
    pub async fn load_dns_records(&self, domain: &str) -> Result<Vec<DnsRecord>, ServiceError> { ... }
    pub async fn add_dns_record(&self, record: NewDnsRecord) -> Result<DnsRecord, ServiceError> { ... }
    pub async fn delete_dns_record(&self, id: &str) -> Result<(), ServiceError> { ... }
    pub async fn sync_dns_records(&self, domain: &str, client: &DnsClient) -> Result<Vec<DnsRecord>, ServiceError> { ... }
}
```

### 5. UI组件重构

#### 5.1 域名列表组件

```rust
// components/domain_list.rs
pub struct DomainListComponent;

impl DomainListComponent {
    pub fn view<'a>(
        domains: &[Domain],
        selected: Option<&Domain>,
        font: Font,
    ) -> Element<'a, Message> {
        // 域名列表渲染逻辑
    }
}
```

#### 5.2 DNS记录组件

```rust
// components/dns_records.rs
pub struct DnsRecordsComponent;

impl DnsRecordsComponent {
    pub fn view<'a>(
        records: &[DnsRecord],
        font: Font,
    ) -> Element<'a, Message> {
        // DNS记录表格渲染逻辑
    }
}
```

### 6. 简化后的主管理器

```rust
// manager.rs (重构后)
pub struct DomainManager {
    state: AppState,
    message_handler: MessageHandler,
    ui_renderer: UiRenderer,
}

impl DomainManager {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
            message_handler: MessageHandler::new(),
            ui_renderer: UiRenderer::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.message_handler.handle_message(&mut self.state, message)
    }

    pub fn view(&self) -> Element<Message> {
        self.ui_renderer.render(&self.state)
    }

    // 其他必要的公共方法
    pub fn get_message(&self) -> &str {
        &self.state.ui.message
    }

    pub fn set_syncing(&mut self, syncing: bool) {
        self.state.ui.is_syncing = syncing;
    }
}
```

## 重构实施计划

### 阶段1：创建基础结构
1. 创建新的模块目录结构
2. 定义状态管理结构
3. 创建基础的服务接口

### 阶段2：迁移状态管理
1. 将 `DomainManager` 中的状态字段迁移到新的状态结构
2. 实现状态管理方法
3. 更新现有代码以使用新的状态结构

### 阶段3：重构消息处理
1. 创建消息分类系统
2. 实现专门的消息处理器
3. 逐步迁移 `update()` 方法中的逻辑

### 阶段4：提取业务服务
1. 创建域名服务
2. 创建DNS服务
3. 创建同步服务
4. 迁移相关的异步方法

### 阶段5：组件化UI
1. 提取可复用的UI组件
2. 重构页面渲染逻辑
3. 简化主 `view()` 方法

### 阶段6：测试和优化
1. 确保所有测试通过
2. 性能优化
3. 代码清理和文档更新

## 预期收益

### 1. 代码质量提升
- 单一职责原则：每个模块只负责特定功能
- 代码可读性：文件大小合理，逻辑清晰
- 可维护性：模块化设计便于维护和扩展

### 2. 开发效率提升
- 并行开发：不同开发者可以同时工作在不同模块
- 测试友好：模块化设计便于单元测试
- 调试便利：问题定位更加精确

### 3. 系统稳定性提升
- 错误隔离：模块间的错误不会相互影响
- 状态管理：集中的状态管理减少状态不一致问题
- 类型安全：更好的类型设计减少运行时错误

## 风险评估

### 1. 重构风险
- **风险**：重构过程中可能引入新的bug
- **缓解**：分阶段重构，每个阶段都要确保测试通过

### 2. 兼容性风险
- **风险**：现有功能可能受到影响
- **缓解**：保持公共API不变，内部重构

### 3. 时间成本
- **风险**：重构需要较长时间
- **缓解**：分阶段实施，可以逐步获得收益

## 总结

这个重构计划旨在将当前的单体 `manager.rs` 文件重构为模块化、可维护的架构。通过分离关注点、提取业务逻辑、组件化UI，我们可以显著提升代码质量和开发效率。

重构将分6个阶段进行，每个阶段都有明确的目标和可验证的结果。这样可以确保重构过程的可控性和安全性。