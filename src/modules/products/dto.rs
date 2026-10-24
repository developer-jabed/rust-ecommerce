use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct ProductListItem {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub short_description: Option<String>,
    pub base_price: i64,
    pub sale_price: Option<i64>,
    pub currency: String,
    pub is_featured: bool,
    pub image_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ProductDetail {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub base_price: i64,
    pub sale_price: Option<i64>,
    pub currency: String,
    pub sku: Option<String>,
    pub is_featured: bool,
    pub status: String,
    pub images: Vec<ProductImage>,
    pub variants: Vec<VariantResponse>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ProductImage {
    pub id: Uuid,
    pub url: String,
    pub alt_text: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize)]
pub struct VariantResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: Option<String>,
    pub price: i64,
    pub attributes: serde_json::Value,
    pub available: i32,
}

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub category: Option<String>,
    pub q: Option<String>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub sort: Option<String>,
}
