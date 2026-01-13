import { error, redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

export const ssr = false;

export const load: PageLoad = async ({ params, fetch }) => {
  const recipeRes = await fetch(`/api/recipes/${params.id}`);

  if (recipeRes.status === 401) {
    throw redirect(303, "/login");
  }

  if (!recipeRes.ok) {
    throw error(recipeRes.status, "Failed to load recipe.");
  }

  const recipe = await recipeRes.json();
  const notesRes = await fetch(`/api/recipes/${params.id}/notes`);

  let notes = [];
  let notesError = "";

  if (notesRes.ok) {
    notes = await notesRes.json();
  } else {
    notesError = "Failed to load notes.";
  }

  return { recipe, notes, notesError };
};
