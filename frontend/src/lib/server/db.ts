import type { RequestEvent } from "@sveltejs/kit";

export function db(event: RequestEvent): D1Database {
  const platform = event.platform;
  const database = platform?.env?.DB;
  if (!database) {
    throw new Error(
      "Missing D1 binding: env.DB (run with Cloudflare Pages/Workers runtime, e.g. `wrangler pages dev`)"
    );
  }
  return database;
}
