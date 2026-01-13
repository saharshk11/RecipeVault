import { json } from "@sveltejs/kit";
import { apiError } from "$lib/server/api_error";
import { ROLE_ADMIN, ROLE_USER, createUser, generateRandomPassword, requireUser } from "$lib/server/auth";
import { db } from "$lib/server/db";

function requireAdmin(user: { role: string } | null) {
  return user?.role === ROLE_ADMIN;
}

export async function GET(event) {
  const user = await requireUser(event);
  if (!requireAdmin(user)) {
    return apiError(403, "FORBIDDEN", "Admin access required.");
  }

  const rows = await db(event)
    .prepare(
      `SELECT id, username, role, must_change_password, tag_colors
       FROM users
       ORDER BY created_at DESC`
    )
    .all<{
      id: string;
      username: string;
      role: string;
      must_change_password: number;
      tag_colors: string;
    }>();

  const users = rows.results.map((row) => ({
    id: row.id,
    username: row.username,
    role: row.role,
    must_change_password: row.must_change_password !== 0,
    tag_colors: (() => {
      try {
        return JSON.parse(row.tag_colors ?? "{}");
      } catch {
        return {};
      }
    })()
  }));

  return json({ users });
}

export async function POST(event) {
  const user = await requireUser(event);
  if (!requireAdmin(user)) {
    return apiError(403, "FORBIDDEN", "Admin access required.");
  }

  const body = await event.request.json().catch(() => null);
  const username = body?.username?.toString() ?? "";
  const role = body?.role?.toString() ?? ROLE_USER;
  const passwordRaw = body?.password?.toString() ?? "";

  if (!username) {
    return apiError(400, "BAD_REQUEST", "Username is required.");
  }

  const password = passwordRaw || generateRandomPassword();
  const generated = !passwordRaw;
  const must_change_password = generated;

  if (role !== ROLE_USER && role !== ROLE_ADMIN) {
    return apiError(400, "BAD_REQUEST", "Invalid role.");
  }

  try {
    const created = await createUser(event, {
      username,
      password,
      role,
      must_change_password
    });

    return json({
      user: created,
      generated_password: generated ? password : null
    });
  } catch (e) {
    return apiError(500, "DB_WRITE_FAILED", e instanceof Error ? e.message : "Failed to create user.");
  }
}

