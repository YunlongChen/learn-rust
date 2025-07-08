use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "domains")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub provider_id: Uuid,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::provider::Entity",
        from = "Column::ProviderId",
        to = "super::provider::Column::Id",
        on_delete = "Cascade"
    )]
    Provider,

    #[sea_orm(has_many = "super::dns_record::Entity")]
    DnsRecord,
}

impl Related<super::provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provider.def()
    }
}

impl Related<super::dns_record::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DnsRecord.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
