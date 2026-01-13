import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { createSession, setSessionCookie, verifyCredentials } from "$lib/server/auth";

export async function POST(event) {
  const body = await event.request.json().catch(() => null);
  const username = body?.username?.toString() ?? "";
  const password = body?.password?.toString() ?? "";

  if (!username || !password) {
    return apiError(400, "BAD_REQUEST", "Username and password are required.");
  }

  const user = await verifyCredentials(event, username, password);
  if (!user) {
    return apiError(401, "UNAUTHORIZED", "Invalid credentials.");
  }

  const session = await createSession(event, user.id);
  setSessionCookie(event, session.id);

  return json({ user });
}

