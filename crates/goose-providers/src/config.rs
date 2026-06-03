use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderConfigError {
    #[error("Configuration key not found: {0}")]
    NotFound(String),
    #[error("Failed to deserialize configuration value: {0}")]
    Deserialize(String),
    #[error("Configuration storage error: {0}")]
    Storage(String),
    #[error("Secret storage error: {0}")]
    SecretStorage(String),
}

pub trait ProviderConfigStore: Send + Sync {
    fn get_param_value(&self, key: &str) -> Result<Value, ProviderConfigError>;
    fn get_secret_value(&self, key: &str) -> Result<Value, ProviderConfigError>;

    fn get_secret_group(
        &self,
        primary: &str,
        maybe_secret: &[&str],
    ) -> Result<HashMap<String, String>, ProviderConfigError>;

    fn set_param_value(&self, key: &str, value: Value) -> Result<(), ProviderConfigError>;
    fn set_secret_value(&self, key: &str, value: Value) -> Result<(), ProviderConfigError>;
    fn delete_secret(&self, key: &str) -> Result<(), ProviderConfigError>;
    fn invalidate_secrets_cache(&self);
}

pub trait ProviderConfigExt {
    fn get_param<T: DeserializeOwned>(&self, key: &str) -> Result<T, ProviderConfigError>;
    fn get_secret<T: DeserializeOwned>(&self, key: &str) -> Result<T, ProviderConfigError>;
    fn set_param<T: Serialize>(&self, key: &str, value: T) -> Result<(), ProviderConfigError>;
    fn set_secret<T: Serialize>(&self, key: &str, value: &T) -> Result<(), ProviderConfigError>;
}

impl<T> ProviderConfigExt for T
where
    T: ProviderConfigStore + ?Sized,
{
    fn get_param<U: DeserializeOwned>(&self, key: &str) -> Result<U, ProviderConfigError> {
        serde_json::from_value(self.get_param_value(key)?)
            .map_err(|e| ProviderConfigError::Deserialize(e.to_string()))
    }

    fn get_secret<U: DeserializeOwned>(&self, key: &str) -> Result<U, ProviderConfigError> {
        serde_json::from_value(self.get_secret_value(key)?)
            .map_err(|e| ProviderConfigError::Deserialize(e.to_string()))
    }

    fn set_param<U: Serialize>(&self, key: &str, value: U) -> Result<(), ProviderConfigError> {
        let value = serde_json::to_value(value)
            .map_err(|e| ProviderConfigError::Deserialize(e.to_string()))?;
        self.set_param_value(key, value)
    }

    fn set_secret<U: Serialize>(&self, key: &str, value: &U) -> Result<(), ProviderConfigError> {
        let value = serde_json::to_value(value)
            .map_err(|e| ProviderConfigError::Deserialize(e.to_string()))?;
        self.set_secret_value(key, value)
    }
}

#[derive(Clone)]
pub struct ProviderRuntime {
    pub config: Arc<dyn ProviderConfigStore>,
}
