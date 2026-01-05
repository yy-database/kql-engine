use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;
use kql_types::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KqlConfig {
    pub project: ProjectConfig,
    pub database: DatabaseConfig,
    pub codegen: CodegenConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_dialect")]
    pub dialect: String,
}

fn default_dialect() -> String {
    "postgres".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodegenConfig {
    #[serde(default = "default_out_dir")]
    pub out_dir: PathBuf,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_out_dir() -> PathBuf {
    PathBuf::from("src/models")
}

fn default_language() -> String {
    "rust".to_string()
}

impl KqlConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: KqlConfig = toml::from_str(&content).map_err(|e| {
            kql_types::KqlError::new(kql_types::KqlErrorKind::IoError {
                message: format!("Failed to parse kql.toml: {}", e),
            })
        })?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            kql_types::KqlError::new(kql_types::KqlErrorKind::IoError {
                message: format!("Failed to serialize config: {}", e),
            })
        })?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Update an existing toml file while preserving comments/formatting
    pub fn update_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = if path.as_ref().exists() {
            std::fs::read_to_string(&path)?
        } else {
            String::new()
        };

        let mut doc = content.parse::<DocumentMut>().map_err(|e| {
            kql_types::KqlError::new(kql_types::KqlErrorKind::IoError {
                message: format!("Failed to parse existing kql.toml: {}", e),
            })
        })?;

        // Update values using toml_edit to preserve formatting
        // This is a simplified version, in a real scenario we'd map fields more carefully
        let new_toml = toml::to_string(self).map_err(|e| {
            kql_types::KqlError::new(kql_types::KqlErrorKind::IoError {
                message: format!("Failed to serialize config: {}", e),
            })
        })?;
        
        let new_doc = new_toml.parse::<DocumentMut>().unwrap();
        
        for (key, item) in new_doc.iter() {
            doc.insert(key, item.clone());
        }

        std::fs::write(path, doc.to_string())?;
        Ok(())
    }
}
