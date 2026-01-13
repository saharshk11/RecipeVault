import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { db } from "$lib/server/db";
import { nowRfc3339 } from "$lib/server/time";
import { requireFreshUserOrResponse } from "$lib/server/require_fresh_user";
import type { RecipeNote } from "$lib/server/recipe_types";

function parseJson<T>(raw: string | null, fallback: T): T {
  if (!raw) return fallback;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

async function loadNotes(event, recipeId: string) {
  const row = await db(event)
    .prepare(`SELECT notes FROM recipes WHERE id = ?1`)
    .bind(recipeId)
    .first<{ notes: string }>();
  return row ? parseJson<RecipeNote[]>(row.notes, []) : null;
}

export async function GET(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (gate.response) return gate.response;

  const id = event.params.id;
  const notes = await loadNotes(event, id);
  if (!notes) {
    return apiError(404, "RECIPE_NOT_FOUND", "Recipe not found");
  }
  return json(notes);
}

export async function POST(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (gate.response) return gate.response;

  const id = event.params.id;
  const body = await event.request.json().catch(() => null);
  const noteBody = body?.body?.toString().trim() ?? "";
  if (!noteBody) {
    return apiError(400, "BAD_REQUEST", "Body is required.");
  }

  const notes = await loadNotes(event, id);
  if (!notes) {
    return apiError(404, "RECIPE_NOT_FOUND", "Recipe not found");
  }

  const now = nowRfc3339();
  const note: RecipeNote = {
    id: crypto.randomUUID(),
    body: noteBody,
    created_at: now,
    updated_at: now
  };

  notes.unshift(note);
  const updated_at = nowRfc3339();

  await db(event)
    .prepare(`UPDATE recipes SET notes = ?1, updated_at = ?2 WHERE id = ?3`)
    .bind(JSON.stringify(notes), updated_at, id)
    .run();

  return json(note);
}

