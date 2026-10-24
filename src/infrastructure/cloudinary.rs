//! Cloudinary upload signature generation.
//! Implemented fully in later phases; skeleton kept for module completeness.

use crate::config::Config;
use crate::errors::AppError;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub struct CloudinaryClient {
    cloud_name: String,
    api_key: String,
    api_secret: String,
}

impl CloudinaryClient {
    pub fn from_config(config: &Config) -> Result<Self, AppError> {
        Ok(Self {
            cloud_name: config
                .cloudinary_cloud_name
                .clone()
                .ok_or_else(|| AppError::Internal)?,
            api_key: config
                .cloudinary_api_key
                .clone()
                .ok_or_else(|| AppError::Internal)?,
            api_secret: config
                .cloudinary_api_secret
                .clone()
                .ok_or_else(|| AppError::Internal)?,
        })
    }

    /// Generate a signed upload parameters map for direct browser upload.
    pub fn sign_upload(&self, params: BTreeMap<String, String>) -> String {
        let mut to_sign = String::new();
        for (k, v) in &params {
            if !to_sign.is_empty() {
                to_sign.push('&');
            }
            to_sign.push_str(k);
            to_sign.push('=');
            to_sign.push_str(v);
        }
        to_sign.push_str(&self.api_secret);

        let mut hasher = Sha256::new();
        hasher.update(to_sign.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn cloud_name(&self) -> &str {
        &self.cloud_name
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}