export type Recipe = {
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

export type RecipeNote = {
  id: string;
  body: string;
  created_at: string;
  updated_at: string;
};

