//! Request-ID utilities (tower-http already injects X-Request-Id).
//! This module exists for future custom extensions.

use uuid::Uuid;

pub fn generate() -> String {
    Uuid::new_v4().to_string()
}