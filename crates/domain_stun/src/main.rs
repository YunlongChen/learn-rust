//! Domain STUN Server
//!
//! A STUN/TURN server for NAT traversal and peer-to-peer connections.

mod config;
mod handlers;
mod stun;
mod turn;
mod db;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use actix_web::{web, App, HttpServer, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::Config;
use crate::stun::{StunMessage, StunMessageType, handle_binding_request, make_error_response};
use crate::turn::TurnHandler;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub agents: Arc<RwLock<HashMap<Uuid, AgentInfo>>>,
    pub turn_handler: Arc<TurnHandler>,
    pub shutdown_tx: broadcast::Sender<()>,
}

/// Agent registration info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: Uuid,
    pub name: String,
    pub public_addr: Option<SocketAddr>,
    pub nat_type: String,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// API Response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            code: -1,
            message: message.into(),
            data: None,
        }
    }
}

/// GET /api/v1/stun/info
async fn get_stun_info(state: web::Data<AppState>) -> HttpResponse {
    let public_ip = get_public_ip();
    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "stun_addr": format!("{}:{}", public_ip, state.config.stun.bind_port),
        "turn_addr": format!("{}:{}", public_ip, state.config.turn.bind_port),
        "public_ip": public_ip
    })))
}

/// POST /api/v1/agent/register
async fn register_agent(
    state: web::Data<AppState>,
    body: web::Json<RegisterAgentRequest>,
) -> HttpResponse {
    let agent = AgentInfo {
        id: body.agent_id,
        name: body.name.clone(),
        public_addr: body.public_addr.as_ref().and_then(|s| s.parse().ok()),
        nat_type: body.nat_type.clone(),
        connected_at: Utc::now(),
        last_seen: Utc::now(),
    };

    let mut agents = state.agents.write().await;
    agents.insert(body.agent_id, agent);

    info!("Agent registered: {} ({})", body.name, body.agent_id);

    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "relay_token": Uuid::new_v4().to_string()
    })))
}

/// GET /api/v1/agents
async fn list_agents(state: web::Data<AppState>) -> HttpResponse {
    let agents = state.agents.read().await;
    let agent_list: Vec<_> = agents.values().map(|a| {
        serde_json::json!({
            "id": a.id,
            "name": a.name,
            "public_addr": a.public_addr.map(|addr| addr.to_string()).unwrap_or_default(),
            "nat_type": a.nat_type,
            "connected_at": a.connected_at.to_rfc3339(),
            "last_seen": a.last_seen.to_rfc3339()
        })
    }).collect();

    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "agents": agent_list
    })))
}

#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    pub agent_id: Uuid,
    pub name: String,
    pub public_addr: Option<String>,
    pub nat_type: String,
}

/// GET /api/v1/turn/allocations
async fn list_allocations(state: web::Data<AppState>) -> HttpResponse {
    let allocations = state.turn_handler.get_allocations().await;
    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "allocations": allocations
    })))
}

/// GET /dashboard
async fn dashboard(state: web::Data<AppState>, tera: web::Data<tera::Tera>) -> HttpResponse {
    let agents = state.agents.read().await;
    let allocations = state.turn_handler.get_allocations().await;
    let public_ip = get_public_ip();

    #[derive(serde::Serialize)]
    struct DashboardCtx {
        page: String,
        online_agents: usize,
        active_allocations: usize,
        public_ip: String,
        stun_port: u16,
        turn_port: u16,
        agents: Vec<AgentJson>,
    }

    #[derive(serde::Serialize)]
    struct AgentJson {
        id: String,
        name: String,
        nat_type: String,
        last_seen: String,
    }

    let agents_list: Vec<AgentJson> = agents.values().map(|a| AgentJson {
        id: a.id.to_string(),
        name: a.name.clone(),
        nat_type: a.nat_type.clone(),
        last_seen: a.last_seen.format("%H:%M:%S").to_string(),
    }).collect();

    let ctx = DashboardCtx {
        page: "dashboard".to_string(),
        online_agents: agents.len(),
        active_allocations: allocations.len(),
        public_ip,
        stun_port: state.config.stun.bind_port,
        turn_port: state.config.turn.bind_port,
        agents: agents_list,
    };

    let rendered = tera.render("dashboard.html", &tera::Context::from_serialize(&ctx).unwrap_or_default())
        .map_err(|e| error!("Template error: {}", e))
        .unwrap_or_default();

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}

/// GET /agents
async fn agents_page(state: web::Data<AppState>, tera: web::Data<tera::Tera>) -> HttpResponse {
    let agents = state.agents.read().await;

    #[derive(serde::Serialize)]
    struct AgentsCtx {
        page: String,
        agents: Vec<AgentDetailJson>,
    }

    #[derive(serde::Serialize)]
    struct AgentDetailJson {
        id: String,
        name: String,
        nat_type: String,
        public_addr: String,
        connected_at: String,
        last_seen: String,
    }

    let agents_list: Vec<AgentDetailJson> = agents.values().map(|a| AgentDetailJson {
        id: a.id.to_string(),
        name: a.name.clone(),
        nat_type: a.nat_type.clone(),
        public_addr: a.public_addr.map(|addr| addr.to_string()).unwrap_or_else(|| "N/A".to_string()),
        connected_at: a.connected_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        last_seen: a.last_seen.format("%Y-%m-%d %H:%M:%S").to_string(),
    }).collect();

    let ctx = AgentsCtx {
        page: "agents".to_string(),
        agents: agents_list,
    };

    let rendered = tera.render("agents.html", &tera::Context::from_serialize(&ctx).unwrap_or_default())
        .unwrap_or_default();

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered)
}

/// Health check
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}

fn get_public_ip() -> String {
    "127.0.0.1".to_string()
}

/// Start STUN/TURN UDP server
async fn start_stun_server(state: Arc<AppState>) {
    let addr = format!("0.0.0.0:{}", state.config.stun.bind_port);
    info!("Starting STUN server on {}", addr);

    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to bind STUN socket: {}", e);
            return;
        }
    };

    let mut buf = [0u8; 1024];
    let turn_handler = state.turn_handler.clone();
    let agents = state.agents.clone();

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, from)) => {
                let data = &buf[..len];
                debug!("Received {} bytes from {}", len, from);

                match StunMessage::parse(data) {
                    Ok(msg) => {
                        debug!("STUN message type: {:?}", msg.message_type);

                        let response: Option<Vec<u8>> = match msg.message_type {
                            StunMessageType::BindingRequest => {
                                handle_binding_request(&msg, &from, &agents).await
                            }
                            StunMessageType::AllocateRequest => {
                                turn_handler.handle_allocate_request(&msg, &from).await
                            }
                            StunMessageType::RefreshRequest => {
                                turn_handler.handle_refresh_request(&msg, &from).await
                            }
                            StunMessageType::ChannelBindRequest => {
                                turn_handler.handle_channel_bind(&msg, &from).await
                            }
                            _ => {
                                Some(make_error_response(&msg, 400, "Not Implemented"))
                            }
                        };

                        if let Some(resp) = response {
                            if let Err(e) = socket.send_to(&resp, &from).await {
                                warn!("Failed to send response: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse STUN message: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("UDP recv error: {}", e);
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting Domain STUN Server");

    let config = Config::load();
    info!("Configuration loaded");

    let (shutdown_tx, _shutdown_rx) = broadcast::channel(1);
    let state = Arc::new(AppState {
        config: config.clone(),
        agents: Arc::new(RwLock::new(HashMap::new())),
        turn_handler: Arc::new(TurnHandler::new()),
        shutdown_tx,
    });

    let stun_state = state.clone();
    tokio::spawn(async move {
        start_stun_server(stun_state).await;
    });

    let tera = tera::Tera::new("templates/**/*.html").unwrap_or_else(|e| {
        info!("Template init error: {}, using defaults", e);
        tera::Tera::default()
    });

    let bind_addr = format!("0.0.0.0:{}", config.server.port);
    info!("Starting HTTP server on http://{}", bind_addr);

    let app_state = (*state).clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(tera.clone()))
            .route("/api/v1/stun/info", web::get().to(get_stun_info))
            .route("/api/v1/agent/register", web::post().to(register_agent))
            .route("/api/v1/agents", web::get().to(list_agents))
            .route("/api/v1/turn/allocations", web::get().to(list_allocations))
            .route("/", web::get().to(dashboard))
            .route("/agents", web::get().to(agents_page))
            .route("/health", web::get().to(health))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
