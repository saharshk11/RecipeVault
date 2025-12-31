# Backend Usage

## Run the server

From the repo root:

```bash
cargo run -p backend
```

The server listens on `http://127.0.0.1:3000`.

## Auth endpoints

- `POST /auth/login`
- `POST /auth/logout`
- `GET /auth/me`
- `POST /auth/change-credentials`

All recipe endpoints require an authenticated session cookie. If the user has
`must_change_password = true`, access to recipe routes will be denied until
`/auth/change-credentials` succeeds.

## Cookie settings

- `RECIPE_COOKIE_SECURE=1` enables the `Secure` cookie flag (recommended for HTTPS).
