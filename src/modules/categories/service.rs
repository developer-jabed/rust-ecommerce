use super::dto::*;
use super::repository::{self, CategoryRow};
use crate::errors::{validation_errors, AppError};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;
use std::collections::HashMap;

pub struct CategoryService<'a> {
    pool: &'a PgPool,
}

impl<'a> CategoryService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_root(&self) -> Result<Vec<CategoryResponse>, AppError> {
        let rows = repository::list_roots(self.pool).await?;
        Ok(rows.into_iter().map(|r| row_to_resp(r, None)).collect())
    }

    pub async fn tree(&self) -> Result<Vec<CategoryResponse>, AppError> {
        let all = repository::list_all_active(self.pool).await?;
        Ok(build_tree(all))
    }

    pub async fn get_one(&self, id_or_slug: &str) -> Result<CategoryResponse, AppError> {
        let row = if let Ok(id) = Uuid::parse_str(id_or_slug) {
            repository::find_by_id(self.pool, id).await?
        } else {
            repository::find_by_slug(self.pool, id_or_slug).await?
        }
        .ok_or_else(|| AppError::not_found("Category not found"))?;
        Ok(row_to_resp(row, None))
    }

    pub async fn children(&self, id: Uuid) -> Result<Vec<CategoryResponse>, AppError> {
        let rows = repository::children(self.pool, id).await?;
        Ok(rows.into_iter().map(|r| row_to_resp(r, None)).collect())
    }

    pub async fn create(&self, req: CreateCategoryRequest) -> Result<CategoryResponse, AppError> {
        req.validate().map_err(validation_errors)?;
        if let Some(pid) = req.parent_id {
            repository::find_by_id(self.pool, pid)
                .await?
                .ok_or_else(|| AppError::bad_request("Parent category not found"))?;
        }
        let row = repository::create(
            self.pool,
            &req.name,
            &req.slug,
            req.parent_id,
            req.description.as_deref(),
            req.image_url.as_deref(),
            req.sort_order.unwrap_or(0),
        )
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db) = &e {
                if db.constraint() == Some("categories_slug_unique") {
                    return AppError::conflict("Slug already exists");
                }
            }
            AppError::from(e)
        })?;
        Ok(row_to_resp(row, None))
    }
}

fn row_to_resp(r: CategoryRow, children: Option<Vec<CategoryResponse>>) -> CategoryResponse {
    CategoryResponse {
        id: r.id,
        parent_id: r.parent_id,
        name: r.name,
        slug: r.slug,
        description: r.description,
        image_url: r.image_url,
        sort_order: r.sort_order,
        is_active: r.is_active,
        children,
        created_at: r.created_at,
    }
}

fn build_tree(rows: Vec<CategoryRow>) -> Vec<CategoryResponse> {
    let mut map: HashMap<Uuid, CategoryResponse> = HashMap::new();
    let mut roots: Vec<Uuid> = Vec::new();

    for r in &rows {
        map.insert(r.id, row_to_resp(r.clone(), Some(Vec::new())));
        if r.parent_id.is_none() {
            roots.push(r.id);
        }
    }

    for r in &rows {
        if let Some(pid) = r.parent_id {
            if let Some(child) = map.remove(&r.id) {
                if let Some(parent) = map.get_mut(&pid) {
                    if let Some(ref mut ch) = parent.children {
                        ch.push(child);
                    }
                } else {
                    // orphan – treat as root
                    map.insert(r.id, child);
                    roots.push(r.id);
                }
            }
        }
    }

    roots.into_iter().filter_map(|id| map.remove(&id)).collect()
}
