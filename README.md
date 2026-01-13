# Recipe Website

A recipe “vault” app: import recipes from URLs, tag/favorite them, and add personal notes.

## Repo layout

- `frontend/`: SvelteKit app deployed to Cloudflare Pages (Workers runtime)
  - Server API routes live under `frontend/src/routes/api/**`
  - Database is Cloudflare D1 (SQLite)
- `backend/`: Rust workspace (parser + legacy server)
  - `recipe-core`: recipe extraction/parsing library
  - `recipe-core-wasm`: WASM wrapper for `recipe-core` (used by Pages when enabled)
  - `server` / `cli`: legacy local backend tooling (not used in the Pages deployment)

## Features

- Import a recipe from a URL (server fetches HTML and extracts recipe data).
- Browse recipes, filter by tags, search, and mark favorites.
- Add/edit/delete per-recipe notes.
- Cookie-based auth (HttpOnly sessions) with an admin-managed user list.

## Local development

This deployment target uses Cloudflare runtime bindings (D1). For end-to-end local dev, run the app via Wrangler so `env.DB` is available.

### 1) Install frontend dependencies

```bash
pnpm -C frontend install
```

### 2) Create/apply D1 schema

Create a D1 database and apply migrations from `frontend/migrations/`.

Common workflow (requires Wrangler + Cloudflare auth):

- `wrangler d1 create ...`
- `wrangler d1 migrations apply ...`

### 3) Run the app (Pages runtime)

```bash
pnpm -C frontend build
wrangler pages dev frontend/.svelte-kit/cloudflare
```

### 4) Bootstrap the first admin user

Set `ALLOW_BOOTSTRAP=1` in your Pages environment (or local dev vars), then:

- Open `/login` and click “Create admin user”, or
- `POST /api/admin/bootstrap` directly.

After bootstrapping, sign in and (as admin) use the “Users” dialog on the home page to create additional users.

## Authentication

- Sessions are stored in D1 (`users` + `sessions` tables).
- API uses an HttpOnly cookie (`recipe_session`) and returns `401`/`403` with JSON error bodies on auth failures.

## Recipe parsing (Rust → WASM)

The server can optionally use the Rust parser compiled to WASM.

- Enable with `USE_WASM_PARSER=1` (otherwise it falls back to a lightweight JSON-LD parser).
- WASM wrapper crate: `backend/recipe-core-wasm/README.md`.

## Tests

Frontend typecheck:

```bash
pnpm -C frontend check
```

Legacy backend tests (if you still use it locally):

```bash
cargo test --manifest-path backend/Cargo.toml -p server
```
