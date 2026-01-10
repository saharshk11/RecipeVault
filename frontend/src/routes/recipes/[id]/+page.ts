import { error, redirect } from "@sveltejs/kit";
import { getToken } from "$lib/api/token";
import type { PageLoad } from "./$types";

export const ssr = false;

export const load: PageLoad = async ({ params, fetch }) => {
  const base = import.meta.env.VITE_BACKEND_URL ?? "";

  const token = getToken();

  const recipeRes = await fetch(`${base}/recipes/${params.id}`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {}
  });

  if (recipeRes.status === 401) {
    throw redirect(303, "/login");
  }

  if (!recipeRes.ok) {
    throw error(recipeRes.status, "Failed to load recipe.");
  }

  const recipe = await recipeRes.json();
  const notesRes = await fetch(`${base}/recipes/${params.id}/notes`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {}
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
