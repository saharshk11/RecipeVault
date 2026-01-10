# Recipe Website Frontend

SvelteKit frontend for the recipe app.

## Setup

Install dependencies:

```sh
pnpm install
```

## Environment

The frontend talks to the Rust backend via `VITE_BACKEND_URL` in `.env`:

```env
VITE_BACKEND_URL=http://localhost:3000
```

After login, the frontend stores a token in `localStorage` and sends it on API requests
as `Authorization: Bearer <token>`.

## Development

```sh
pnpm run dev
```

## Build + Preview

```sh
pnpm run build
pnpm run preview
```

## Auth Notes

If you bootstrap the backend admin user with generated credentials, the first
login will require a password change. The landing page in the frontend includes
that flow.
