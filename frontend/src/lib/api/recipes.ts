import { apiFetch } from "./http";

export type Recipe = {
  id: string;
  title: string;
  // add fields as your backend returns them (e.g., description, tags, created_at)
};

export function listRecipes() {
  return apiFetch<Recipe[]>("/recipes");
}

export function getRecipe(id: string) {
  return apiFetch<Recipe>(`/recipes/${id}`);
}