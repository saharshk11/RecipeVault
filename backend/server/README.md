# Backend Usage

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

All recipe endpoints require an `Authorization: Bearer <token>` header. If the user has
`must_change_password = true`, access to recipe routes will be denied until
`/auth/change-credentials` succeeds.
