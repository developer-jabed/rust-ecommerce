use super::dto::*;
use super::repository;
use crate::errors::{validation_errors, AppError};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

pub struct CartService<'a> {
    pool: &'a PgPool,
}

impl<'a> CartService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    async fn resolve_cart(&self, user_id: Option<Uuid>, guest_token: Option<&str>) -> Result<Uuid, AppError> {
        if let Some(uid) = user_id {
            if let Some(id) = repository::find_user_cart(self.pool, uid).await? {
                return Ok(id);
            }
            return Ok(repository::create_user_cart(self.pool, uid).await?);
        }
        let token = guest_token.ok_or_else(|| AppError::bad_request("Guest token required"))?;
        if let Some(id) = repository::find_guest_cart(self.pool, token).await? {
            return Ok(id);
        }
        Ok(repository::create_guest_cart(self.pool, token).await?)
    }

    pub async fn get_cart(&self, user_id: Option<Uuid>, guest_token: Option<&str>) -> Result<CartResponse, AppError> {
        let cart_id = self.resolve_cart(user_id, guest_token).await?;
        let rows = repository::get_items(self.pool, cart_id).await?;
        let mut items = Vec::new();
        let mut subtotal = 0i64;
        let mut item_count = 0i32;
        for (id, variant_id, product_name, variant_name, sku, unit_price, quantity, available) in rows {
            let line_total = unit_price * quantity as i64;
            subtotal += line_total;
            item_count += quantity;
            items.push(CartItemResponse {
                id,
                variant_id,
                product_name,
                variant_name,
                sku,
                unit_price,
                quantity,
                line_total,
                available,
            });
        }
        Ok(CartResponse {
            id: cart_id,
            items,
            subtotal,
            currency: "BDT".into(),
            item_count,
        })
    }

    pub async fn add_item(
        &self,
        user_id: Option<Uuid>,
        guest_token: Option<&str>,
        req: AddItemRequest,
    ) -> Result<CartResponse, AppError> {
        req.validate().map_err(validation_errors)?;
        if !repository::variant_exists_and_active(self.pool, req.variant_id).await? {
            return Err(AppError::bad_request("Variant not available"));
        }
        let cart_id = self.resolve_cart(user_id, guest_token).await?;
        repository::upsert_item(self.pool, cart_id, req.variant_id, req.quantity).await?;
        self.get_cart(user_id, guest_token).await
    }

    pub async fn update_item(
        &self,
        user_id: Option<Uuid>,
        guest_token: Option<&str>,
        item_id: Uuid,
        req: UpdateItemRequest,
    ) -> Result<CartResponse, AppError> {
        req.validate().map_err(validation_errors)?;
        let cart_id = self.resolve_cart(user_id, guest_token).await?;
        if !repository::set_item_qty(self.pool, cart_id, item_id, req.quantity).await? {
            return Err(AppError::not_found("Cart item not found"));
        }
        self.get_cart(user_id, guest_token).await
    }

    pub async fn remove_item(
        &self,
        user_id: Option<Uuid>,
        guest_token: Option<&str>,
        item_id: Uuid,
    ) -> Result<CartResponse, AppError> {
        let cart_id = self.resolve_cart(user_id, guest_token).await?;
        if !repository::delete_item(self.pool, cart_id, item_id).await? {
            return Err(AppError::not_found("Cart item not found"));
        }
        self.get_cart(user_id, guest_token).await
    }

    pub async fn clear(&self, user_id: Option<Uuid>, guest_token: Option<&str>) -> Result<(), AppError> {
        let cart_id = self.resolve_cart(user_id, guest_token).await?;
        repository::clear(self.pool, cart_id).await?;
        Ok(())
    }
}
