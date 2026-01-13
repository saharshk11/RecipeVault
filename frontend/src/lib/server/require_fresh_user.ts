import { apiError } from "./api_error";
import { requireUser } from "./auth";

export async function requireFreshUserOrResponse(event: any) {
  const user = await requireUser(event);
  if (!user) {
    return { user: null, response: apiError(401, "UNAUTHORIZED", "Authentication required.") };
  }
  if (user.must_change_password) {
    return {
      user: null,
      response: apiError(403, "PASSWORD_RESET_REQUIRED", "Password reset required before accessing recipes.")
    };
  }
  return { user, response: null };
}

