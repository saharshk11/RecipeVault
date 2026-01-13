import type { Recipe } from "./recipe_types";
import { extractRecipeFromHtmlFallback } from "./recipe_parser_fallback";
import { env } from "$env/dynamic/private";

// Planned: Rust -> WASM parser. This wrapper is written so we can swap in a WASM
// implementation once build + bundling is wired up.
//
// Expected WASM interface (wasm-pack style):
// - default export `init()` (optional)
// - named export `extract_recipe_json(html: string, base_url: string): string`
async function tryWasm(html: string, baseUrl: string): Promise<Recipe | null> {
  const useWasmRaw = env.USE_WASM_PARSER ?? "";
  const useWasm = useWasmRaw === "1" || useWasmRaw.toLowerCase() === "true";
  if (!useWasm) return null;

  try {
    const mod: any = await import("$lib/wasm/recipe_core_wasm");
    if (typeof mod.default === "function") {
      await mod.default();
    }
    if (typeof mod.extract_recipe_json !== "function") {
      return null;
    }
    const json = mod.extract_recipe_json(html, baseUrl);
    return JSON.parse(json) as Recipe;
  } catch {
    return null;
  }
}

export async function extractRecipe(html: string, baseUrl: string): Promise<Recipe> {
  const wasm = await tryWasm(html, baseUrl);
  if (wasm) return wasm;

  const fallback = extractRecipeFromHtmlFallback(html, baseUrl);
  if (fallback) return fallback;

  throw new Error("No recipe found at the provided URL.");
}
