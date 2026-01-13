export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public payload?: unknown
  ) {
    super(message);
  }
}

export async function apiFetch<T>(
  path: string,
  init: RequestInit = {}
): Promise<T> {
  const res = await fetch(path, {
    ...init,
    headers: {
      ...(init.body ? { "Content-Type": "application/json" } : {}),
      ...(init.headers ?? {})
    }
  });

  const text = await res.text();
  const payload = text ? safeJsonParse(text) : null;

  if (!res.ok) {
    const msg =
      (payload as any)?.message ??
      (payload as any)?.error?.message ??
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
