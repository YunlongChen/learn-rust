//! Database models for STUN server

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub public_addr: Option<String>,
    pub nat_type: String,
    pub connected_at: DateTime,
    pub last_seen: DateTime,
}

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "allocations")]
pub struct AllocationModel {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub client_addr: String,
    pub relayed_addr: String,
    pub lifetime: u32,
    pub created_at: DateTime,
}

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "connections")]
pub struct ConnectionModel {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub agent_id: Option<Uuid>,
    pub peer_addr: String,
    pub connected_at: DateTime,
    pub bytes_transferred: u64,
}

impl ActiveModelBehavior for ActiveModel {}
