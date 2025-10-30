//! Database connection pool for handling concurrent operations
//!
//! This module provides a connection pool that creates separate database connections
//! for concurrent operations to avoid BorrowMutError issues with Turso.

use crate::{
    config::DatabaseConfig,
    error::{DatabaseError, Result},
};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};
use turso::{Builder, Connection};

pub mod pooled_writer;

pub use pooled_writer::PooledDatabaseWriter;

/// Database connection pool for concurrent operations
#[derive(Clone)]
pub struct ConnectionPool {
    /// Database configuration
    config: DatabaseConfig,
    /// Pool of available connections
    connections: Arc<Mutex<Vec<Connection>>>,
    /// Semaphore to limit concurrent connections
    semaphore: Arc<Semaphore>,
    /// Maximum pool size
    max_connections: usize,
    /// Current pool size
    current_size: Arc<Mutex<usize>>,
    /// Flag to track if schema has been initialized
    schema_initialized: Arc<Mutex<bool>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: DatabaseConfig, max_connections: usize) -> Result<Self> {
        info!(
            "[POOL] Creating connection pool with max {} connections",
            max_connections
        );

        let pool = Self {
            config,
            connections: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(max_connections)),
            max_connections,
            current_size: Arc::new(Mutex::new(0)),
            schema_initialized: Arc::new(Mutex::new(false)),
        };

        // Pre-warm pool with at least 1 connection
        pool.create_connection().await?;

        info!("[POOL] Connection pool created successfully");
        Ok(pool)
    }

    /// Create a new database connection
    async fn create_connection(&self) -> Result<()> {
        let mut current_size = self.current_size.lock().await;
        if *current_size >= self.max_connections {
            return Err(DatabaseError::generic(
                "Maximum connection pool size reached",
            ));
        }

        debug!("[POOL] Creating new database connection");

        // Create parent directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&self.config.path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DatabaseError::connection_with_source(
                    format!("Failed to create database directory: {}", parent.display()),
                    e,
                )
            })?;
            debug!(
                "[POOL] Database directory created/verified: {}",
                parent.display()
            );
        }

        let db = Builder::new_local(&self.config.path)
            .build()
            .await
            .map_err(|e| {
                DatabaseError::connection_with_source(
                    format!("Failed to create local database: {}", self.config.path),
                    e,
                )
            })?;

        let conn = db.connect().map_err(|e| {
            DatabaseError::connection_with_source("Failed to establish database connection", e)
        })?;

        // Initialize schema only once for the first connection to prevent locking issues
        // This approach is based on Turso testing insights about concurrency limitations
        let mut schema_init = self.schema_initialized.lock().await;
        if !*schema_init {
            debug!("[POOL] Initializing database schema for first connection");

            let schema_string = include_str!("../../.schema/current_schema.sql")
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty() && !line.starts_with("--"))
                .collect::<Vec<&str>>()
                .join(" ");

            let statements: Vec<&str> = schema_string
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            for statement in statements.iter() {
                if statement.trim().is_empty() {
                    continue;
                }

                conn.execute(statement, ()).await.map_err(|e| {
                    DatabaseError::schema_with_source(
                        format!("Failed to execute schema statement: {statement}"),
                        e,
                    )
                })?;
            }

            *schema_init = true;
            debug!("[POOL] Database schema initialized successfully");
        } else {
            debug!("[POOL] Schema already initialized, skipping for this connection");
        }

        // Add connection to pool
        let mut connections = self.connections.lock().await;
        connections.push(conn);
        *current_size += 1;

        debug!("[POOL] Connection created. Pool size: {}", *current_size);
        Ok(())
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<PooledConnection> {
        loop {
            debug!("[POOL] Acquiring connection from pool");

            // First try to get an existing connection
            {
                let mut connections = self.connections.lock().await;
                if let Some(conn) = connections.pop() {
                    debug!("[POOL] Using existing connection from pool");
                    return Ok(PooledConnection {
                        connection: Some(conn),
                        pool: self.clone(),
                    });
                }
            } // connections mutex guard is dropped here

            // No available connections, check if we can create a new one
            let current_size = *self.current_size.lock().await;

            if current_size < self.max_connections {
                self.create_connection().await?;

                // Try again after creating connection
                let mut connections = self.connections.lock().await;
                if let Some(conn) = connections.pop() {
                    debug!("[POOL] Using newly created connection");
                    return Ok(PooledConnection {
                        connection: Some(conn),
                        pool: self.clone(),
                    });
                } else {
                    error!("[POOL] Failed to get connection after creating new one");
                    return Err(DatabaseError::generic("Failed to create connection"));
                }
            } else {
                warn!("[POOL] Connection pool exhausted, waiting for available connection");

                // Wait and retry
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                continue;
            }
        }
    }

    /// Return a connection to the pool
    async fn return_connection(&self, conn: Connection) {
        debug!("[POOL] Returning connection to pool");
        let mut connections = self.connections.lock().await;
        connections.push(conn);
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let current_size = *self.current_size.lock().await;
        let available_connections = self.connections.lock().await.len();
        let active_connections = current_size - available_connections;
        let available_permits = self.semaphore.available_permits();

        PoolStats {
            max_connections: self.max_connections,
            current_size,
            available_connections,
            active_connections,
            available_permits,
        }
    }

    /// Close all connections in the pool
    pub async fn close(&self) -> Result<()> {
        info!("[POOL] Closing all database connections...");

        // Clear all connections from the pool
        let mut connections = self.connections.lock().await;
        connections.clear();

        // Reset current size
        let mut current_size = self.current_size.lock().await;
        *current_size = 0;

        info!("[POOL] All database connections closed");
        Ok(())
    }
}

/// A pooled database connection that returns itself to the pool when dropped
pub struct PooledConnection {
    connection: Option<Connection>,
    pool: ConnectionPool,
}

impl PooledConnection {
    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        self.connection.as_ref().expect("Connection not available")
    }

    /// Get a mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut Connection {
        self.connection.as_mut().expect("Connection not available")
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.return_connection(conn).await;
            });
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub max_connections: usize,
    pub current_size: usize,
    pub available_connections: usize,
    pub active_connections: usize,
    pub available_permits: usize,
}

impl std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool Stats: {}/{} active ({} available), {} permits",
            self.active_connections,
            self.max_connections,
            self.available_connections,
            self.available_permits
        )
    }
}
