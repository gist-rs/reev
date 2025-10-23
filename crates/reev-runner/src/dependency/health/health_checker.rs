//! Health checker implementation for dependency services

use super::{HealthCheckConfig, HealthCheckResult, HealthError, ServiceHealth};
use anyhow::Result;
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Health checker for individual services
#[derive(Clone)]
pub struct HealthChecker {
    client: Client,
    config: HealthCheckConfig,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthCheckConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent("reev-health-checker/1.0")
            .build()
            .expect("Failed to create HTTP client for health checking");

        Self { client, config }
    }

    /// Check health of reev-agent service
    pub async fn check_reev_agent(&self, base_url: &str) -> HealthCheckResult {
        let service_name = "reev-agent".to_string();
        let url = format!("{}/health", base_url.trim_end_matches('/'));

        self.check_http_endpoint(&service_name, &url).await
    }

    /// Check health of surfpool service
    pub async fn check_surfpool(&self, rpc_url: &str) -> HealthCheckResult {
        let service_name = "surfpool".to_string();

        // For surfpool, we check if the RPC endpoint is responsive
        self.check_rpc_endpoint(&service_name, rpc_url).await
    }

    /// Generic HTTP endpoint health check
    pub async fn check_http_endpoint(&self, service_name: &str, url: &str) -> HealthCheckResult {
        let start_time = Instant::now();

        debug!(service_name, url, "Performing HTTP health check");

        let result = timeout(self.config.timeout, async {
            let response = self.client.get(url).send().await?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "HTTP {} - {}",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Unknown")
                ))
            }
        })
        .await;

        let response_time = start_time.elapsed();

        match result {
            Ok(Ok(())) => {
                info!(
                    service_name,
                    url,
                    response_time_ms = response_time.as_millis(),
                    "Health check passed"
                );

                HealthCheckResult::new(service_name.to_string())
                    .with_status(ServiceHealth::Healthy)
                    .with_response_time(response_time)
            }
            Ok(Err(e)) => {
                let error_msg = format!("HTTP request failed: {e}");
                debug!(service_name, url, error = %e, "Health check failed");

                HealthCheckResult::new(service_name.to_string())
                    .with_status(ServiceHealth::Unhealthy(error_msg.clone()))
                    .with_response_time(response_time)
                    .with_details(error_msg)
            }
            Err(_) => {
                let error_msg = format!(
                    "Health check timeout after {}ms",
                    self.config.timeout.as_millis()
                );
                debug!(
                    service_name,
                    url,
                    timeout_ms = self.config.timeout.as_millis(),
                    "Health check timed out"
                );

                HealthCheckResult::new(service_name.to_string())
                    .with_status(ServiceHealth::Unhealthy(error_msg.clone()))
                    .with_response_time(response_time)
                    .with_details(error_msg)
            }
        }
    }

    /// RPC endpoint health check (for surfpool)
    async fn check_rpc_endpoint(&self, service_name: &str, rpc_url: &str) -> HealthCheckResult {
        let start_time = Instant::now();

        debug!(service_name, rpc_url, "Performing RPC health check");

        // Try a simple RPC call to check if the service is responsive
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getHealth"
        });

        let result = timeout(self.config.timeout, async {
            let response = self
                .client
                .post(rpc_url)
                .header("Content-Type", "application/json")
                .json(&rpc_request)
                .send()
                .await?;

            if response.status().is_success() {
                let body = response.bytes().await?;
                let json: serde_json::Value = serde_json::from_slice(&body)?;

                // Check if the RPC response is valid
                if json.get("result").is_some() {
                    Ok(())
                } else if let Some(error) = json.get("error") {
                    Err(anyhow::anyhow!("RPC error: {error}"))
                } else {
                    Err(anyhow::anyhow!("Invalid RPC response format"))
                }
            } else {
                Err(anyhow::anyhow!(
                    "HTTP {} - {}",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Unknown")
                ))
            }
        })
        .await;

        let response_time = start_time.elapsed();

        match result {
            Ok(Ok(())) => {
                info!(
                    service_name,
                    rpc_url,
                    response_time_ms = response_time.as_millis(),
                    "RPC health check passed"
                );

                HealthCheckResult::new(service_name.to_string())
                    .with_status(ServiceHealth::Healthy)
                    .with_response_time(response_time)
            }
            Ok(Err(e)) => {
                let error_msg = format!("RPC request failed: {e}");
                debug!(service_name, rpc_url, error = %e, "RPC health check failed");

                HealthCheckResult::new(service_name.to_string())
                    .with_status(ServiceHealth::Unhealthy(error_msg.clone()))
                    .with_response_time(response_time)
                    .with_details(error_msg)
            }
            Err(_) => {
                let error_msg = format!(
                    "RPC health check timeout after {}ms",
                    self.config.timeout.as_millis()
                );
                warn!(
                    service_name,
                    rpc_url,
                    timeout_ms = self.config.timeout.as_millis(),
                    "RPC health check timed out"
                );

                HealthCheckResult::new(service_name.to_string())
                    .with_status(ServiceHealth::Unhealthy(error_msg.clone()))
                    .with_response_time(response_time)
                    .with_details(error_msg)
            }
        }
    }

    /// Perform a single health check with retry logic
    pub async fn check_with_retry(
        &self,
        service_name: &str,
        check_fn: impl Fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = HealthCheckResult> + Send>,
        >,
    ) -> HealthCheckResult {
        let mut last_result = None;

        for attempt in 1..=self.config.failure_threshold {
            debug!(service_name, attempt, "Performing health check attempt");

            let result = check_fn().await;

            if result.is_healthy() {
                if attempt > 1 {
                    info!(service_name, attempt, "Health check passed after retries");
                }
                return result;
            }

            last_result = Some(result);

            if attempt < self.config.failure_threshold {
                debug!(service_name, attempt, "Health check failed, retrying...");
                tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
            }
        }

        warn!(
            service_name,
            attempts = self.config.failure_threshold,
            "Health check failed after all retries"
        );

        last_result.unwrap_or_else(|| {
            HealthCheckResult::new(service_name.to_string()).with_status(ServiceHealth::Unhealthy(
                "All health check attempts failed".to_string(),
            ))
        })
    }

    /// Wait for a service to become healthy
    pub async fn wait_for_health<F, Fut>(
        &self,
        service_name: &str,
        mut check_fn: F,
        timeout_duration: Duration,
    ) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = HealthCheckResult> + Send,
    {
        info!(
            service_name,
            timeout_secs = timeout_duration.as_secs(),
            "Waiting for service to become healthy"
        );

        let start_time = Instant::now();

        while start_time.elapsed() < timeout_duration {
            let result = check_fn().await;

            if result.is_healthy() {
                info!(
                    service_name,
                    elapsed_ms = start_time.elapsed().as_millis(),
                    "Service became healthy"
                );
                return Ok(());
            }

            if self.config.verbose_logging {
                debug!(
                    service_name,
                    status = ?result.status,
                    elapsed_ms = start_time.elapsed().as_millis(),
                    "Service not yet healthy, waiting..."
                );
            }

            tokio::time::sleep(self.config.check_interval).await;
        }

        Err(HealthError::Timeout {
            service: service_name.to_string(),
            timeout_ms: timeout_duration.as_millis() as u64,
        }
        .into())
    }

    /// Get the current configuration
    pub fn config(&self) -> &HealthCheckConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: HealthCheckConfig) {
        self.config = config;

        // Recreate client with new timeout
        self.client = Client::builder()
            .timeout(self.config.timeout)
            .user_agent("reev-health-checker/1.0")
            .build()
            .expect("Failed to recreate HTTP client for health checking");
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(HealthCheckConfig::default())
    }
}
