# Agent Management Service - Architecture

This document describes the internal architecture of the Agent Management Service.

## Overview

The Agent Management Service is a Rust-based microservice that manages agents in a distributed system. It provides three transport layer implementations (gRPC, REST, WebSocket) that all share the same business logic layer.

## Layered Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Transport Layer                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  gRPC       │  │   REST      │  │   WebSocket          │  │
│  │  (tonic)    │  │   (axum)    │  │   (tungstenite)      │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
└─────────┼────────────────┼───────────────────┼───────────────┘
          │                │                    │
          └────────────────┴────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│                      Service Layer                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                    Service                              │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │  │
│  │  │AgentService  │  │LifecycleSvc   │  │HealthSvc   │  │  │
│  │  └──────────────┘  └──────────────┘  └────────────┘  │  │
│  │  ┌──────────────┐                                      │  │
│  │  │DiagnosticSvc │                                      │  │
│  │  └──────────────┘                                      │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Domain Layer                               │  │
│  │  ┌──────────────────────┐                             │  │
│  │  │ LifecycleStateMachine│                             │  │
│  │  └──────────────────────┘                             │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│                      Storage Layer                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Database (SeaORM)                          │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │  │
│  │  │Agent Entity  │  │LifecycleEvt  │  │HealthScore  │  │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  │  │
│  │  ┌──────────────┐                                      │  │
│  │  │SystemInfo    │                                      │  │
│  │  └──────────────┘                                      │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │           PostgreSQL Database                           │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Component Descriptions

### Transport Layer

#### gRPC Server (`server/grpc.rs`)

- Implements `AgentManagementService` from generated protobuf code
- Bridges proto request/response types to service layer
- Supports streaming for events and health scores
- Default port: 50051

#### REST Server (`server/rest.rs`)

- Built with Axum framework
- Provides HTTP/REST interface for agent CRUD operations
- Returns JSON responses
- Default port: 8080

#### WebSocket Server (`server/websocket.rs`)

- Accepts connections from agents
- Handles three message types:
  - `RegisterWithSecret`: Agent registration
  - `SystemInfoReport`: Diagnostic data submission
  - `Heartbeat`: Health metrics submission
- Default port: 8081

### Service Layer

#### AgentService (`service/agent.rs`)

Manages agent CRUD operations:

- `create_agent()` - Register new agent
- `get_agent()` - Retrieve agent by ID
- `list_agents()` - List with optional filters
- `update_agent()` - Update agent fields
- `delete_agent()` - Remove agent
- `approve_agent()` - Approve pending agent
- `deny_agent()` - Deny pending agent

#### LifecycleService (`service/lifecycle.rs`)

Handles lifecycle event recording and retrieval:

- `record_event()` - Store lifecycle event
- `get_events_for_agent()` - Query lifecycle history

#### HealthService (`service/health.rs`)

Calculates and stores health scores:

- `record_health_score()` - Store health metrics
- `get_latest_score()` - Retrieve latest health score
- Pure functions for score calculation:
  - `calculate_latency_score()`
  - `calculate_jitter_score()`
  - `calculate_packet_loss_score()`
  - `calculate_bandwidth_score()`
  - `calculate_health_score()`

Health score weights: Latency=30%, Jitter=20%, Packet Loss=40%, Bandwidth=10%

#### DiagnosticService (`service/diagnostic.rs`)

Manages system information:

- `store_system_info()` - Store SystemInfoReport
- `get_system_info()` - Retrieve latest system info

### Domain Layer

#### LifecycleStateMachine (`domain/state_machine.rs`)

State machine for agent lifecycle management.

**States:**

| State | Description |
|-------|-------------|
| `Created` | Agent created but not started registration |
| `Pending` | Agent in registration process |
| `Authorized` | Agent approved but not yet connected |
| `Connected` | Agent connected but not registered |
| `Registered` | Agent fully operational |
| `Reconnecting` | Agent attempting reconnection |
| `Closed` | Agent terminated (terminal) |

**Transitions:**

```
Created → Pending (AgentRegistering)
Pending → Authorized (AgentApproved)
Pending → Closed (AgentDenied)
Authorized → Connected (AgentConnected)
Connected → Registered (AgentRegistered)
Registered → Reconnecting (AgentDisconnected | AgentReconnecting)
Reconnecting → Registered (AgentRegistered)
Reconnecting → Closed (AgentClosed)
Any → Closed (AgentError | AgentClosed)
```

### Storage Layer

#### Database Wrapper (`storage/mod.rs`)

- Wraps SeaORM `DatabaseConnection`
- Provides `run_migrations()`
- Clone-safe for use across async tasks

#### Entities

| Entity | Description |
|--------|-------------|
| `Agent` | Agent records (id, name, endpoint, status, approval_state, capabilities, etc.) |
| `LifecycleEvent` | Lifecycle event history (agent_id, event_type, payload, timestamp) |
| `HealthScore` | Health metric records (agent_id, overall_score, latency, jitter, packet_loss, bandwidth) |
| `SystemInfo` | System diagnostic snapshots (agent_id, os_info, cpu, memory, disk, network) |

### Configuration (`config.rs`)

Hierarchical configuration with environment variable override:

```
AppConfig
├── ServerConfig (host, port, ws_port)
├── DatabaseConfig (url, username, password, max_connections)
├── GrpcConfig (host, port)
└── RestConfig (host, port)
```

Environment variable format: `AGENT_MANAGEMENT__<SECTION>__<KEY>`

## Data Flow

### Agent Registration Flow

```
1. Agent connects via WebSocket
2. Agent sends RegisterWithSecret message
3. WebSocket server validates credentials
4. Creates agent record via AgentService.create_agent()
5. AgentService creates Agent entity in database
6. Returns agent_id to agent via WebSocket
```

### Health Score Flow

```
1. Agent sends Heartbeat via WebSocket
2. WebSocket server extracts metrics
3. HealthService.record_health_score() called
4. calculate_health_score() computes weighted score
5. HealthScore entity saved to database
```

### Lifecycle Event Flow

```
1. Agent sends lifecycle event via WebSocket
2. WebSocket server receives event
3. LifecycleService.record_event() called
4. LifecycleStateMachine.handle_event() validates transition
5. Event saved to database
6. Agent status updated if transition occurred
```

## Concurrency Model

- `Service::run()` spawns three async tasks:
  - gRPC server task
  - REST server task
  - WebSocket server task
- All services are `Clone` and shared via `Arc`-like pattern
- Database connections are pooled via SeaORM
- Shutdown triggered by Ctrl+C signal

## Code Generation

The protobuf definitions are compiled at build time:

```bash
# build.rs uses tonic-build to generate:
src/generated/agent_management.rs  # Rust types from proto
```

Regeneration is triggered when `proto/agent_management.proto` changes.
