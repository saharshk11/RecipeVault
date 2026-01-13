import type { RequestEvent } from "@sveltejs/kit";
import { apiError } from "./api_error";
import { requireUser, type AuthUser } from "./auth";

export type FreshUserResult =
  | { ok: true; user: AuthUser }
  | { ok: false; response: Response };

export async function requireFreshUserOrResponse(
  event: RequestEvent
): Promise<FreshUserResult> {
  const user = await requireUser(event);
  if (!user) {
    return {
      ok: false,
      response: apiError(401, "UNAUTHORIZED", "Authentication required.")
    };
  }
  if (user.must_change_password) {
    return {
      ok: false,
      response: apiError(
        403,
        "PASSWORD_RESET_REQUIRED",
        "Password reset required before accessing recipes."
      )
    };
  }
  return { ok: true, user };
}
