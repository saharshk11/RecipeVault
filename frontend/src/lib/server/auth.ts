import type { RequestEvent } from "@sveltejs/kit";
import { db } from "./db";
import { nowRfc3339, addSecondsRfc3339 } from "./time";
import { hashPassword, verifyPassword } from "./password";

export const ROLE_ADMIN = "admin";
export const ROLE_USER = "user";

export const SESSION_COOKIE_NAME = "recipe_session";
export const DEFAULT_SESSION_TTL_SECS = 60 * 60 * 24 * 30;

export type AuthUser = {
  id: string;
  username: string;
  role: string;
  must_change_password: boolean;
  tag_colors: Record<string, string>;
};

type DbUserRow = {
  id: string;
  username: string;
  password_hash: string;
  role: string;
  must_change_password: number;
  tag_colors: string;
};

function parseTagColors(raw: string | null) {
  if (!raw) return {};
  try {
    return JSON.parse(raw) as Record<string, string>;
  } catch {
    return {};
  }
}

function toAuthUser(row: Omit<DbUserRow, "password_hash">): AuthUser {
  return {
    id: row.id,
    username: row.username,
    role: row.role,
    must_change_password: row.must_change_password !== 0,
    tag_colors: parseTagColors(row.tag_colors)
  };
}

export async function findUserByUsername(event: RequestEvent, username: string) {
  return db(event)
    .prepare(
      `SELECT id, username, password_hash, role, must_change_password, tag_colors
       FROM users
       WHERE username = ?1`
    )
    .bind(username)
    .first<DbUserRow>();
}

export async function findUserById(event: RequestEvent, userId: string) {
  return db(event)
    .prepare(
      `SELECT id, username, role, must_change_password, tag_colors
       FROM users
       WHERE id = ?1`
    )
    .bind(userId)
    .first<Omit<DbUserRow, "password_hash">>();
}

export async function createUser(
  event: RequestEvent,
  input: {
    username: string;
    password: string;
    role: string;
    must_change_password: boolean;
  }
) {
  const id = crypto.randomUUID();
  const created_at = nowRfc3339();
  const updated_at = created_at;
  const password_hash = await hashPassword(input.password);

  await db(event)
    .prepare(
      `INSERT INTO users (id, username, password_hash, role, must_change_password, tag_colors, created_at, updated_at)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)`
    )
    .bind(
      id,
      input.username,
      password_hash,
      input.role,
      input.must_change_password ? 1 : 0,
      "{}",
      created_at,
      updated_at
    )
    .run();

  return {
    id,
    username: input.username,
    role: input.role,
    must_change_password: input.must_change_password,
    tag_colors: {}
  } satisfies AuthUser;
}

export async function verifyCredentials(event: RequestEvent, username: string, password: string) {
  const row = await findUserByUsername(event, username);
  if (!row) return null;
  const ok = await verifyPassword(row.password_hash, password);
  if (!ok) return null;
  return toAuthUser(row);
}

export async function verifyUserPassword(event: RequestEvent, userId: string, password: string) {
  const row = await db(event)
    .prepare(`SELECT password_hash FROM users WHERE id = ?1`)
    .bind(userId)
    .first<{ password_hash: string }>();
  if (!row) return false;
  return verifyPassword(row.password_hash, password);
}

export async function updateUserCredentials(
  event: RequestEvent,
  userId: string,
  input: { new_username: string; new_password: string; must_change_password: boolean }
) {
  const password_hash = await hashPassword(input.new_password);
  const updated_at = nowRfc3339();

  await db(event)
    .prepare(
      `UPDATE users
       SET username = ?1,
           password_hash = ?2,
           must_change_password = ?3,
           updated_at = ?4
       WHERE id = ?5`
    )
    .bind(input.new_username, password_hash, input.must_change_password ? 1 : 0, updated_at, userId)
    .run();

  const updated = await findUserById(event, userId);
  if (!updated) return null;
  return toAuthUser(updated);
}

export async function updateUserTagColors(
  event: RequestEvent,
  userId: string,
  tagColors: Record<string, string>
) {
  const updated_at = nowRfc3339();
  const json = JSON.stringify(tagColors ?? {});

  await db(event)
    .prepare(
      `UPDATE users
       SET tag_colors = ?1,
           updated_at = ?2
       WHERE id = ?3`
    )
    .bind(json, updated_at, userId)
    .run();

  const updated = await findUserById(event, userId);
  if (!updated) return null;
  return toAuthUser(updated);
}

export async function createSession(event: RequestEvent, userId: string, ttlSecs = DEFAULT_SESSION_TTL_SECS) {
  const id = crypto.randomUUID();
  const created_at = nowRfc3339();
  const expires_at = addSecondsRfc3339(ttlSecs);

  await db(event)
    .prepare(
      `INSERT INTO sessions (id, user_id, created_at, expires_at)
       VALUES (?1, ?2, ?3, ?4)`
    )
    .bind(id, userId, created_at, expires_at)
    .run();

  return { id, user_id: userId, created_at, expires_at };
}

export async function deleteSession(event: RequestEvent, sessionId: string) {
  await db(event)
    .prepare(`DELETE FROM sessions WHERE id = ?1`)
    .bind(sessionId)
    .run();
}

export async function findUserBySessionId(event: RequestEvent, sessionId: string) {
  const now = nowRfc3339();
  const row = await db(event)
    .prepare(
      `SELECT u.id, u.username, u.role, u.must_change_password, u.tag_colors
       FROM sessions s
       JOIN users u ON u.id = s.user_id
       WHERE s.id = ?1 AND s.expires_at > ?2`
    )
    .bind(sessionId, now)
    .first<Omit<DbUserRow, "password_hash">>();

  return row ? toAuthUser(row) : null;
}

export function getSessionId(event: RequestEvent) {
  return event.cookies.get(SESSION_COOKIE_NAME) ?? null;
}

export async function requireUser(event: RequestEvent) {
  const sessionId = getSessionId(event);
  if (!sessionId) return null;
  return findUserBySessionId(event, sessionId);
}

export function setSessionCookie(event: RequestEvent, sessionId: string) {
  const secure = event.url.protocol === "https:";
  event.cookies.set(SESSION_COOKIE_NAME, sessionId, {
    path: "/",
    httpOnly: true,
    sameSite: "lax",
    secure
  });
}

export function clearSessionCookie(event: RequestEvent) {
  event.cookies.delete(SESSION_COOKIE_NAME, { path: "/" });
}

export function generateRandomPassword(length = 24) {
  const alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  const bytes = crypto.getRandomValues(new Uint8Array(length));
  let out = "";
  for (const b of bytes) out += alphabet[b % alphabet.length];
  return out;
}

