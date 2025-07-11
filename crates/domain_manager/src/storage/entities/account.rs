use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub name: String,
    pub api_key: String,
    pub api_secret: String,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub extra_config: Option<serde_json::Value>,
    pub created_at: DateTime,
    pub updated_at: DateTime,

    pub username: String,
    pub email: String,
    pub salt: String,
    pub last_login: Option<DateTime>,
    pub credential_type: String,
    pub credential_data: String,
    pub provider_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // #[sea_orm(has_many = "super::domain::Entity")]
    // Domain,
}

impl Related<super::domain::Entity> for Entity {
    fn to() -> RelationDef {
        crate::storage::dns_record::Relation::Domain.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
