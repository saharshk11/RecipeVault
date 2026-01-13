import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { db } from "$lib/server/db";
import { nowRfc3339 } from "$lib/server/time";
import { requireFreshUserOrResponse } from "$lib/server/require_fresh_user";
import type { RecipeNote } from "$lib/server/recipe_types";
import type { RequestEvent } from "@sveltejs/kit";

function parseJson<T>(raw: string | null, fallback: T): T {
  if (!raw) return fallback;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

async function loadNotes(event: RequestEvent, recipeId: string) {
  const row = await db(event)
    .prepare(`SELECT notes FROM recipes WHERE id = ?1`)
    .bind(recipeId)
    .first<{ notes: string }>();
  return row ? parseJson<RecipeNote[]>(row.notes, []) : null;
}

export async function PATCH(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (gate.response) return gate.response;

  const id = event.params.id;
  const note_id = event.params.note_id;

  const body = await event.request.json().catch(() => null);
  const nextBody = body?.body?.toString().trim() ?? "";
  if (!nextBody) {
    return apiError(400, "BAD_REQUEST", "Body is required.");
  }

  const notes = await loadNotes(event, id);
  if (!notes) {
    return apiError(404, "RECIPE_NOT_FOUND", "Recipe not found");
  }

  const now = nowRfc3339();
  let updated: RecipeNote | null = null;
  for (const note of notes) {
    if (note.id === note_id) {
      note.body = nextBody;
      note.updated_at = now;
      updated = { ...note };
      break;
    }
  }

  if (!updated) {
    return apiError(404, "NOTE_NOT_FOUND", "Note not found");
  }

  const updated_at = nowRfc3339();
  await db(event)
    .prepare(`UPDATE recipes SET notes = ?1, updated_at = ?2 WHERE id = ?3`)
    .bind(JSON.stringify(notes), updated_at, id)
    .run();

  return json(updated);
}

export async function DELETE(event) {
  const gate = await requireFreshUserOrResponse(event);
  if (gate.response) return gate.response;

  const id = event.params.id;
  const note_id = event.params.note_id;

  const notes = await loadNotes(event, id);
  if (!notes) {
    return apiError(404, "RECIPE_NOT_FOUND", "Recipe not found");
  }

  const before = notes.length;
  const remaining = notes.filter((n) => n.id !== note_id);
  if (remaining.length === before) {
    return apiError(404, "NOTE_NOT_FOUND", "Note not found");
  }

  const updated_at = nowRfc3339();
  await db(event)
    .prepare(`UPDATE recipes SET notes = ?1, updated_at = ?2 WHERE id = ?3`)
    .bind(JSON.stringify(remaining), updated_at, id)
    .run();

  return new Response(null, { status: 204 });
}
