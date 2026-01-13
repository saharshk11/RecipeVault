// Placeholder module so the app builds before the WASM artifacts are wired up.
//
// Planned: replace this folder with wasm-pack (or wasm-bindgen) output and expose:
// - default `init()` (optional)
// - `extract_recipe_json(html: string, base_url: string): string`

export default async function init() {
  // no-op
}

export function extract_recipe_json(_html: string, _base_url: string): string {
  throw new Error("recipe_core_wasm not built");
}

