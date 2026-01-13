import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { requireUser, updateUserTagColors } from "$lib/server/auth";

export async function PATCH(event) {
  const user = await requireUser(event);
  if (!user) {
    return apiError(401, "UNAUTHORIZED", "Authentication required.");
  }

  const body = await event.request.json().catch(() => null);
  const tag_colors = (body?.tag_colors ?? {}) as Record<string, string>;

  const updated = await updateUserTagColors(event, user.id, tag_colors);
  if (!updated) {
    return apiError(500, "DB_READ_FAILED", "Failed to load updated user.");
  }

  return json(updated);
}

