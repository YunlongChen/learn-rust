# Domain Manager

一个基于Rust和Iced GUI框架开发的域名管理工具，支持多个DNS服务提供商的域名和DNS记录管理。

## 功能特性

- 🌐 **多DNS提供商支持**: 支持阿里云DNS、Cloudflare等主流DNS服务
- 🎨 **现代化GUI**: 基于Iced框架的跨平台图形界面
- 🔧 **完整DNS管理**: 支持域名和DNS记录的增删改查操作
- 🌍 **国际化支持**: 多语言界面支持
- 📊 **可视化**: 集成图表和数据可视化功能
- 🔒 **安全**: 使用secrecy库安全处理敏感信息

## 快速开始

### 环境要求

- Rust 1.70+
- 系统依赖（Linux）:
  - `libgtk-3-dev`
  - `libxcb-render0-dev`
  - `libxcb-shape0-dev`
  - `libxcb-xfixes0-dev`

### 构建和运行

#### 使用构建脚本（推荐）

```bash
# Windows
powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1

# Linux/macOS
bash ./scripts/build-linux.sh
```

#### 手动构建

```bash
# 开发版本
cargo build

# 发布版本
cargo build --release

# 运行
cargo run
```

### Docker 支持

```bash
# 构建镜像
docker build -t domain-manager .

# 使用 Docker Compose
docker-compose up -d
```

## 项目结构

```
domain_manager/
├── src/                    # 源代码
├── resources/              # 资源文件
│   ├── icons/             # 图标资源
│   ├── fonts/             # 字体文件
│   ├── sounds/            # 音效文件
│   └── migrations/        # 数据库迁移文件
├── config/                # 配置文件
├── locales/               # 国际化文件
├── scripts/               # 构建脚本
│   ├── build.ps1         # 跨平台构建脚本
│   ├── build-windows.ps1 # Windows构建脚本
│   └── build-linux.sh    # Linux构建脚本
├── .github/workflows/     # CI/CD配置
├── Dockerfile             # Docker配置
├── docker-compose.yml     # Docker Compose配置
└── Makefile              # Make构建配置
```

## 开发指南

### 代码格式化
```bash
cargo fmt
```

### 代码检查
```bash
cargo clippy
```

### 运行测试
```bash
cargo test
```

### 数据库迁移

- 生成新的迁移文件
    ```sh
    cargo run -- generate MIGRATION_NAME
    ```
- 应用所有待处理的迁移
    ```sh
    cargo run -- up
    ```
- 回滚最后应用的迁移
    ```sh
    cargo run -- down
    ```
- 检查迁移状态
    ```sh
    cargo run -- status
    ```

## 配置

应用程序配置文件位于 `config/` 目录下，支持以下配置：

- DNS服务提供商API密钥
- 界面主题和语言设置
- 数据库连接配置
- 日志级别设置

## 贡献

欢迎提交Issue和Pull Request来帮助改进项目。

## 许可证

本项目采用开源许可证，具体请查看LICENSE文件。
