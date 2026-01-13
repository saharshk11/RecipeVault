import { apiError } from "$lib/server/api_error";
import { clearSessionCookie, deleteSession, getSessionId } from "$lib/server/auth";

export async function POST(event) {
  const sessionId = getSessionId(event);
  if (!sessionId) {
    clearSessionCookie(event);
    return new Response(null, { status: 204 });
  }

  try {
    await deleteSession(event, sessionId);
  } catch (e) {
    return apiError(500, "DB_WRITE_FAILED", e instanceof Error ? e.message : "Logout failed.");
  } finally {
    clearSessionCookie(event);
  }

  return new Response(null, { status: 204 });
}

