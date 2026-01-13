import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { requireUser, updateUserCredentials, verifyUserPassword } from "$lib/server/auth";

export async function POST(event) {
  const user = await requireUser(event);
  if (!user) {
    return apiError(401, "UNAUTHORIZED", "Authentication required.");
  }

  const body = await event.request.json().catch(() => null);
  const current_password = body?.current_password?.toString() ?? "";
  const new_username = body?.new_username?.toString() ?? "";
  const new_password = body?.new_password?.toString() ?? "";

  if (!current_password || !new_username || !new_password) {
    return apiError(400, "BAD_REQUEST", "Missing required fields.");
  }

  const ok = await verifyUserPassword(event, user.id, current_password);
  if (!ok) {
    return apiError(401, "UNAUTHORIZED", "Invalid credentials.");
  }

  const updated = await updateUserCredentials(event, user.id, {
    new_username,
    new_password,
    must_change_password: false
  });

  if (!updated) {
    return apiError(500, "DB_READ_FAILED", "Failed to load updated user.");
  }

  return json(updated);
}

