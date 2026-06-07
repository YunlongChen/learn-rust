# Agent Management Service

A Rust-based microservice for managing agents in a distributed system. It provides gRPC, REST, and WebSocket APIs for agent registration, lifecycle management, health monitoring, and diagnostics.

## Features

- **Multi-Protocol Support**: gRPC (port 50051), REST (port 8080), and WebSocket (port 8081)
- **Agent Lifecycle Management**: State machine-based lifecycle tracking (Created → Pending → Authorized → Connected → Registered → Reconnecting → Closed)
- **Health Monitoring**: Network health scoring based on latency, jitter, packet loss, and bandwidth
- **System Diagnostics**: Collect and query system information from agents (OS, CPU, memory, disk, network)
- **Approval Workflow**: Pending/Approved/Denied agent registration workflow
- **PostgreSQL Storage**: Persistent storage with SeaORM for agents, lifecycle events, health scores, and system info

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+

### Configuration

Create a `config` file or set environment variables:

```bash
# Environment variables
export AGENT_MANAGEMENT__DATABASE__URL="postgres://user:pass@localhost:5432/agent_management"
export AGENT_MANAGEMENT__DATABASE__USERNAME="agent_management"
export AGENT_MANAGEMENT__DATABASE__PASSWORD="your_password"
export AGENT_MANAGEMENT__GRPC__PORT=50051
export AGENT_MANAGEMENT__REST__PORT=8080
```

Or create a `config` file:

```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 8080,
    "ws_port": 8081
  },
  "database": {
    "url": "postgres://192.168.3.112:5432/agent_management",
    "username": "agent_management",
    "password": "123456",
    "max_connections": 10
  },
  "grpc": {
    "host": "127.0.0.1",
    "port": 50051
  },
  "rest": {
    "host": "127.0.0.1",
    "port": 8080
  }
}
```

### Build and Run

```bash
# Build
cargo build --release

# Run
cargo run --release
```

### Run Tests

```bash
cargo test
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Agent Management Service                      │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  gRPC Server │  │ REST Server │  │   WebSocket Server       │  │
│  │  (port 50051)│  │ (port 8080) │  │   (port 8081)            │  │
│  └──────┬──────┘  └──────┬──────┘  └────────────┬────────────┘  │
│         │                │                       │              │
│         └────────────────┼───────────────────────┘              │
│                          ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                      Service Layer                           ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────┐││
│  │  │AgentService │  │LifecycleSvc │  │HealthService│  │DiagSvc│││
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────┘││
│  └─────────────────────────────────────────────────────────────┘│
│                          │                                       │
│                          ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                     Storage Layer                            ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────┐││
│  │  │   Agents    │  │  Lifecycle  │  │   Health    │  │System │││
│  │  │   Entity   │  │   Events    │  │   Scores    │  │ Info  │││
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────┘││
│  └─────────────────────────────────────────────────────────────┘│
│                          │                                       │
│                          ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              PostgreSQL Database                             ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Agent Lifecycle State Machine

Agents transition through the following states:

```
                    ┌─────────────┐
                    │   Created   │
                    └──────┬──────┘
                           │ AgentRegistering
                           ▼
                    ┌─────────────┐
              ┌─────│   Pending   │─────┐
              │     └─────────────┘     │
    AgentDenied│                         │AgentApproved
              │                         │
              ▼                         ▼
     ┌─────────────┐            ┌─────────────┐
     │   Closed    │            │ Authorized  │
     └─────────────┘            └──────┬──────┘
                                      │AgentConnected
                                      ▼
                               ┌─────────────┐
                               │  Connected  │
                               └──────┬──────┘
                                      │AgentRegistered
                                      ▼
                               ┌─────────────┐
                    ┌─────────│  Registered  │─────────┐
                    │         └─────────────┘         │
          AgentDisconnected                    AgentReconnecting
          AgentError                                │
                    │                               │
                    └─────────────┬─────────────────┘
                                  │AgentClosed/AgentError
                                  ▼
                           ┌─────────────┐
                           │   Closed    │
                           └─────────────┘
```

## Project Structure

```
agent-management/
├── proto/
│   └── agent_management.proto    # Protobuf service definition
├── src/
│   ├── lib.rs                    # Library entry point
│   ├── main.rs                   # Binary entry point
│   ├── config.rs                 # Configuration structs
│   ├── service/
│   │   ├── mod.rs                # Unified Service struct
│   │   ├── agent.rs             # Agent CRUD service
│   │   ├── lifecycle.rs         # Lifecycle event service
│   │   ├── health.rs            # Health scoring service
│   │   └── diagnostic.rs        # Diagnostic service
│   ├── storage/
│   │   ├── mod.rs               # Database wrapper
│   │   ├── entities/            # SeaORM entities
│   │   │   ├── agent.rs
│   │   │   ├── lifecycle_event.rs
│   │   │   ├── health_score.rs
│   │   │   └── system_info.rs
│   │   └── migrations/
│   ├── domain/
│   │   └── state_machine.rs     # Lifecycle state machine
│   └── server/
│       ├── mod.rs               # Server exports
│       ├── grpc.rs              # gRPC server
│       ├── rest.rs              # REST API server
│       └── websocket.rs         # WebSocket server
└── tests/
    ├── agent_service_tests.rs
    └── lifecycle_service_tests.rs
```

## API Reference

### gRPC API (Port 50051)

Full-featured API with all operations. See [API.md](API.md) for details.

### REST API (Port 8080)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/agents` | List all agents |
| GET | `/api/v1/agents/{id}` | Get agent by ID |
| PATCH | `/api/v1/agents/{id}` | Update agent |
| DELETE | `/api/v1/agents/{id}` | Delete agent |
| POST | `/api/v1/agents/{id}/approve` | Approve agent |
| POST | `/api/v1/agents/{id}/deny` | Deny agent |
| GET | `/api/v1/agents/{id}/system-info` | Get system info |
| POST | `/api/v1/agents/{id}/system-info/query` | Query system info |
| GET | `/api/v1/agents/{id}/health` | Get health score |
| GET | `/api/v1/agents/{id}/lifecycle` | Get lifecycle events |

### WebSocket API (Port 8081)

Agents connect via WebSocket and send:

- `RegisterWithSecret` - Agent registration
- `SystemInfoReport` - System diagnostic data
- `Heartbeat` - Health metrics

## Health Scoring

Health scores (0-100) are calculated from network metrics:

| Metric | Weight | Excellent (<) | Good (<) | Fair (<) | Poor (<) | Critical |
|--------|--------|---------------|----------|----------|----------|----------|
| Latency | 30% | 50ms | 100ms | 200ms | 500ms | ≥1000ms |
| Jitter | 20% | 10ms | 30ms | 50ms | 100ms | ≥100ms |
| Packet Loss | 40% | 0.1% | 0.5% | 1% | 3% | ≥5% |
| Bandwidth | 10% | 10Mbps | 5Mbps | 1Mbps | 512Kbps | ≤128Kbps |

## Dependencies

- **Protocol Buffers**: `tonic`, `prost` for gRPC
- **Web Framework**: `axum` for REST API
- **WebSocket**: `tokio-tungstenite`
- **Database**: `sea-orm`, `sea-orm-migration`
- **Async Runtime**: `tokio`
- **Configuration**: `config`
- **Logging**: `tracing`, `tracing-subscriber`
