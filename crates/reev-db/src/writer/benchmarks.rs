//! Benchmark management and synchronization operations
//!
//! Provides atomic benchmark upserts, directory synchronization, and
//! duplicate prevention for consistent benchmark storage.

use crate::{
    error::{DatabaseError, Result},
    shared::benchmark::BenchmarkUtils,
    types::{BenchmarkData, SyncError, SyncResult, SyncedBenchmark},
};
use chrono::Utc;
use std::error::Error;
use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn};

use super::core::DatabaseWriter;

impl DatabaseWriter {
    /// Upsert a benchmark with atomic conflict resolution
    pub async fn upsert_benchmark(
        &self,
        benchmark_name: &str,
        prompt: &str,
        content: &str,
    ) -> Result<String> {
        info!(
            "[DB] Upserting benchmark: '{}' (prompt: {:.50}...)",
            benchmark_name, prompt
        );

        // Generate MD5 using the utility function for consistency
        let prompt_md5 = BenchmarkUtils::generate_md5(benchmark_name, prompt);

        // Use fixed timestamp for consistent content
        let timestamp = Utc::now().to_rfc3339();

        info!(
            "[DB] Upserting benchmark '{}' with MD5 '{}'",
            benchmark_name, prompt_md5
        );

        // Atomic upsert with proper ON CONFLICT handling
        let query = "
            INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                benchmark_name = excluded.benchmark_name,
                prompt = excluded.prompt,
                content = excluded.content,
                updated_at = excluded.updated_at;
        ";

        self.conn
            .execute(
                query,
                [
                    prompt_md5.clone(),
                    benchmark_name.to_string(),
                    prompt.to_string(),
                    content.to_string(),
                    timestamp.clone(),
                    timestamp.clone(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to upsert benchmark with ON CONFLICT", e))?;

        info!(
            "[DB] Upserted benchmark '{}' with MD5 '{}' (prompt: {:.50}...)",
            benchmark_name, prompt_md5, prompt
        );

        Ok(prompt_md5)
    }

    /// Sync all benchmark files from a directory to the database
    pub async fn sync_benchmarks_from_dir<P: AsRef<Path>>(
        &self,
        benchmarks_dir: P,
    ) -> Result<SyncResult> {
        let benchmarks_path = benchmarks_dir.as_ref();
        info!(
            "[DB] Starting benchmark synchronization from: {:?}",
            benchmarks_path
        );

        let start_time = std::time::Instant::now();
        let mut sync_result = SyncResult::default();

        // Check if directory exists
        if !benchmarks_path.exists() {
            let error_msg = format!("Benchmarks directory does not exist: {benchmarks_path:?}");
            error!("[DB] {}", error_msg);
            return Err(DatabaseError::configuration(error_msg));
        }

        // Read directory entries
        let mut entries = fs::read_dir(benchmarks_path).await.map_err(|e| {
            DatabaseError::filesystem_with_source(
                format!("Failed to read benchmarks directory: {benchmarks_path:?}"),
                e,
            )
        })?;

        info!("[DB] Scanning benchmark files...");

        // Process each YAML file
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            DatabaseError::filesystem_with_source("Failed to read directory entry", e)
        })? {
            let path = entry.path();

            // Skip non-files and non-YAML files
            if !path.is_file() {
                continue;
            }

            if let Some(extension) = path.extension() {
                if extension != "yml" && extension != "yaml" {
                    continue;
                }
            } else {
                continue;
            }

            // Process single benchmark file
            match self.sync_single_benchmark(&path).await {
                Ok(synced) => {
                    sync_result.processed_count += 1;
                    match synced.operation.as_str() {
                        "created" => sync_result.new_count += 1,
                        "updated" => sync_result.updated_count += 1,
                        _ => {}
                    }
                    sync_result.synced_benchmarks.push(synced);
                    info!("[DB] ✅ Synced: {:?}", path.file_name());
                }
                Err(e) => {
                    sync_result.error_count += 1;
                    let sync_error = SyncError {
                        file_path: path.to_string_lossy().to_string(),
                        error_message: e.to_string(),
                        error_type: "SyncError".to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    sync_result.errors.push(sync_error);
                    warn!("[DB] ❌ Failed to sync {:?}: {}", path.file_name(), e);
                }
            }
        }

        sync_result.duration_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "[DB] Benchmark synchronization completed: {} processed, {} new, {} updated, {} errors in {}ms",
            sync_result.processed_count,
            sync_result.new_count,
            sync_result.updated_count,
            sync_result.error_count,
            sync_result.duration_ms
        );

        Ok(sync_result)
    }

    /// Sync a single benchmark file
    async fn sync_single_benchmark(&self, file_path: &Path) -> Result<SyncedBenchmark> {
        let start_time = std::time::Instant::now();

        // Read file content
        let content = fs::read_to_string(file_path).await.map_err(|e| {
            DatabaseError::filesystem_with_source(
                format!("Failed to read benchmark file: {file_path:?}"),
                e,
            )
        })?;

        // Parse YAML to extract benchmark name and prompt
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
            DatabaseError::yaml_with_source(format!("Failed to parse YAML from {file_path:?}"), e)
        })?;

        // Extract benchmark name from filename if not in YAML
        let benchmark_name = yaml_value
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
            });

        // Extract prompt
        let prompt = yaml_value
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Check if benchmark already exists
        let existing_md5 = self
            .get_prompt_md5_by_benchmark_name(benchmark_name)
            .await
            .ok()
            .flatten();

        // Upsert benchmark
        let new_md5 = self
            .upsert_benchmark(benchmark_name, prompt, &content)
            .await?;

        // Determine operation
        let operation = if existing_md5.as_ref() == Some(&new_md5) {
            "unchanged"
        } else if existing_md5.is_some() {
            "updated"
        } else {
            "created"
        };

        Ok(SyncedBenchmark {
            name: benchmark_name.to_string(),
            md5: new_md5,
            operation: operation.to_string(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Get benchmark by ID
    pub async fn get_benchmark_by_id(&self, id: &str) -> Result<Option<BenchmarkData>> {
        info!("[DB] Getting benchmark by ID: {}", id);

        let mut rows = self
            .conn
            .query(
                "SELECT id, benchmark_name, prompt, content, created_at, updated_at
                 FROM benchmarks WHERE id = ?",
                [id],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get benchmark by ID", e))?;

        if let Some(row) = rows.next().await? {
            let benchmark = BenchmarkData {
                id: row.get(0)?,
                benchmark_name: row.get(1)?,
                prompt: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            };
            info!("[DB] Found benchmark: {}", benchmark.benchmark_name);
            Ok(Some(benchmark))
        } else {
            info!("[DB] Benchmark not found: {}", id);
            Ok(None)
        }
    }

    /// Get benchmark by name
    pub async fn get_benchmark_by_name(&self, name: &str) -> Result<Option<BenchmarkData>> {
        info!("[DB] Getting benchmark by name: {}", name);

        let mut rows = self
            .conn
            .query(
                "SELECT id, benchmark_name, prompt, content, created_at, updated_at
                 FROM benchmarks WHERE benchmark_name = ? ORDER BY created_at DESC LIMIT 1",
                [name],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get benchmark by name", e))?;

        if let Some(row) = rows.next().await? {
            let benchmark = BenchmarkData {
                id: row.get(0)?,
                benchmark_name: row.get(1)?,
                prompt: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            };
            info!("[DB] Found benchmark: {}", benchmark.benchmark_name);
            Ok(Some(benchmark))
        } else {
            info!("[DB] Benchmark not found: {}", name);
            Ok(None)
        }
    }

    /// Get all benchmarks
    pub async fn get_all_benchmarks(&self) -> Result<Vec<BenchmarkData>> {
        info!("[DB] Getting all benchmarks");

        let mut rows = self
            .conn
            .query(
                "SELECT id, benchmark_name, prompt, content, created_at, updated_at
                 FROM benchmarks ORDER BY benchmark_name",
                (),
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get all benchmarks", e))?;

        let mut benchmarks = Vec::new();
        while let Some(row) = rows.next().await? {
            benchmarks.push(BenchmarkData {
                id: row.get(0)?,
                benchmark_name: row.get(1)?,
                prompt: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            });
        }

        info!("[DB] Retrieved {} benchmarks", benchmarks.len());
        Ok(benchmarks)
    }

    /// Get total benchmark count
    pub async fn get_all_benchmark_count(&self) -> Result<i64> {
        let mut rows = self
            .conn
            .query("SELECT COUNT(*) FROM benchmarks", ())
            .await
            .map_err(|e| DatabaseError::query("Failed to get benchmark count", e))?;

        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0).map_err(|e| {
                DatabaseError::generic_with_source("Failed to parse benchmark count", e)
            })?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Delete benchmark by ID
    pub async fn delete_benchmark(&self, id: &str) -> Result<()> {
        info!("[DB] Deleting benchmark: {}", id);

        let rows_affected = self
            .conn
            .execute("DELETE FROM benchmarks WHERE id = ?", [id])
            .await
            .map_err(|e| DatabaseError::query("Failed to delete benchmark", e))?;

        if rows_affected > 0 {
            info!("[DB] Benchmark deleted successfully: {}", id);
        } else {
            warn!("[DB] Benchmark not found for deletion: {}", id);
        }

        Ok(())
    }

    /// Get prompt MD5 by benchmark name
    pub async fn get_prompt_md5_by_benchmark_name(
        &self,
        benchmark_name: &str,
    ) -> Result<Option<String>> {
        info!(
            "[DB] Looking up prompt MD5 for benchmark_name: '{}'",
            benchmark_name
        );

        let query = "
            SELECT id
            FROM benchmarks
            WHERE benchmark_name = ?
            ORDER BY created_at DESC
            LIMIT 1
        ";
        let mut rows = self
            .conn
            .query(query, [benchmark_name])
            .await
            .map_err(|e| {
                error!("[DB] Query failed: {} - Full error: {:?}", e, e.source());
                DatabaseError::query("Failed to get prompt MD5 by benchmark name", e)
            })?;

        if let Some(row) = rows.next().await? {
            let prompt_md5: Option<String> = row
                .get(0)
                .map_err(|e| DatabaseError::generic_with_source("Failed to parse prompt MD5", e))?;

            Ok(prompt_md5)
        } else {
            Ok(None)
        }
    }
}
