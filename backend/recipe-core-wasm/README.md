# `recipe-core-wasm`

This crate exposes `recipe-core` via a small `wasm-bindgen` API so it can run in the Cloudflare Workers runtime (via SvelteKit on Cloudflare Pages).

## Intended build output

This repo currently ships a placeholder module at:

- `frontend/src/lib/wasm/recipe_core_wasm/`

The long-term intent is to generate JS + `.wasm` artifacts (e.g. via `wasm-pack` or `wasm-bindgen-cli`) into that folder (or a subfolder) and have the SvelteKit server call:

- `extract_recipe_json(html, base_url)`

## Environment flag

Server-side parsing only attempts WASM when:

- `USE_WASM_PARSER=1` (or `true`)

Otherwise it falls back to a lightweight JSON-LD parser.

