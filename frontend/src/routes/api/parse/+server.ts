import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { extractRecipe } from "$lib/server/recipe_parser";
import { fetchTextWithLimit } from "$lib/server/fetch_limit";
import { requireUser } from "$lib/server/auth";

const MAX_HTML_BYTES = 2 * 1024 * 1024;

export async function POST(event) {
  const user = await requireUser(event);
  if (!user) {
    return apiError(401, "UNAUTHORIZED", "Authentication required.");
  }
  if (user.must_change_password) {
    return apiError(403, "PASSWORD_RESET_REQUIRED", "Password reset required before accessing recipes.");
  }

  const body = await event.request.json().catch(() => null);
  const url = body?.url?.toString() ?? "";
  if (!url) {
    return apiError(400, "BAD_REQUEST", "URL is required.");
  }

  let html: string;
  try {
    html = await fetchTextWithLimit(url, MAX_HTML_BYTES);
  } catch (e) {
    return apiError(502, "UPSTREAM_FETCH_FAILED", e instanceof Error ? e.message : "Failed to fetch remote page.");
  }

  try {
    const recipe = await extractRecipe(html, url);
    return json(recipe);
  } catch (e) {
    const msg = e instanceof Error ? e.message : "Failed to parse recipe.";
    const code = msg.includes("No recipe found") ? "NO_RECIPE_FOUND" : "RECIPE_PARSE_ERROR";
    const status = code === "NO_RECIPE_FOUND" ? 422 : 500;
    return apiError(status, code, msg);
  }
}

