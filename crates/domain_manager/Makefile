# Domain Manager Makefile
# 提供常用的构建、测试和部署命令

.PHONY: help build build-release build-debug clean test check fmt clippy run install docker-build docker-run docker-stop release-windows release-linux release-all

# 默认目标
help:
	@echo "Domain Manager 构建工具"
	@echo ""
	@echo "可用命令:"
	@echo "  build          - 构建项目（debug模式）"
	@echo "  build-release  - 构建项目（release模式）"
	@echo "  build-debug    - 构建项目（debug模式）"
	@echo "  clean          - 清理构建缓存"
	@echo "  test           - 运行测试"
	@echo "  check          - 代码检查（fmt + clippy）"
	@echo "  fmt            - 格式化代码"
	@echo "  clippy         - 运行 clippy 检查"
	@echo "  run            - 运行应用程序"
	@echo "  install        - 安装到系统"
	@echo ""
	@echo "Docker 命令:"
	@echo "  docker-build   - 构建 Docker 镜像"
	@echo "  docker-run     - 运行 Docker 容器"
	@echo "  docker-stop    - 停止 Docker 容器"
	@echo ""
	@echo "发布命令:"
	@echo "  release-windows - 构建 Windows 发布版本"
	@echo "  release-linux   - 构建 Linux 发布版本"
	@echo "  release-all     - 构建所有平台发布版本"

# 构建命令
build: build-debug

build-release:
	@echo "构建 release 版本..."
	cargo build --release

build-debug:
	@echo "构建 debug 版本..."
	cargo build

# 清理
clean:
	@echo "清理构建缓存..."
	cargo clean
	rm -rf release/

# 测试
test:
	@echo "运行测试..."
	cargo test --verbose

# 代码检查
check: fmt clippy

fmt:
	@echo "格式化代码..."
	cargo fmt --all

clippy:
	@echo "运行 clippy 检查..."
	cargo clippy --all-targets --all-features -- -D warnings

# 运行
run:
	@echo "运行应用程序..."
	cargo run

run-release:
	@echo "运行 release 版本..."
	cargo run --release

# 安装
install:
	@echo "安装到系统..."
	cargo install --path .

# Docker 命令
docker-build:
	@echo "构建 Docker 镜像..."
	docker build -t domain-manager:latest .

docker-run:
	@echo "运行 Docker 容器..."
	docker-compose up -d

docker-stop:
	@echo "停止 Docker 容器..."
	docker-compose down

docker-logs:
	@echo "查看 Docker 日志..."
	docker-compose logs -f domain-manager

# 发布构建
release-windows:
	@echo "构建 Windows 发布版本..."
	@if command -v pwsh >/dev/null 2>&1; then \
		pwsh -File scripts/build-windows.ps1; \
	else \
		echo "错误: 需要 PowerShell 来构建 Windows 版本"; \
		exit 1; \
	fi

release-linux:
	@echo "构建 Linux 发布版本..."
	@if [ -f scripts/build-linux.sh ]; then \
		bash scripts/build-linux.sh; \
	else \
		echo "错误: 未找到 Linux 构建脚本"; \
		exit 1; \
	fi

release-all: release-windows release-linux
	@echo "所有平台构建完成"

# 开发环境设置
dev-setup:
	@echo "设置开发环境..."
	@echo "安装 Rust 组件..."
	rustup component add rustfmt clippy
	@echo "检查系统依赖..."
	@if command -v pkg-config >/dev/null 2>&1; then \
		echo "✓ pkg-config 已安装"; \
	else \
		echo "✗ 需要安装 pkg-config"; \
	fi

# 性能分析
profile:
	@echo "运行性能分析..."
	cargo build --release
	perf record --call-graph=dwarf target/release/domain_manager
	perf report

# 基准测试
bench:
	@echo "运行基准测试..."
	cargo bench

# 文档生成
docs:
	@echo "生成文档..."
	cargo doc --open

# 依赖更新
update-deps:
	@echo "更新依赖..."
	cargo update
	cargo audit

# 安全检查
security-check:
	@echo "运行安全检查..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "安装 cargo-audit: cargo install cargo-audit"; \
		cargo install cargo-audit; \
		cargo audit; \
	fi

# 代码覆盖率
coverage:
	@echo "生成代码覆盖率报告..."
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		cargo tarpaulin --out Html; \
	else \
		echo "安装 cargo-tarpaulin: cargo install cargo-tarpaulin"; \
		cargo install cargo-tarpaulin; \
		cargo tarpaulin --out Html; \
	fi