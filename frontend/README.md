# Recipe Website Frontend

SvelteKit app for the recipe vault, deployed to Cloudflare Pages (Workers runtime).

## Setup

Install dependencies:

```sh
pnpm install
```

## Cloudflare bindings

This app expects a D1 database bound as `DB`. See `frontend/wrangler.toml`.

Migrations live in `frontend/migrations/`.

## Development

```sh
pnpm run dev
```

Note: the Vite/SvelteKit dev server does not provide Cloudflare `platform.env` bindings by default. For end-to-end testing with D1, use Wrangler Pages dev:

```sh
pnpm run build
wrangler pages dev .svelte-kit/cloudflare
```

## Build + Preview

```sh
pnpm run build
pnpm run preview
```

## Auth Notes

To create the first admin user, enable `ALLOW_BOOTSTRAP=1` and use the “Create admin user” button on `/login` (or `POST /api/admin/bootstrap`).
