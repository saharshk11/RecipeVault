import { json } from "@sveltejs/kit";

export function apiError(status: number, code: string, message: string) {
  return json(
    {
      error: {
        code,
        message
      }
    },
    { status }
  );
}

