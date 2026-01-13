import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { ROLE_ADMIN, createUser, generateRandomPassword } from "$lib/server/auth";
import { db } from "$lib/server/db";
import type { RequestEvent } from "@sveltejs/kit";
import { bootstrapEnabled } from "$lib/server/bootstrap";

function generateAdminUsername() {
  const bytes = crypto.getRandomValues(new Uint8Array(4));
  const suffix = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  return `admin_${suffix}`;
}

export async function POST(event) {
  if (!bootstrapEnabled(event)) {
    return apiError(403, "FORBIDDEN", "Bootstrap is disabled.");
  }

  if (!event.platform?.env?.DB) {
    return apiError(
      500,
      "DB_NOT_CONFIGURED",
      "D1 binding is missing. In Cloudflare Pages, bind your D1 database as `DB` under Settings → Functions → D1 database bindings."
    );
  }

  try {
    const existing = await db(event)
      .prepare(`SELECT id FROM users WHERE role = ?1 LIMIT 1`)
      .bind(ROLE_ADMIN)
      .first<{ id: string }>();

    if (existing) {
      return apiError(409, "ADMIN_EXISTS", "An admin user already exists.");
    }

    const username = generateAdminUsername();
    const password = generateRandomPassword();
    const user = await createUser(event, {
      username,
      password,
      role: ROLE_ADMIN,
      must_change_password: true
    });

    return json({
      user,
      generated_credentials: {
        username,
        password
      }
    });
  } catch (e) {
    const message = e instanceof Error ? e.message : "Bootstrap failed.";
    return apiError(500, "INTERNAL_ERROR", message);
  }
}
