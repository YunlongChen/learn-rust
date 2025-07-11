use crate::storage::migration::{
    m20220101_000001_create_table, m20250712_000001_create_account_table,
};
pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        dbg!("移植数据库！");
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250712_000001_create_account_table::Migration),
        ]
    }
}
