use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct AddItemRequest {
    pub variant_id: Uuid,
    #[validate(range(min = 1, max = 99))]
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateItemRequest {
    #[validate(range(min = 1, max = 99))]
    pub quantity: i32,
}

#[derive(Debug, Serialize)]
pub struct CartItemResponse {
    pub id: Uuid,
    pub variant_id: Uuid,
    pub product_name: String,
    pub variant_name: Option<String>,
    pub sku: String,
    pub unit_price: i64,
    pub quantity: i32,
    pub line_total: i64,
    pub available: i32,
}

#[derive(Debug, Serialize)]
pub struct CartResponse {
    pub id: Uuid,
    pub items: Vec<CartItemResponse>,
    pub subtotal: i64,
    pub currency: String,
    pub item_count: i32,
}
