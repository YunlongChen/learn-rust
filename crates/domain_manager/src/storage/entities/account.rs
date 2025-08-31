use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub name: String,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
    pub salt: String,
    pub last_login: Option<DateTime>,
    pub provider_type: String,
    pub credential_type: String,
    pub credential_data: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // #[sea_orm(has_many = "super::domain::Entity")]
    // Domain,
}

impl Related<super::domain::Entity> for Entity {
    fn to() -> RelationDef {
        super::dns_record::Relation::Domain.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
