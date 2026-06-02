# Domain STUN 服务部署指南

## 概述

Domain STUN 是一个用于 NAT 穿透的 STUN/TURN 服务器，提供：
- **STUN Server** - 帮助客户端发现公网 IP 和 NAT 类型
- **TURN Relay** - 当 P2P 直接连接失败时，提供流量中继

## 系统要求

### 硬件要求
- CPU: 1 核+
- 内存: 512MB+
- 磁盘: 1GB+

### 软件要求

#### 直接部署
- Rust 1.75+
- SQLite3

#### Docker 部署
- Docker 20.10+
- Docker Compose 2.0+

### 网络要求

| 端口 | 协议 | 说明 |
|------|------|------|
| 3478 | UDP | STUN/TURN 服务 |
| 3479 | TCP | HTTP API / Web UI |

**注意**：这些端口需要在防火墙/安全组中开放。

---

## 部署方式一：直接部署

### 1. 编译构建

```bash
# 克隆代码后，进入 domain-stun 目录
cd crates/domain-stun

# 编译 release 版本
cargo build --release

# 二进制文件位于
# target/release/domain-stun
```

### 2. 创建配置目录

```bash
# 创建数据目录
sudo mkdir -p /opt/domain-stun/data
sudo mkdir -p /var/log/domain-stun

# 创建配置目录
sudo mkdir -p /etc/domain-stun
```

### 3. 创建配置（可选）

默认配置会自动使用以下端口：
- HTTP API: 3479
- STUN/TURN: 3478

如需自定义，可创建配置文件：

```bash
sudo nano /etc/domain-stun/config.toml
```

```toml
[server]
host = "0.0.0.0"
port = 3479

[stun]
bind_host = "0.0.0.0"
bind_port = 3478
realm = "domain-stun"

[turn]
bind_host = "0.0.0.0"
bind_port = 3478
max_allocations = 1000
default_lifetime = 600

[database]
url = "sqlite:/opt/domain-stun/data/domain-stun.db"
```

### 4. 设置权限

```bash
sudo chown -R $USER:$USER /opt/domain-stun
sudo chmod -R 755 /opt/domain-stun
```

### 5. 运行服务

#### 前台运行（测试用）

```bash
cd /opt/domain-stun
./target/release/domain-stun
```

#### 后台运行（systemd）

创建服务文件：

```bash
sudo nano /etc/systemd/system/domain-stun.service
```

```ini
[Unit]
Description=Domain STUN Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/domain-stun
ExecStart=/opt/domain-stun/domain-stun
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable domain-stun
sudo systemctl start domain-stun

# 查看状态
sudo systemctl status domain-stun
```

---

## 部署方式二：Docker 部署

### 前置条件

确保已安装 Docker 和 Docker Compose：

```bash
# 检查 Docker 版本
docker --version

# 检查 Docker Compose 版本
docker compose version
```

### 1. 构建镜像

```bash
cd crates/domain-stun

# 构建 Docker 镜像
docker build -t domain-stun:latest .
```

### 2. 使用 Docker Compose 启动

```bash
# 启动服务（后台运行）
docker compose up -d

# 查看服务状态
docker compose ps

# 查看日志
docker compose logs -f
```

### 3. 验证服务

```bash
# 检查健康状态
curl http://localhost:3479/health

# 获取 STUN 服务器信息
curl http://localhost:3479/api/v1/stun/info
```

预期输出：
```json
{"status":"healthy"}
```

```json
{
  "code":0,
  "message":"success",
  "data":{
    "stun_addr":"你的公网IP:3478",
    "turn_addr":"你的公网IP:3478",
    "public_ip":"你的公网IP"
  }
}
```

### 4. 停止服务

```bash
docker compose down

# 删除数据卷（谨慎操作，会删除数据库）
docker compose down -v
```

---

## Docker Compose 配置说明

```yaml
version: "3.8"

services:
  domain-stun:
    image: domain-stun:latest
    container_name: domain-stun
    restart: unless-stopped
    ports:
      - "3479:3479/tcp"   # HTTP API / Web UI
      - "3478:3478/udp"   # STUN/TURN
    volumes:
      - stun_data:/data              # 数据持久化
      - ./templates:/app/templates:ro  # 模板文件（可选）
    environment:
      - RUST_LOG=info                # 日志级别
```

### 自定义配置示例

```yaml
services:
  domain-stun:
    # ... 其他配置 ...
    environment:
      - RUST_LOG=debug              # 调试模式
    ports:
      - "8080:3479/tcp"            # 自定义 HTTP 端口
      - "3478:3478/udp"            # STUN/TURN 端口保持默认
    volumes:
      - /path/to/data:/data         # 使用宿主机目录
```

---

## 防火墙配置

### Ubuntu (ufw)

```bash
# 开放端口
sudo ufw allow 3478/udp
sudo ufw allow 3479/tcp

# 重新加载
sudo ufw reload
```

### CentOS/RHEL (firewalld)

```bash
# 开放端口
sudo firewall-cmd --permanent --add-port=3478/udp
sudo firewall-cmd --permanent --add-port=3479/tcp

# 重新加载
sudo firewall-cmd --reload
```

### 云平台安全组

需要开放入站规则：
- UDP 3478
- TCP 3479

---

## 验证测试

### 1. 本地健康检查

```bash
curl http://localhost:3479/health
# 预期: {"status":"healthy"}
```

### 2. API 测试

```bash
# 获取服务器信息
curl http://localhost:3479/api/v1/stun/info

# 获取在线 Agent 列表
curl http://localhost:3479/api/v1/agents

# 获取 TURN 分配列表
curl http://localhost:3479/api/v1/turn/allocations
```

### 3. Web UI 访问

在浏览器中访问：
- Dashboard: http://localhost:3479/
- Agents: http://localhost:3479/agents

### 4. STUN 客户端测试

使用 STUN 客户端测试 UDP 端口：

```bash
# 使用 Python 和 pystun3 库测试
pip install pystun3
python3 -c "import pystun3; print(pystun3.get_external_ip())"
```

---

## 日志管理

### 直接部署

日志输出到 stdout，可通过 systemd 查阅：

```bash
sudo journalctl -u domain-stun -f
```

### Docker 部署

```bash
# 查看实时日志
docker compose logs -f

# 查看最近 100 行
docker compose logs --tail=100

# 导出日志到文件
docker compose logs > domain-stun.log
```

---

## 数据备份

### SQLite 数据库

```bash
# 直接部署
cp /opt/domain-stun/data/domain-stun.db /path/to/backup/domain-stun.db.$(date +%Y%m%d)

# Docker 部署
docker compose exec domain-stun sh -c 'cp /data/domain-stun.db /data/backup.db'
docker cp domain-stun:/data/backup.db ./domain-stun.db.$(date +%Y%m%d)
```

---

## 升级更新

### 直接部署

```bash
# 停止服务
sudo systemctl stop domain-stun

# 拉取新代码
git pull

# 重新编译
cargo build --release

# 重启服务
sudo systemctl restart domain-stun
```

### Docker 部署

```bash
# 拉取最新代码
git pull

# 重新构建镜像
docker compose build --no-cache

# 重启服务
docker compose up -d
```

---

## 常见问题

### Q1: 端口被占用

```
Error: Failed to bind to 0.0.0.0:3479 - address already in use
```

解决方案：检查端口占用并释放或使用其他端口

```bash
# 检查端口占用
sudo lsof -i :3479
sudo lsof -i :3478
```

### Q2: Docker 容器无法启动

```bash
# 查看详细日志
docker compose logs domain-stun

# 检查容器状态
docker ps -a | grep domain-stun
```

### Q3: UDP 端口无法访问

确认防火墙放行了 UDP 3478：

```bash
# 在服务器上测试
nc -ul 3478

# 从客户端测试
echo "test" | nc -u <服务器IP> 3478
```

---

## 性能优化建议

### 1. 增加最大分配数

在生产环境中，如需支持更多并发 TURN 用户：

```yaml
# docker-compose.yml
environment:
  - TURN_MAX_ALLOCATIONS=5000
```

### 2. 调整生命周期

```yaml
environment:
  - TURN_DEFAULT_LIFETIME=3600
```

### 3. 使用独立数据卷

对于生产环境，建议使用命名卷：

```yaml
volumes:
  stun_production_data:
    driver: local
```

---

## 相关文档

- [README.md](./README.md) - 项目详细说明
- [API 文档](./README.md#接口设计) - Web API 接口定义
- [domain_agent README](../domain-agent/README.md) - Agent 端使用说明
