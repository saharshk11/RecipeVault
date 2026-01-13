import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { fetchTextWithLimit } from "$lib/server/fetch_limit";
import { extractRecipe } from "$lib/server/recipe_parser";
import { db } from "$lib/server/db";
import { nowRfc3339 } from "$lib/server/time";
import { requireFreshUserOrResponse } from "$lib/server/require_fresh_user";
import type { Recipe } from "$lib/server/recipe_types";

const MAX_HTML_BYTES = 2 * 1024 * 1024;

function parseTags(input: unknown) {
  if (!Array.isArray(input)) return [];
  return input.map((t) => String(t).trim()).filter(Boolean);
}

export async function POST(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (!gate.ok) return gate.response;
  const currentUser = gate.user;

  const body = await event.request.json().catch(() => null);
  const url = body?.url?.toString() ?? "";
  const tags = parseTags(body?.tags);

  if (!url) {
    return apiError(400, "BAD_REQUEST", "URL is required.");
  }

  let html: string;
  try {
    html = await fetchTextWithLimit(url, MAX_HTML_BYTES);
  } catch (e) {
    return apiError(502, "UPSTREAM_FETCH_FAILED", e instanceof Error ? e.message : "Failed to fetch remote page.");
  }

  let recipe: Recipe;
  try {
    recipe = await extractRecipe(html, url);
  } catch (e) {
    const msg = e instanceof Error ? e.message : "Failed to parse recipe.";
    const code = msg.includes("No recipe found") ? "NO_RECIPE_FOUND" : "RECIPE_PARSE_ERROR";
    const status = code === "NO_RECIPE_FOUND" ? 422 : 500;
    return apiError(status, code, msg);
  }

  recipe.tags = tags;
  recipe.source_url = recipe.source_url ?? url;

  const recipe_json = JSON.stringify(recipe);
  const tags_json = JSON.stringify(tags);
  const now = nowRfc3339();
  const id = crypto.randomUUID();
  const addedByUserId = currentUser.id;

  try {
    await db(event)
      .prepare(
        `INSERT INTO recipes (id, source_url, title, image_url, recipe_json, tags, added_by_user_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(source_url) DO UPDATE SET
           title = excluded.title,
           image_url = excluded.image_url,
           recipe_json = excluded.recipe_json,
           tags = excluded.tags,
           added_by_user_id = COALESCE(recipes.added_by_user_id, excluded.added_by_user_id),
           updated_at = excluded.updated_at`
      )
      .bind(
        id,
        recipe.source_url,
        recipe.title,
        recipe.image_url,
        recipe_json,
        tags_json,
        addedByUserId,
        now,
        now
      )
      .run();
  } catch (e) {
    return apiError(500, "DB_WRITE_FAILED", e instanceof Error ? e.message : "Failed to store recipe.");
  }

  const stored = await db(event)
    .prepare(`SELECT id, recipe_json, created_at, updated_at FROM recipes WHERE source_url = ?1`)
    .bind(recipe.source_url)
    .first<{ id: string; recipe_json: string; created_at: string; updated_at: string }>();

  if (!stored) {
    return apiError(500, "DB_READ_FAILED", "Failed to read stored recipe.");
  }

  let storedRecipe: Recipe;
  try {
    storedRecipe = JSON.parse(stored.recipe_json) as Recipe;
  } catch (e) {
    return apiError(500, "DB_DESERIALIZATION_ERROR", e instanceof Error ? e.message : "Failed to decode recipe.");
  }

  return json({
    id: stored.id,
    recipe: storedRecipe,
    created_at: stored.created_at,
    updated_at: stored.updated_at
  });
}
