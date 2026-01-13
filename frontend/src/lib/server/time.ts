export function nowRfc3339() {
  return new Date().toISOString();
}

export function addSecondsRfc3339(seconds: number) {
  return new Date(Date.now() + seconds * 1000).toISOString();
}

