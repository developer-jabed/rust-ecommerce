use super::dto::*;
use super::repository;
use crate::errors::AppError;
use crate::response::PaginationMeta;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ProductService<'a> {
    pool: &'a PgPool,
}

impl<'a> ProductService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        query: ProductQuery,
    ) -> Result<(Vec<ProductListItem>, PaginationMeta), AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100).max(1);
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let category_id = if let Some(ref cat) = query.category {
            repository::resolve_category_id(self.pool, cat).await?
        } else {
            None
        };

        // Requested category not found → empty result (not all products)
        if query.category.is_some() && category_id.is_none() {
            let meta = PaginationMeta::new(page, per_page, 0);
            return Ok((Vec::new(), meta));
        }

        let sort = query.sort.as_deref().unwrap_or("newest");

        let (rows, total) = repository::list_published(
            self.pool,
            limit,
            offset,
            query.q.as_deref(),
            query.min_price,
            query.max_price,
            category_id,
            sort,
        )
        .await?;

        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            let image_url = repository::first_image_url(self.pool, r.id)
                .await
                .ok()
                .flatten();
            items.push(ProductListItem {
                id: r.id,
                name: r.name,
                slug: r.slug,
                short_description: r.short_description,
                base_price: r.base_price,
                sale_price: r.sale_price,
                currency: r.currency,
                is_featured: r.is_featured,
                image_url,
                status: r.status,
            });
        }

        let meta = PaginationMeta::new(page, per_page, total);
        Ok((items, meta))
    }

    pub async fn get_one(&self, id_or_slug: &str) -> Result<ProductDetail, AppError> {
        let row = if let Ok(id) = Uuid::parse_str(id_or_slug) {
            repository::find_by_id(self.pool, id).await?
        } else {
            repository::find_by_slug(self.pool, id_or_slug).await?
        }
        .ok_or_else(|| AppError::not_found("Product not found"))?;

        if row.status != "published" {
            return Err(AppError::not_found("Product not found"));
        }

        let images = repository::product_images(self.pool, row.id)
            .await?
            .into_iter()
            .map(|(id, url, alt, sort)| ProductImage {
                id,
                url,
                alt_text: alt,
                sort_order: sort,
            })
            .collect();

        let variants = repository::product_variants(self.pool, row.id)
            .await?
            .into_iter()
            .map(|(id, sku, name, price_override, attrs, available)| VariantResponse {
                id,
                sku,
                name,
                price: price_override.unwrap_or(row.base_price),
                attributes: attrs,
                available,
            })
            .collect();

        Ok(ProductDetail {
            id: row.id,
            name: row.name,
            slug: row.slug,
            description: row.description,
            short_description: row.short_description,
            base_price: row.base_price,
            sale_price: row.sale_price,
            currency: row.currency,
            sku: row.sku,
            is_featured: row.is_featured,
            status: row.status,
            images,
            variants,
            created_at: row.created_at,
        })
    }
}