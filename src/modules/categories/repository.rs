use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CategoryRow {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list_roots(pool: &PgPool) -> Result<Vec<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"SELECT id, parent_id, name, slug, description, image_url, sort_order, is_active, created_at, updated_at
           FROM categories WHERE parent_id IS NULL AND is_active = TRUE
           ORDER BY sort_order, name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_all_active(pool: &PgPool) -> Result<Vec<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"SELECT id, parent_id, name, slug, description, image_url, sort_order, is_active, created_at, updated_at
           FROM categories WHERE is_active = TRUE
           ORDER BY sort_order, name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"SELECT id, parent_id, name, slug, description, image_url, sort_order, is_active, created_at, updated_at
           FROM categories WHERE id = $1"#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"SELECT id, parent_id, name, slug, description, image_url, sort_order, is_active, created_at, updated_at
           FROM categories WHERE slug = $1"#
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn children(pool: &PgPool, parent_id: Uuid) -> Result<Vec<CategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"SELECT id, parent_id, name, slug, description, image_url, sort_order, is_active, created_at, updated_at
           FROM categories WHERE parent_id = $1 AND is_active = TRUE
           ORDER BY sort_order, name"#
    )
    .bind(parent_id)
    .fetch_all(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    slug: &str,
    parent_id: Option<Uuid>,
    description: Option<&str>,
    image_url: Option<&str>,
    sort_order: i32,
) -> Result<CategoryRow, sqlx::Error> {
    sqlx::query_as::<_, CategoryRow>(
        r#"INSERT INTO categories (name, slug, parent_id, description, image_url, sort_order)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, parent_id, name, slug, description, image_url, sort_order, is_active, created_at, updated_at"#
    )
    .bind(name)
    .bind(slug)
    .bind(parent_id)
    .bind(description)
    .bind(image_url)
    .bind(sort_order)
    .fetch_one(pool)
    .await
}

pub async fn would_create_cycle(pool: &PgPool, id: Uuid, new_parent: Option<Uuid>) -> Result<bool, sqlx::Error> {
    let Some(parent) = new_parent else { return Ok(false) };
    if parent == id { return Ok(true) }
    // Walk up the tree
    let mut current = parent;
    loop {
        let row: Option<(Option<Uuid>,)> = sqlx::query_as(
            "SELECT parent_id FROM categories WHERE id = $1"
        )
        .bind(current)
        .fetch_optional(pool)
        .await?;
        match row {
            None => return Ok(false),
            Some((None,)) => return Ok(false),
            Some((Some(p),)) => {
                if p == id { return Ok(true) }
                current = p;
            }
        }
    }
}
