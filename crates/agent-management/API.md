# Agent Management Service - API Reference

## Table of Contents

- [gRPC API](#grpc-api)
- [REST API](#rest-api)
- [WebSocket API](#websocket-api)
- [Data Types](#data-types)

---

## gRPC API

**Service:** `AgentManagementService`
**Port:** 50051

### RPC Methods

#### RegisterAgent

Register a new agent with the management service.

**Request:**

```protobuf
message RegisterRequest {
  string name = 1;
  string version = 2;
  optional string owner_id = 3;
  optional string description = 4;
}
```

**Response:**

```protobuf
message RegisterResponse {
  Agent agent = 1;
  bool approved = 2;
}
```

#### GetAgent

Retrieve an agent by its ID.

**Request:**

```protobuf
message GetAgentRequest {
  string agent_id = 1;
}
```

**Response:**

```protobuf
message Agent {
  string id = 1;
  string name = 2;
  string version = 3;
  string status = 4;
  string approval_status = 5;
  optional string owner_id = 6;
  optional string description = 7;
  int64 registered_at = 8;
  int64 last_seen_at = 9;
  optional int64 approved_at = 10;
  optional int64 denied_at = 11;
  optional string denial_reason = 12;
}
```

#### ListAgents

List all agents with optional filtering and pagination.

**Request:**

```protobuf
message ListAgentsRequest {
  optional string status_filter = 1;
  optional string approval_status_filter = 2;
  int32 page_size = 3;
  optional string page_token = 4;
}
```

**Response:**

```protobuf
message ListAgentsResponse {
  repeated Agent agents = 1;
  optional string next_page_token = 2;
}
```

#### UpdateAgent

Update an existing agent's fields.

**Request:**

```protobuf
message UpdateAgentRequest {
  string agent_id = 1;
  optional string name = 2;
  optional string version = 3;
  optional string status = 4;
  optional string owner_id = 5;
  optional string description = 6;
}
```

**Response:** `Agent`

#### DeleteAgent

Delete an agent by ID.

**Request:**

```protobuf
message DeleteAgentRequest {
  string agent_id = 1;
}
```

**Response:**

```protobuf
message Empty {}
```

#### ApproveAgent

Approve a pending agent.

**Request:**

```protobuf
message ApproveRequest {
  string agent_id = 1;
  optional string approved_by = 2;
  optional string notes = 3;
}
```

**Response:** `Agent`

#### DenyAgent

Deny a pending agent.

**Request:**

```protobuf
message DenyRequest {
  string agent_id = 1;
  string reason = 2;
  optional string denied_by = 3;
}
```

**Response:** `Agent`

#### GetAgentSystemInfo

Retrieve system information for an agent.

**Request:**

```protobuf
message GetSystemInfoRequest {
  string agent_id = 1;
}
```

**Response:**

```protobuf
message SystemInfo {
  string agent_id = 1;
  OsInfo os_info = 2;
  EnvironmentInfo environment_info = 3;
  ProcessInfo process_info = 4;
  NetworkInfo network_info = 5;
  ResourceInfo resource_info = 6;
  int64 collected_at = 7;
}

message OsInfo {
  string os_name = 1;
  string os_version = 2;
  string kernel_version = 3;
  string hostname = 4;
  string arch = 5;
}

message EnvironmentInfo {
  string user = 1;
  string home_dir = 2;
  string cwd = 3;
  repeated string env_vars = 4;
}

message ProcessInfo {
  int32 pid = 1;
  string parent_pid = 2;
  string name = 3;
  string exe_path = 4;
  int64 start_time = 5;
  int64 uptime_seconds = 6;
}

message NetworkInfo {
  repeated NetworkInterface interfaces = 1;
  repeated NetworkConnection connections = 2;
}

message NetworkInterface {
  string name = 1;
  string mac_address = 2;
  repeated string ipv4_addresses = 3;
  repeated string ipv6_addresses = 4;
  bool is_up = 5;
}

message NetworkConnection {
  string protocol = 1;
  string local_addr = 2;
  string remote_addr = 3;
  string state = 4;
}

message ResourceInfo {
  CpuInfo cpu_info = 1;
  MemoryInfo memory_info = 2;
  repeated DiskInfo disk_info = 3;
}

message CpuInfo {
  string model = 1;
  int32 core_count = 2;
  float usage_percent = 3;
}

message MemoryInfo {
  uint64 total_bytes = 1;
  uint64 used_bytes = 2;
  uint64 available_bytes = 3;
  float usage_percent = 4;
}

message DiskInfo {
  string mount_point = 1;
  string fs_type = 2;
  uint64 total_bytes = 3;
  uint64 used_bytes = 4;
  uint64 available_bytes = 5;
  float usage_percent = 6;
}
```

#### StreamAgentEvents

Stream agent events in real-time.

**Request:**

```protobuf
message StreamEventsRequest {
  string agent_id = 1;
}
```

**Response:** `stream AgentEvent`

#### GetAgentHealth

Get the current health score for an agent.

**Request:**

```protobuf
message GetAgentHealthRequest {
  string agent_id = 1;
}
```

**Response:**

```protobuf
message HealthScore {
  string agent_id = 1;
  int32 score = 2;
  string status = 3;
  repeated string factors = 4;
  int64 calculated_at = 5;
}
```

#### StreamAgentHealth

Stream health scores for an agent at specified intervals.

**Request:**

```protobuf
message StreamHealthRequest {
  string agent_id = 1;
  int32 update_interval_seconds = 2;
}
```

**Response:** `stream HealthScore`

#### GetAgentLifecycleEvents

Get lifecycle events for an agent.

**Request:**

```protobuf
message GetLifecycleRequest {
  string agent_id = 1;
  int32 max_events = 2;
}
```

**Response:**

```protobuf
message LifecycleEventsResponse {
  repeated AgentEvent events = 1;
}

message AgentEvent {
  string event_id = 1;
  string agent_id = 2;
  string event_type = 3;
  string payload = 4;
  int64 timestamp = 5;
}
```

#### StreamLifecycleEvents

Stream lifecycle events for an agent in real-time.

**Request:**

```protobuf
message StreamLifecycleRequest {
  string agent_id = 1;
}
```

**Response:** `stream AgentEvent`

---

## REST API

**Base URL:** `http://localhost:8080/api/v1`

### Endpoints

#### List Agents

```
GET /agents
```

**Response:**

```json
{
  "agents": [
    {
      "id": "uuid",
      "name": "agent-1",
      "agentType": "worker",
      "status": "registered",
      "createdAt": "2024-01-01T00:00:00Z",
      "updatedAt": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1
}
```

#### Get Agent

```
GET /agents/{id}
```

**Response:**

```json
{
  "id": "uuid",
  "name": "agent-1",
  "agentType": "worker",
  "status": "registered",
  "createdAt": "2024-01-01T00:00:00Z",
  "updatedAt": "2024-01-01T00:00:00Z"
}
```

#### Update Agent

```
PATCH /agents/{id}
```

**Request:**

```json
{
  "name": "new-name",
  "status": "offline"
}
```

#### Delete Agent

```
DELETE /agents/{id}
```

**Response:** `204 No Content`

#### Approve Agent

```
POST /agents/{id}/approve
```

#### Deny Agent

```
POST /agents/{id}/deny
```

#### Get Agent System Info

```
GET /agents/{id}/system-info
```

**Response:**

```json
{
  "osInfo": {
    "os": "Linux",
    "osVersion": "5.15.0",
    "kernelVersion": "5.15.0-100-generic",
    "hostname": "agent-1",
    "arch": "x86_64"
  },
  "environmentInfo": {
    "user": "agent",
    "homeDir": "/home/agent",
    "cwd": "/home/agent",
    "envVars": [
      "PATH=/usr/bin"
    ]
  },
  "processInfo": {
    "pid": 1234,
    "parentPid": "1",
    "name": "agent-process",
    "exePath": "/usr/bin/agent",
    "startTime": 1704067200,
    "uptimeSeconds": 3600
  },
  "networkInfo": {
    "interfaces": [
      ...
    ],
    "connections": [
      ...
    ]
  },
  "resourceInfo": {
    "cpuInfo": {
      ...
    },
    "memoryInfo": {
      ...
    },
    "diskInfo": [
      ...
    ]
  }
}
```

#### Query System Info

```
POST /agents/{id}/system-info/query
```

#### Get Agent Health

```
GET /agents/{id}/health
```

#### Get Agent Lifecycle Events

```
GET /agents/{id}/lifecycle
```

---

## WebSocket API

**Port:** 8081

### Connection

Connect via WebSocket to `ws://localhost:8081`

### Message Protocol

All messages are JSON with a `type` discriminator field.

### Messages from Agent

#### RegisterWithSecret

Agent registration message.

```json
{
  "type": "RegisterWithSecret",
  "agent_name": "my-agent",
  "agent_key": "secret-key",
  "capabilities": [
    "task-execution",
    "file-transfer"
  ],
  "version": "1.0.0",
  "hostname": "agent-host-1"
}
```

**Response:**

```json
{
  "success": true,
  "message": "Agent registered",
  "agent_id": "uuid-of-agent"
}
```

#### SystemInfoReport

System diagnostic information report.

```json
{
  "type": "SystemInfoReport",
  "agent_id": "uuid-of-agent",
  "timestamp": "2024-01-01T00:00:00Z",
  "data": {
    "osInfo": {
      ...
    },
    "environmentInfo": {
      ...
    },
    "processInfo": {
      ...
    },
    "networkInfo": {
      ...
    },
    "resourceInfo": {
      ...
    }
  }
}
```

#### Heartbeat

Agent heartbeat with health metrics.

```json
{
  "type": "Heartbeat",
  "status": "healthy",
  "metrics": {
    "latency_ms": 25.5,
    "jitter_ms": 3.2,
    "packet_loss_percent": 0.1,
    "bandwidth_kbps": 50000.0
  },
  "timestamp": 1704067200
}
```

---

## Data Types

### Agent Status

| Status         | Description                                |
|----------------|--------------------------------------------|
| `created`      | Agent created but not started registration |
| `pending`      | Agent in registration process              |
| `authorized`   | Agent approved but not connected           |
| `connected`    | Agent connected but not registered         |
| `registered`   | Agent fully operational                    |
| `reconnecting` | Agent attempting reconnection              |
| `closed`       | Agent terminated                           |

### Approval Status

| Status     | Description       |
|------------|-------------------|
| `pending`  | Awaiting approval |
| `approved` | Agent approved    |
| `denied`   | Agent denied      |

### Health Score

| Score Range | Status    |
|-------------|-----------|
| 90-100      | Excellent |
| 70-89       | Good      |
| 50-69       | Fair      |
| 30-49       | Poor      |
| 0-29        | Critical  |

### Lifecycle Event Types

| Event Type          | Description                |
|---------------------|----------------------------|
| `AgentRegistering`  | Agent started registration |
| `AgentApproved`     | Agent was approved         |
| `AgentDenied`       | Agent was denied           |
| `AgentConnected`    | Agent connected            |
| `AgentRegistered`   | Agent registered           |
| `AgentReconnecting` | Agent reconnecting         |
| `AgentDisconnected` | Agent disconnected         |
| `AgentClosed`       | Agent closed               |
| `AgentError`        | Agent error occurred       |
