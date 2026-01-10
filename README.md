# Recipe Website

A full-stack recipe “vault” app: import recipes from URLs, tag/favorite them, and add personal notes.

## Repo layout

- `backend/`: Rust workspace
  - `server`: Axum API server + SQLite (SQLx)
  - `cli`: utilities (admin bootstrap, parse URL)
  - `recipe-core`: recipe extraction/parsing library
- `frontend/`: SvelteKit UI (Tailwind)

## Features

- Import a recipe from a URL (server fetches HTML and extracts recipe data).
- Browse recipes, filter by tags, search, and mark favorites.
- Add/edit/delete per-recipe notes.
- Single-admin auth with a “must change password” first-login flow.

## Local development

### 1) Start the backend

From the repo root:

```bash
cargo run --manifest-path backend/Cargo.toml -p server
```

Defaults:
- API listens on `http://127.0.0.1:3000` (override with `SERVER_ADDR`).
- SQLite DB is created at `./dev.db` (relative to where you run the server).

### 2) Bootstrap the admin user

From the repo root:

```bash
cargo run --manifest-path backend/Cargo.toml -p cli -- admin-bootstrap
```

This prints the username and password once if it generated new credentials. The first login will require updating credentials.

### 3) Start the frontend

```bash
pnpm -C frontend install
pnpm -C frontend dev
```

Set `frontend/.env`:

```env
VITE_BACKEND_URL=http://localhost:3000
```

Then open the dev server URL printed by Vite (typically `http://localhost:5173`).

## Authentication

The backend issues an opaque token on login; the frontend stores it in `localStorage` as `recipe_token` and sends it on API requests via:

```
Authorization: Bearer <token>
```

## Configuration (backend)

- `SERVER_ADDR` (default `127.0.0.1:3000`)
- `CORS_ALLOW_ORIGIN` (comma-separated origins; default allows local Vite dev origins)

## Tests

Backend tests:

```bash
cargo test --manifest-path backend/Cargo.toml -p server
```

Frontend typecheck:

```bash
pnpm -C frontend check
```
