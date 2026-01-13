import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { db } from "$lib/server/db";
import { nowRfc3339 } from "$lib/server/time";
import { requireFreshUserOrResponse } from "$lib/server/require_fresh_user";
import type { Recipe } from "$lib/server/recipe_types";

function parseJson<T>(raw: string | null): T | null {
  if (!raw) return null;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

export async function GET(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (!gate.ok) return gate.response;

  const id = event.params.id;
  const row = await db(event)
    .prepare(`SELECT id, recipe_json, created_at, updated_at FROM recipes WHERE id = ?1`)
    .bind(id)
    .first<{ id: string; recipe_json: string; created_at: string; updated_at: string }>();

  if (!row) {
    return apiError(404, "RECIPE_NOT_FOUND", "Recipe not found");
  }

  const recipe = parseJson<Recipe>(row.recipe_json);
  if (!recipe) {
    return apiError(500, "DB_DESERIALIZATION_ERROR", "Failed to decode recipe.");
  }

  return json({
    id: row.id,
    recipe,
    created_at: row.created_at,
    updated_at: row.updated_at
  });
}

export async function PATCH(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (!gate.ok) return gate.response;

  const id = event.params.id;
  const row = await db(event)
    .prepare(
      `SELECT id, recipe_json, title, tags, favorite, created_at, updated_at
       FROM recipes
       WHERE id = ?1`
    )
    .bind(id)
    .first<{
      id: string;
      recipe_json: string;
      title: string;
      tags: string;
      favorite: number;
      created_at: string;
      updated_at: string;
    }>();

  if (!row) {
    return apiError(404, "RECIPE_NOT_FOUND", "Recipe not found");
  }

  const recipe = parseJson<Recipe>(row.recipe_json);
  if (!recipe) {
    return apiError(500, "DB_DESERIALIZATION_ERROR", "Failed to decode recipe.");
  }

  const body = await event.request.json().catch(() => null);
  const patchTitle = body?.title != null ? String(body.title) : null;
  const patchTags = Array.isArray(body?.tags)
    ? (body.tags as unknown[]).map((t: unknown) => String(t))
    : null;
  const patchFavorite = typeof body?.favorite === "boolean" ? body.favorite : null;

  let title = row.title;
  let tags: string[] = parseJson<string[]>(row.tags) ?? [];
  let favorite = row.favorite !== 0;

  if (patchTitle !== null) {
    title = patchTitle;
    recipe.title = title;
  }

  if (patchTags !== null) {
    tags = patchTags;
    recipe.tags = tags;
  }

  if (patchFavorite !== null) {
    favorite = patchFavorite;
  }

  if (patchTitle === null && patchTags === null && patchFavorite === null) {
    return json({ id: row.id, recipe, created_at: row.created_at, updated_at: row.updated_at });
  }

  const updated_at = nowRfc3339();
  const recipe_json = JSON.stringify(recipe);
  const tags_json = JSON.stringify(tags);

  await db(event)
    .prepare(
      `UPDATE recipes
       SET title = ?1,
           tags = ?2,
           recipe_json = ?3,
           updated_at = ?4,
           favorite = ?5
       WHERE id = ?6`
    )
    .bind(title, tags_json, recipe_json, updated_at, favorite ? 1 : 0, row.id)
    .run();

  return json({ id: row.id, recipe, created_at: row.created_at, updated_at });
}

export async function DELETE(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (!gate.ok) return gate.response;

  const id = event.params.id;
  const result: any = await db(event).prepare(`DELETE FROM recipes WHERE id = ?1`).bind(id).run();

  // D1 doesn't guarantee a stable rows_affected shape, so follow up with existence check on failure cases.
  if (result?.success === false) {
    return apiError(500, "DB_WRITE_FAILED", "Failed to delete recipe.");
  }

  const stillThere = await db(event)
    .prepare(`SELECT id FROM recipes WHERE id = ?1`)
    .bind(id)
    .first<{ id: string }>();

  if (stillThere) {
    return apiError(500, "DB_WRITE_FAILED", "Failed to delete recipe.");
  }

  return new Response(null, { status: 204 });
}
