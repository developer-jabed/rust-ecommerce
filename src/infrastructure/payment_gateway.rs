//! Payment gateway abstraction.
//! Concrete providers (SSLCommerz, aamarPay, …) will be added in Phase 5.

use async_trait::async_trait;
use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInitRequest {
    pub order_id: Uuid,
    pub amount_minor: i64,
    pub currency: String,
    pub customer_email: String,
    pub customer_name: String,
    pub customer_phone: Option<String>,
    pub success_url: String,
    pub fail_url: String,
    pub cancel_url: String,
    pub ipn_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInitResponse {
    pub provider_ref: String,
    pub redirect_url: Option<String>,
    pub session_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentVerification {
    pub provider_ref: String,
    pub amount_minor: i64,
    pub currency: String,
    pub status: PaymentStatus,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Success,
    Failed,
    Cancelled,
    Expired,
    Refunded,
}

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn initiate(&self, req: PaymentInitRequest) -> Result<PaymentInitResponse, AppError>;
    async fn verify(&self, provider_ref: &str, payload: serde_json::Value) -> Result<PaymentVerification, AppError>;
}

/// Placeholder that always returns an error until a real provider is wired.
pub struct NoopGateway;

#[async_trait]
impl PaymentGateway for NoopGateway {
    async fn initiate(&self, _req: PaymentInitRequest) -> Result<PaymentInitResponse, AppError> {
        Err(AppError::Internal)
    }

    async fn verify(
        &self,
        _provider_ref: &str,
        _payload: serde_json::Value,
    ) -> Result<PaymentVerification, AppError> {
        Err(AppError::Internal)
    }
}