use sea_orm::prelude::{DateTime, Json};
use sea_orm::{ActiveModelBehavior, DeriveEntityModel, DerivePrimaryKey, DeriveRelation, EnumIter, PrimaryKeyTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent ActiveModel
#[derive(Clone, Debug, PartialEq, Eq, Hash, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "name")]
    pub name: String,
    #[sea_orm(nullable)]
    pub description: Option<String>,
    #[sea_orm(column_name = "endpoint")]
    pub endpoint: String,
    #[sea_orm(nullable, column_name = "auth_key")]
    pub auth_key: Option<String>,
    #[sea_orm(nullable, column_type = "Json")]
    pub capabilities: Option<Json>,
    #[sea_orm(column_name = "status")]
    pub status: String,
    #[sea_orm(nullable, column_type = "Json")]
    pub tags: Option<Json>,
    #[sea_orm(nullable, column_type = "Json")]
    pub system_info: Option<Json>,
    #[sea_orm(nullable, column_type = "Json")]
    pub connection_info: Option<Json>,
    #[sea_orm(nullable)]
    pub last_heartbeat: Option<DateTime>,
    #[sea_orm(nullable, column_name = "enabled")]
    pub enabled: bool,
    #[sea_orm(column_name = "approval_state")]
    pub approval_state: String,
    #[sea_orm(nullable, column_name = "agent_key_hash")]
    pub agent_key_hash: Option<String>,
    #[sea_orm(nullable)]
    pub approved_at: Option<DateTime>,
    #[sea_orm(nullable, column_name = "approved_by")]
    pub approved_by: Option<String>,
    #[sea_orm(nullable)]
    pub created_at: Option<DateTime>,
    #[sea_orm(nullable)]
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
