//! Transactional email abstraction (order confirmation, password reset, etc.).
//! Concrete providers will be added later; this keeps the module boundary clean.

use crate::errors::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, to: &str, subject: &str, html_body: &str) -> Result<(), AppError>;
}

pub struct NoopEmailSender;

#[async_trait]
impl EmailSender for NoopEmailSender {
    async fn send(&self, _to: &str, _subject: &str, _html_body: &str) -> Result<(), AppError> {
        tracing::debug!("NoopEmailSender: email suppressed");
        Ok(())
    }
}