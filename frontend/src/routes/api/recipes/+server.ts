import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { db } from "$lib/server/db";
import { requireFreshUserOrResponse } from "$lib/server/require_fresh_user";
import type { Recipe } from "$lib/server/recipe_types";

type RecipeListItem = {
  id: string;
  title: string;
  source_url: string;
  image_url: string | null;
  created_at: string;
  updated_at: string;
  tags: string[];
  description: string | null;
  ingredients: string[];
  instructions: string[];
  favorite: boolean;
  added_by: string | null;
};

function parseJson<T>(raw: string | null, fallback: T): T {
  if (!raw) return fallback;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

export async function GET(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (!gate.ok) return gate.response;

  const rows = await db(event)
    .prepare(
      `SELECT
         r.id,
         r.title,
         r.source_url,
         r.image_url,
         r.created_at,
         r.updated_at,
         r.tags,
         r.recipe_json,
         r.favorite,
         u.username AS added_by
       FROM recipes r
       LEFT JOIN users u ON u.id = r.added_by_user_id
       ORDER BY r.created_at DESC`
    )
    .all<{
      id: string;
      title: string;
      source_url: string;
      image_url: string | null;
      created_at: string;
      updated_at: string;
      tags: string;
      recipe_json: string;
      favorite: number;
      added_by: string | null;
    }>();

  const items: RecipeListItem[] = [];
  for (const row of rows.results) {
    const tags = parseJson<string[]>(row.tags, []);
    const recipe = parseJson<Recipe>(row.recipe_json, {
      title: row.title,
      description: null,
      ingredients: [],
      instructions: [],
      servings: null,
      prep_time: null,
      cook_time: null,
      total_time: null,
      image_url: row.image_url,
      source_url: row.source_url,
      tags
    });

    items.push({
      id: row.id,
      title: row.title,
      source_url: row.source_url,
      image_url: row.image_url,
      created_at: row.created_at,
      updated_at: row.updated_at,
      tags,
      description: recipe.description ?? null,
      ingredients: recipe.ingredients ?? [],
      instructions: recipe.instructions ?? [],
      favorite: row.favorite !== 0,
      added_by: row.added_by
    });
  }

  return json(items);
}

export async function POST() {
  return apiError(405, "METHOD_NOT_ALLOWED", "Use /api/recipes/import to import recipes.");
}
