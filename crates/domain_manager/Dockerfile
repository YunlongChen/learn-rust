# 多阶段构建 Dockerfile
# 用于构建 Domain Manager 的 Docker 镜像

# 构建阶段
FROM rust:1.75-slim as builder

# 设置工作目录
WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建一个虚拟的 main.rs 来缓存依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 构建依赖（这一层会被缓存）
RUN cargo build --release && rm -rf src

# 复制源代码
COPY src ./src
COPY build.rs ./
COPY resources ./resources
COPY config ./config
COPY locales ./locales

# 构建应用程序
RUN cargo build --release

# 运行时阶段
FROM debian:bookworm-slim as runtime

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libfontconfig1 \
    libfreetype6 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 domainmgr

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/domain_manager /app/domain_manager

# 复制资源文件
COPY --from=builder /app/resources /app/resources
COPY --from=builder /app/config /app/config
COPY --from=builder /app/locales /app/locales

# 设置权限
RUN chown -R domainmgr:domainmgr /app
RUN chmod +x /app/domain_manager

# 切换到非 root 用户
USER domainmgr

# 暴露端口（如果应用有 web 界面）
EXPOSE 8080

# 设置环境变量
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep domain_manager || exit 1

# 启动命令
CMD ["./domain_manager"]

# 添加标签
LABEL maintainer="Domain Manager Team"
LABEL description="Domain Manager - 域名和DNS管理工具"
LABEL version="1.0.0"