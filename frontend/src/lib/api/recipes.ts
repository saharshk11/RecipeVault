import { apiFetch } from "./http";

export type RecipeCore = {
  title: string;
  description: string | null;
  ingredients: string[];
  instructions: string[];
  servings: string | null;
  prep_time: string | null;
  cook_time: string | null;
  total_time: string | null;
  image_url: string | null;
  source_url: string | null;
  tags: string[];
};

export type RecipeListItem = {
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
};

export type GetRecipeResponse = {
  id: string;
  recipe: RecipeCore;
  created_at: string;
  updated_at: string;
};

export type ImportRecipeResponse = {
  id: string;
  recipe: RecipeCore;
  created_at: string;
  updated_at: string;
};

export type RecipeNote = {
  id: string;
  body: string;
  created_at: string;
  updated_at: string;
};

export function listRecipes() {
  return apiFetch<RecipeListItem[]>("/api/recipes");
}

export function getRecipe(id: string) {
  return apiFetch<GetRecipeResponse>(`/api/recipes/${id}`);
}

export function importRecipe(url: string, tags: string[]) {
  return apiFetch<ImportRecipeResponse>("/api/recipes/import", {
    method: "POST",
    body: JSON.stringify({ url, tags })
  });
}

export function listRecipeNotes(id: string) {
  return apiFetch<RecipeNote[]>(`/api/recipes/${id}/notes`);
}

export function createRecipeNote(id: string, body: string) {
  return apiFetch<RecipeNote>(`/api/recipes/${id}/notes`, {
    method: "POST",
    body: JSON.stringify({ body })
  });
}

export function updateRecipeNote(id: string, noteId: string, body: string) {
  return apiFetch<RecipeNote>(`/api/recipes/${id}/notes/${noteId}`, {
    method: "PATCH",
    body: JSON.stringify({ body })
  });
}

export function deleteRecipeNote(id: string, noteId: string) {
  return apiFetch<void>(`/api/recipes/${id}/notes/${noteId}`, {
    method: "DELETE"
  });
}

export function updateRecipeFavorite(id: string, favorite: boolean) {
  return apiFetch<GetRecipeResponse>(`/api/recipes/${id}`, {
    method: "PATCH",
    body: JSON.stringify({ favorite })
  });
}

export function updateRecipeTitle(id: string, title: string) {
  return apiFetch<GetRecipeResponse>(`/api/recipes/${id}`, {
    method: "PATCH",
    body: JSON.stringify({ title })
  });
}

export function deleteRecipe(id: string) {
  return apiFetch<void>(`/api/recipes/${id}`, {
    method: "DELETE"
  });
}
