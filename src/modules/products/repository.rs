use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, sqlx::FromRow)]
pub struct ProductRow {
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
    pub created_at: DateTime<Utc>,
}

/// Resolve category filter: accept UUID or slug. Returns category id if found.
pub async fn resolve_category_id(pool: &PgPool, category: &str) -> Result<Option<Uuid>, sqlx::Error> {
    if let Ok(id) = Uuid::parse_str(category) {
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM categories WHERE id = $1 AND is_active = TRUE",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        return Ok(exists.map(|r| r.0));
    }
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM categories WHERE slug = $1 AND is_active = TRUE",
    )
    .bind(category)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

pub async fn list_published(
    pool: &PgPool,
    limit: i64,
    offset: i64,
    q: Option<&str>,
    min_price: Option<i64>,
    max_price: Option<i64>,
    category_id: Option<Uuid>,
    sort: &str,
) -> Result<(Vec<ProductRow>, i64), sqlx::Error> {
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT p.id)
           FROM products p
           LEFT JOIN product_categories pc ON pc.product_id = p.id
           WHERE p.status = 'published'
             AND ($1::text IS NULL OR p.name ILIKE '%' || $1 || '%' OR p.description ILIKE '%' || $1 || '%' OR p.sku ILIKE '%' || $1 || '%')
             AND ($2::bigint IS NULL OR COALESCE(p.sale_price, p.base_price) >= $2)
             AND ($3::bigint IS NULL OR COALESCE(p.sale_price, p.base_price) <= $3)
             AND ($4::uuid IS NULL OR pc.category_id = $4)"#,
    )
    .bind(q)
    .bind(min_price)
    .bind(max_price)
    .bind(category_id)
    .fetch_one(pool)
    .await?;

    // Whitelist only — never interpolate user input into ORDER BY
    let order_sql = match sort {
        "price_asc" => "COALESCE(p.sale_price, p.base_price) ASC, p.id ASC",
        "price_desc" => "COALESCE(p.sale_price, p.base_price) DESC, p.id ASC",
        "name_asc" => "p.name ASC, p.id ASC",
        "name_desc" => "p.name DESC, p.id ASC",
        "oldest" => "p.created_at ASC, p.id ASC",
        "featured" => "p.is_featured DESC, p.created_at DESC, p.id ASC",
        _ => "p.created_at DESC, p.id ASC", // newest (default)
    };

    let sql = format!(
        r#"SELECT DISTINCT p.id, p.name, p.slug, p.description, p.short_description,
                  p.base_price, p.sale_price, p.currency, p.sku, p.is_featured, p.status, p.created_at
           FROM products p
           LEFT JOIN product_categories pc ON pc.product_id = p.id
           WHERE p.status = 'published'
             AND ($1::text IS NULL OR p.name ILIKE '%' || $1 || '%' OR p.description ILIKE '%' || $1 || '%' OR p.sku ILIKE '%' || $1 || '%')
             AND ($2::bigint IS NULL OR COALESCE(p.sale_price, p.base_price) >= $2)
             AND ($3::bigint IS NULL OR COALESCE(p.sale_price, p.base_price) <= $3)
             AND ($4::uuid IS NULL OR pc.category_id = $4)
           ORDER BY {order_sql}
           LIMIT $5 OFFSET $6"#
    );

    let rows = sqlx::query_as::<_, ProductRow>(&sql)
        .bind(q)
        .bind(min_price)
        .bind(max_price)
        .bind(category_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((rows, total.0))
}

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<ProductRow>, sqlx::Error> {
    sqlx::query_as::<_, ProductRow>(
        r#"SELECT id, name, slug, description, short_description, base_price, sale_price,
                  currency, sku, is_featured, status, created_at
           FROM products WHERE slug = $1 AND status = 'published'"#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ProductRow>, sqlx::Error> {
    sqlx::query_as::<_, ProductRow>(
        r#"SELECT id, name, slug, description, short_description, base_price, sale_price,
                  currency, sku, is_featured, status, created_at
           FROM products WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn product_images(
    pool: &PgPool,
    product_id: Uuid,
) -> Result<Vec<(Uuid, String, Option<String>, i32)>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, url, alt_text, sort_order FROM product_images
           WHERE product_id = $1 ORDER BY sort_order"#,
    )
    .bind(product_id)
    .fetch_all(pool)
    .await
}

pub async fn product_variants(
    pool: &PgPool,
    product_id: Uuid,
) -> Result<Vec<(Uuid, String, Option<String>, Option<i64>, serde_json::Value, i32)>, sqlx::Error>
{
    sqlx::query_as(
        r#"SELECT v.id, v.sku, v.name, v.price_override, v.attributes,
                  COALESCE(i.quantity_on_hand - i.quantity_reserved, 0)::int AS available
           FROM product_variants v
           LEFT JOIN inventory i ON i.variant_id = v.id
           WHERE v.product_id = $1 AND v.status = 'active'"#,
    )
    .bind(product_id)
    .fetch_all(pool)
    .await
}

pub async fn first_image_url(pool: &PgPool, product_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"SELECT url FROM product_images WHERE product_id = $1 ORDER BY sort_order LIMIT 1"#,
    )
    .bind(product_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}