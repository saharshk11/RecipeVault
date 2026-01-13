import { requireUser } from "$lib/server/auth";

export async function handle({ event, resolve }) {
  if (event.platform?.env?.DB) {
    event.locals.user = (await requireUser(event)) ?? undefined;
  } else {
    event.locals.user = undefined;
  }
  return resolve(event);
}
