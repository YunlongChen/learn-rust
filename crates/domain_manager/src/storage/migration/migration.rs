use crate::storage::migration::{
    m20250712_000001_create_account_table, m20250712_000001_create_dns_record_table,
    m20250712_000001_create_domain_table, m20250712_000001_create_provider_table,
};
pub use sea_orm_migration::prelude::*;
use tracing::info;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        info!("移植数据库！");
        vec![
            Box::new(m20250712_000001_create_provider_table::Migration),
            Box::new(m20250712_000001_create_dns_record_table::Migration),
            Box::new(m20250712_000001_create_account_table::Migration),
            Box::new(m20250712_000001_create_domain_table::Migration),
        ]
    }
}
