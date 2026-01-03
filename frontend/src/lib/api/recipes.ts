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

export function listRecipes() {
  return apiFetch<RecipeListItem[]>("/recipes");
}

export function getRecipe(id: string) {
  return apiFetch<GetRecipeResponse>(`/recipes/${id}`);
}

export function importRecipe(url: string, tags: string[]) {
  return apiFetch<ImportRecipeResponse>("/recipes/import", {
    method: "POST",
    body: JSON.stringify({ url, tags })
  });
}
