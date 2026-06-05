//! Storage layer for agent management
//!
//! Provides PostgreSQL storage using sea-orm with entities and migrations.

pub mod entities;
mod migrations;

pub use entities::*;
pub use migrations::Migrator;

use sea_orm::Database as SeaDatabase;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;

/// Database wrapper providing connection management for the agent management service.
#[derive(Clone)]
pub struct Database {
    /// The underlying sea-orm database connection.
    conn: DatabaseConnection,
}

impl Database {
    /// Create a new Database instance from a database connection URL.
    ///
    /// # Arguments
    ///
    /// * `url` - PostgreSQL connection URL (e.g., "postgres://user:pass@localhost/db")
    ///
    /// # Returns
    ///
    /// A new Database instance wrapped in a Result.
    pub async fn new(url: &str) -> Result<Self, sea_orm::DbErr> {
        let conn = SeaDatabase::connect(url).await?;
        Ok(Self { conn })
    }

    /// Create a new Database instance from an existing DatabaseConnection.
    ///
    /// # Arguments
    ///
    /// * `conn` - An existing sea-orm DatabaseConnection
    ///
    /// # Returns
    ///
    /// A new Database instance.
    pub fn from_conn(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Get a reference to the underlying database connection.
    ///
    /// # Returns
    ///
    /// A reference to the DatabaseConnection.
    pub fn get_conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Get the underlying database connection, consuming self.
    ///
    /// # Returns
    ///
    /// The DatabaseConnection.
    pub fn into_conn(self) -> DatabaseConnection {
        self.conn
    }

    /// Run all pending migrations.
    ///
    /// This will apply any migrations that have not yet been run.
    pub async fn run_migrations(&self) -> Result<(), sea_orm::DbErr> {
        Migrator::up(&self.conn, None).await
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}
