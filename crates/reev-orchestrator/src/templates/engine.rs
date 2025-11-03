//! Template Engine for Handlebars Compilation and Caching
//!
//! This module provides the core template engine functionality including
//! compilation, caching, and template registration.

use anyhow::Result;
use handlebars::Handlebars;
use lru::LruCache;
use reev_types::flow::WalletContext;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, instrument, trace, warn};

use super::{helpers, TemplateMetadata, TemplateRegistration, TemplateRenderResult};

/// Cache TTL for compiled templates
const TEMPLATE_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Template engine with compilation and caching
#[derive(Debug)]
pub struct TemplateEngine {
    /// Handlebars instance for template compilation
    handlebars: Mutex<Handlebars<'static>>,
    /// Cache for compiled templates
    template_cache: Mutex<LruCache<String, CachedTemplate>>,
    /// Template metadata registry
    metadata_registry: Mutex<std::collections::HashMap<String, TemplateMetadata>>,
    /// Base templates directory
    templates_dir: PathBuf,
}

/// Cached template with TTL
#[derive(Debug, Clone)]
struct CachedTemplate {
    template: String,
    compiled_at: Instant,
}

impl CachedTemplate {
    fn new(template: String) -> Self {
        Self {
            template,
            compiled_at: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.compiled_at.elapsed() > TEMPLATE_CACHE_TTL
    }
}

impl TemplateEngine {
    /// Create new template engine with templates directory
    pub fn new<P: AsRef<Path>>(templates_dir: P) -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register helper functions
        helpers::register_all(&mut handlebars)?;

        // Configure handlebars options
        handlebars.set_strict_mode(true);
        handlebars.register_escape_fn(handlebars::no_escape);

        let templates_dir = templates_dir.as_ref().to_path_buf();

        debug!(
            "Template engine initialized with templates dir: {:?}",
            templates_dir
        );

        Ok(Self {
            handlebars: Mutex::new(handlebars),
            template_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(100).expect("Non-zero cache size"),
            )),
            metadata_registry: Mutex::new(std::collections::HashMap::new()),
            templates_dir,
        })
    }

    /// Register template from file with metadata
    pub async fn register_template_file<P: AsRef<Path>>(
        &self,
        template_path: P,
        metadata: TemplateMetadata,
    ) -> Result<TemplateRegistration> {
        let start_time = std::time::Instant::now();
        let template_path = template_path.as_ref();

        debug!("Registering template: {:?}", template_path);

        // Read template file
        let template_content = tokio::fs::read_to_string(template_path).await?;

        // Register with handlebars
        let template_name = metadata.name.clone();
        {
            let mut handlebars = self.handlebars.lock().await;
            handlebars.register_template_string(&template_name, &template_content)?;
        }

        // Cache template
        {
            let mut cache = self.template_cache.lock().await;
            cache.put(template_name.clone(), CachedTemplate::new(template_content));
        }

        // Store metadata
        {
            let mut registry = self.metadata_registry.lock().await;
            registry.insert(template_name.clone(), metadata.clone());
        }

        let compilation_time = start_time.elapsed().as_millis() as u64;

        debug!(
            "Template registered successfully: {} ({}ms)",
            template_name, compilation_time
        );

        Ok(TemplateRegistration {
            name: template_name,
            source: template_path.to_string_lossy().to_string(),
            metadata,
            compilation_time_ms: compilation_time,
        })
    }

    /// Register all templates from directory structure
    pub async fn register_all_templates(&self) -> Result<Vec<TemplateRegistration>> {
        let mut registrations = Vec::new();

        // Register base templates
        let base_dir = self.templates_dir.join("base");
        println!("DEBUG: Checking base templates dir: {base_dir:?}");
        if base_dir.exists() {
            println!("DEBUG: Base dir exists");
            let mut entries = tokio::fs::read_dir(&base_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                println!("DEBUG: Found base template: {path:?}");

                if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    println!("DEBUG: Registering base template: {name}");

                    let metadata = TemplateMetadata::new(
                        name.clone(),
                        super::TemplateType::Base,
                        format!("Base template: {name}"),
                        vec!["wallet".to_string()],
                        vec!["amount".to_string(), "slippage".to_string()],
                    );

                    let registration = self.register_template_file(&path, metadata).await?;
                    println!("DEBUG: Registered base template: {registration:?}");
                    registrations.push(registration);
                } else {
                    println!("DEBUG: Skipping non-hbs file: {path:?}");
                }
            }
        }

        // Register protocol templates
        let protocols_dir = self.templates_dir.join("protocols");
        debug!("Checking protocols dir: {:?}", protocols_dir);
        if protocols_dir.exists() {
            let mut entries = tokio::fs::read_dir(&protocols_dir).await?;
            while let Some(protocol_entry) = entries.next_entry().await? {
                let protocol_path = protocol_entry.path();
                debug!("Found protocol dir: {:?}", protocol_path);

                if protocol_path.is_dir() {
                    let protocol_name = protocol_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    debug!("Processing protocol: {}", protocol_name);

                    let mut template_entries = tokio::fs::read_dir(&protocol_path).await?;
                    while let Some(template_entry) = template_entries.next_entry().await? {
                        let template_path = template_entry.path();
                        debug!("Found protocol template: {:?}", template_path);

                        if template_path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                            let name = template_path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            let metadata = TemplateMetadata::new(
                                format!("{protocol_name}/{name}"),
                                super::TemplateType::Protocol(protocol_name.to_string()),
                                format!("{protocol_name} protocol template: {name}"),
                                vec!["wallet".to_string()],
                                vec!["amount".to_string(), "apy".to_string()],
                            );

                            let registration = self
                                .register_template_file(&template_path, metadata)
                                .await?;
                            registrations.push(registration);
                        }
                    }
                }
            }
        }

        // Register scenario templates
        let scenarios_dir = self.templates_dir.join("scenarios");
        if scenarios_dir.exists() {
            let mut entries = tokio::fs::read_dir(&scenarios_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let metadata = TemplateMetadata::new(
                        format!("scenarios/{name}"),
                        super::TemplateType::Scenario(name.clone()),
                        format!("Scenario template: {name}"),
                        vec!["wallet".to_string(), "amount".to_string()],
                        vec!["slippage".to_string(), "apy".to_string()],
                    );

                    let registration = self.register_template_file(&path, metadata).await?;
                    registrations.push(registration);
                }
            }
        }

        debug!("Registered {} templates total", registrations.len());
        Ok(registrations)
    }

    /// Render template with context and variables
    #[instrument(skip(self, context, variables))]
    pub async fn render_template(
        &self,
        template_name: &str,
        context: &WalletContext,
        variables: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<TemplateRenderResult> {
        let start_time = std::time::Instant::now();

        trace!("Rendering template: {}", template_name);

        // Get template metadata for validation
        let metadata = {
            let registry = self.metadata_registry.lock().await;
            registry.get(template_name).cloned()
        };

        if let Some(ref metadata) = metadata {
            metadata.validate_variables(context, variables)?;
        } else {
            warn!("Template metadata not found for: {}", template_name);
        }

        // Prepare render data
        let mut render_data = serde_json::to_value(context)?;
        if let serde_json::Value::Object(ref mut map) = render_data {
            for (key, value) in variables {
                map.insert(key.clone(), value.clone());
            }
        }

        // Render template
        let rendered = {
            let handlebars = self.handlebars.lock().await;
            handlebars.render(template_name, &render_data)?
        };

        let render_time = start_time.elapsed().as_millis() as u64;
        let variables_used = variables.keys().cloned().collect();

        trace!(
            "Template rendered successfully: {} ({}ms)",
            template_name,
            render_time
        );

        Ok(TemplateRenderResult {
            rendered,
            template_name: template_name.to_string(),
            render_time_ms: render_time,
            variables_used,
        })
    }

    /// Get template metadata
    pub async fn get_template_metadata(&self, template_name: &str) -> Option<TemplateMetadata> {
        let registry = self.metadata_registry.lock().await;
        registry.get(template_name).cloned()
    }

    /// List all registered templates
    pub async fn list_templates(&self) -> Vec<String> {
        let registry = self.metadata_registry.lock().await;
        registry.keys().cloned().collect()
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.template_cache.lock().await;
        let registry = self.metadata_registry.lock().await;
        (cache.len(), registry.len())
    }

    /// Clear template cache
    pub async fn clear_cache(&self) {
        let mut cache = self.template_cache.lock().await;
        cache.clear();
        debug!("Template cache cleared");
    }

    /// Reload template from file
    #[instrument(skip(self))]
    pub async fn reload_template(&self, template_name: &str) -> Result<bool> {
        debug!("Reloading template: {}", template_name);

        let metadata = {
            let registry = self.metadata_registry.lock().await;
            registry.get(template_name).cloned()
        };

        if let Some(metadata) = metadata {
            // Find and re-register template file
            let template_path = PathBuf::from(&metadata.name.replace('/', "/"));
            let full_path = self.templates_dir.join(template_path);

            if full_path.exists() {
                drop(full_path);
                debug!(
                    "Template reload not implemented with current architecture: {}",
                    template_name
                );
                return Ok(false);
            }
        }

        warn!("Template not found for reload: {}", template_name);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::TemplateType;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_template_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TemplateEngine::new(temp_dir.path());
        assert!(engine.is_ok());
    }

    #[tokio::test]

    async fn test_template_registration() {
        let temp_dir = TempDir::new().unwrap();

        // Create a simple template file
        let template_path = temp_dir.path().join("test.hbs");
        tokio::fs::write(&template_path, "Hello {{name}}!")
            .await
            .unwrap();

        let metadata = TemplateMetadata::new(
            "test".to_string(),
            TemplateType::Base,
            "Test template".to_string(),
            vec!["name".to_string()],
            vec![],
        );

        let engine = TemplateEngine::new(temp_dir.path()).unwrap();
        let registration = engine
            .register_template_file(&template_path, metadata)
            .await
            .unwrap();

        assert_eq!(registration.name, "test");
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TemplateEngine::new(temp_dir.path()).unwrap();

        // Create and register a simple template
        let template_path = temp_dir.path().join("test.hbs");
        tokio::fs::write(&template_path, "Amount: {{amount}}")
            .await
            .unwrap();

        let metadata = TemplateMetadata::new(
            "test".to_string(),
            TemplateType::Base,
            "Test template".to_string(),
            vec!["amount".to_string()],
            vec![],
        );

        let engine = TemplateEngine::new(temp_dir.path()).unwrap();
        engine
            .register_template_file(&template_path, metadata)
            .await
            .unwrap();

        // Render template
        let mut variables = std::collections::HashMap::new();
        variables.insert("amount".to_string(), json!(100));

        let context = WalletContext::new("test".to_string());
        let result = engine
            .render_template("test", &context, &variables)
            .await
            .unwrap();

        assert_eq!(result.rendered, "Amount: 100");
        assert_eq!(result.template_name, "test");
    }
}
