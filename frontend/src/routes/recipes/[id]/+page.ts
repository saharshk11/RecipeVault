import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

export const ssr = false;

export const load: PageLoad = async ({ params, fetch }) => {
  const base = import.meta.env.VITE_BACKEND_URL ?? "";

  const recipeRes = await fetch(`${base}/recipes/${params.id}`, {
    credentials: "include"
  });

  if (!recipeRes.ok) {
    throw error(recipeRes.status, "Failed to load recipe.");
  }

  const recipe = await recipeRes.json();
  const notesRes = await fetch(`${base}/recipes/${params.id}/notes`, {
    credentials: "include"
  });

  let notes = [];
  let notesError = "";

  if (notesRes.ok) {
    notes = await notesRes.json();
  } else {
    notesError = "Failed to load notes.";
  }

  return { recipe, notes, notesError };
};
