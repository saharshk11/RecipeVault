import { json } from "@sveltejs/kit";
import { bootstrapEnabled } from "$lib/server/bootstrap";

export async function GET(event) {
  if (!bootstrapEnabled(event)) {
    return new Response("Not found", { status: 404 });
  }

  if (!event.platform?.env?.DB) {
    return json(
      {
        bootstrap_enabled: true,
        db_configured: false
      },
      { status: 200 }
    );
  }

  return json({
    bootstrap_enabled: true,
    db_configured: true
  });
}
