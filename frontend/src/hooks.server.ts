import { redirect } from "@sveltejs/kit";

const PUBLIC_PATH_PREFIXES = ["/login", "/_app", "/favicon", "/robots.txt"];

export async function handle({ event, resolve }) {
  const pathname = event.url.pathname;
  if (PUBLIC_PATH_PREFIXES.some((prefix) => pathname.startsWith(prefix))) {
    return resolve(event);
  }

  const cookieHeader = event.request.headers.get("cookie");
  if (!cookieHeader) {
    throw redirect(303, "/login");
  }

  const backend = process.env.VITE_BACKEND_URL;
  const authUrl = backend
    ? `${backend.replace(/\/$/, "")}/auth/me`
    : `${event.url.origin}/auth/me`;

  const response = await fetch(authUrl, {
    headers: {
      cookie: cookieHeader,
      accept: "application/json"
    }
  });

  if (!response.ok) {
    throw redirect(303, "/login");
  }

  return resolve(event);
}
