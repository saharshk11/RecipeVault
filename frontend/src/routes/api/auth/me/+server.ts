import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { requireUser } from "$lib/server/auth";

export async function GET(event) {
  const user = await requireUser(event);
  if (!user) {
    return apiError(401, "UNAUTHORIZED", "Authentication required.");
  }
  return json(user);
}

