# E-Commerce API (Rust / Axum / SQLx / PostgreSQL)

Production-oriented modular monolith for a full-featured e-commerce backend.

## Implemented in this package

### Phase 1 – Foundation
- Project structure, config, DB pool, migrations, errors, responses
- Tracing, request-ID, CORS, timeouts, graceful shutdown
- Health endpoint: `GET /health`

### Phase 2 – Identity
- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/refresh` (token rotation)
- `POST /api/v1/auth/logout`
- `GET  /api/v1/auth/me`
- Argon2id password hashing
- Short-lived JWT access + opaque hashed refresh tokens with revocation

### Phase 3 – Catalog (partial)
- Hierarchical categories with tree, children, cycle prevention
  - `GET /api/v1/categories`
  - `GET /api/v1/categories/tree`
  - `GET /api/v1/categories/:id_or_slug`
  - `GET /api/v1/categories/:id/children`
- Products list + detail with variants & images
  - `GET /api/v1/products?page=&per_page=&q=&min_price=&max_price=`
  - `GET /api/v1/products/:id_or_slug`

### Phase 4 – Cart
- Guest (`X-Guest-Token` header) + authenticated carts
- `GET    /api/v1/cart`
- `POST   /api/v1/cart/items`
- `PATCH  /api/v1/cart/items/:itemId`
- `DELETE /api/v1/cart/items/:itemId`
- `DELETE /api/v1/cart`
- Server-side price calculation, stock availability in response

### Database
Full schema covering:
users, addresses, refresh_tokens, audit_logs,
brands, categories (nested), products, variants, images, attributes,
inventory + movements + reservations,
carts + cart_items, coupons,
orders + order_items, payments + events, idempotency_keys,
analytics_events, search_queries, product/category daily metrics

Money is stored as integer minor units (never float).

## Quick start

```bash
unzip ecommerce-api-full.zip
cd backend
cp .env.example .env
# Set DATABASE_URL, JWT_ACCESS_SECRET, JWT_REFRESH_SECRET

# PostgreSQL 14+
# docker run -d --name ecommerce-pg \
#   -e POSTGRES_PASSWORD=password -e POSTGRES_DB=ecommerce -p 5432:5432 postgres:16

cargo run
curl http://localhost:3000/health
```

## Example flow

```bash
# Register
curl -s -X POST http://localhost:3000/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"email":"demo@example.com","password":"secret123","full_name":"Demo User"}' | jq

# Login
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"demo@example.com","password":"secret123"}' | jq -r .data.tokens.access_token)

# Me
curl -s http://localhost:3000/api/v1/auth/me -H "Authorization: Bearer $TOKEN" | jq

# Categories tree
curl -s http://localhost:3000/api/v1/categories/tree | jq

# Products
curl -s 'http://localhost:3000/api/v1/products?page=1&per_page=10' | jq

# Guest cart
GUEST=$(uuidgen)
curl -s -X POST http://localhost:3000/api/v1/cart/items \
  -H "X-Guest-Token: $GUEST" \
  -H 'Content-Type: application/json' \
  -d '{"variant_id":"...","quantity":1}' | jq
```

## Still to implement (later phases)

- Admin product/category CRUD + image upload (Cloudinary)
- Full search with ranking, suggestions
- Atomic inventory reservation under concurrent checkout
- Checkout + order state machine
- Payment gateway adapter (SSLCommerz / aamarPay) + verified IPN
- Coupons enforcement
- Analytics event ingestion + admin reports
- Rate limiting middleware on auth endpoints
- Comprehensive integration & concurrency tests

## Architecture

Modular monolith. Each domain under `src/modules/<name>/` with
`routes → handlers → service → repository + dto`.

See original specification for the full design document.

## License

MIT
