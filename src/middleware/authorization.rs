//! Role-based authorization helpers.
//! Full policy checks arrive with Phase 2.

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;

pub fn require_admin(user: &AuthUser) -> Result<(), AppError> {
    if user.role == "admin" || user.role == "superadmin" {
        Ok(())
    } else {
        Err(AppError::forbidden("Admin access required"))
    }
}

pub fn require_role(user: &AuthUser, allowed: &[&str]) -> Result<(), AppError> {
    if allowed.contains(&user.role.as_str()) {
        Ok(())
    } else {
        Err(AppError::forbidden(format!(
            "Required role: one of {:?}",
            allowed
        )))
    }
}