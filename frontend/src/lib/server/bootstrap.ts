import type { RequestEvent } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";

export function bootstrapEnabled(event: RequestEvent) {
  const raw = event.platform?.env?.ALLOW_BOOTSTRAP ?? env.ALLOW_BOOTSTRAP ?? "";
  return raw === "1" || raw.toLowerCase() === "true";
}

