use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_user_cart(pool: &PgPool, user_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM carts WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn find_guest_cart(pool: &PgPool, guest_token: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM carts WHERE guest_token = $1")
        .bind(guest_token)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn create_user_cart(pool: &PgPool, user_id: Uuid) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO carts (user_id) VALUES ($1) RETURNING id"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn create_guest_cart(pool: &PgPool, guest_token: &str) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO carts (guest_token) VALUES ($1) RETURNING id"
    )
    .bind(guest_token)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn get_items(pool: &PgPool, cart_id: Uuid) -> Result<Vec<(Uuid, Uuid, String, Option<String>, String, i64, i32, i32)>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT ci.id, ci.variant_id, p.name, v.name, v.sku,
                  COALESCE(v.price_override, p.base_price),
                  ci.quantity,
                  COALESCE(i.quantity_on_hand - i.quantity_reserved, 0)::int
           FROM cart_items ci
           JOIN product_variants v ON v.id = ci.variant_id
           JOIN products p ON p.id = v.product_id
           LEFT JOIN inventory i ON i.variant_id = v.id
           WHERE ci.cart_id = $1"#
    )
    .bind(cart_id)
    .fetch_all(pool)
    .await
}

pub async fn upsert_item(pool: &PgPool, cart_id: Uuid, variant_id: Uuid, quantity: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO cart_items (cart_id, variant_id, quantity)
           VALUES ($1, $2, $3)
           ON CONFLICT (cart_id, variant_id)
           DO UPDATE SET quantity = cart_items.quantity + EXCLUDED.quantity, updated_at = NOW()"#
    )
    .bind(cart_id)
    .bind(variant_id)
    .bind(quantity)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_item_qty(pool: &PgPool, cart_id: Uuid, item_id: Uuid, quantity: i32) -> Result<bool, sqlx::Error> {
    let r = sqlx::query(
        "UPDATE cart_items SET quantity = $1, updated_at = NOW() WHERE id = $2 AND cart_id = $3"
    )
    .bind(quantity)
    .bind(item_id)
    .bind(cart_id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn delete_item(pool: &PgPool, cart_id: Uuid, item_id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM cart_items WHERE id = $1 AND cart_id = $2")
        .bind(item_id)
        .bind(cart_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn clear(pool: &PgPool, cart_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM cart_items WHERE cart_id = $1")
        .bind(cart_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn variant_exists_and_active(pool: &PgPool, variant_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: Option<(bool,)> = sqlx::query_as(
        r#"SELECT TRUE FROM product_variants v
           JOIN products p ON p.id = v.product_id
           WHERE v.id = $1 AND v.status = 'active' AND p.status = 'published'"#
    )
    .bind(variant_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}
