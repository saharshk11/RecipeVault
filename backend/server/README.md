# Backend Usage

This Rust server is now considered **legacy** for the Cloudflare Pages deployment (the Pages app implements the API routes directly in SvelteKit under `frontend/src/routes/api/**` with D1).

## Run the server

From the repo root:

```bash
cargo run --manifest-path backend/Cargo.toml -p server
```

The server listens on `http://127.0.0.1:3000`.

## Auth endpoints

- `POST /auth/login`
- `POST /auth/logout`
- `GET /auth/me`
- `POST /auth/change-credentials`

Note: this legacy server’s auth model may differ from the Pages deployment.
