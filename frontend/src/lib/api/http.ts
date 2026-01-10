import { clearToken, getToken } from "./token";
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public payload?: unknown
  ) {
    super(message);
  }
}

const API_BASE = import.meta.env.VITE_BACKEND_URL?.replace(/\/$/, "");

function resolveApiUrl(path: string) {
  if (!API_BASE) return path;
  if (path.startsWith("http://") || path.startsWith("https://")) return path;
  if (path.startsWith("/")) return `${API_BASE}${path}`;
  return `${API_BASE}/${path}`;
}

export async function apiFetch<T>(
  path: string,
  init: RequestInit = {}
): Promise<T> {
  const token = getToken();
  const res = await fetch(resolveApiUrl(path), {
    ...init,
    headers: {
      ...(init.body ? { "Content-Type": "application/json" } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(init.headers ?? {})
    }
  });

  const text = await res.text();
  const payload = text ? safeJsonParse(text) : null;

  if (!res.ok) {
    if (res.status === 401) {
      clearToken();
    }
    const msg =
      (payload as any)?.message ??
      (payload as any)?.error ??
      `Request failed (${res.status})`;
    const safeMsg =
      typeof msg === "string" ? msg : JSON.stringify(msg ?? payload);
    throw new ApiError(safeMsg, res.status, payload);
  }

  return payload as T;
}

function safeJsonParse(text: string) {
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}
