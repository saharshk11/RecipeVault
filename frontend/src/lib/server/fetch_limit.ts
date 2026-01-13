export async function fetchTextWithLimit(url: string, maxBytes: number) {
  const resp = await fetch(url);
  if (!resp.ok) {
    throw new Error(`Upstream fetch failed (${resp.status})`);
  }

  const reader = resp.body?.getReader();
  if (!reader) {
    const text = await resp.text();
    if (new TextEncoder().encode(text).length > maxBytes) {
      throw new Error(`Upstream body exceeded ${maxBytes} bytes`);
    }
    return text;
  }

  const chunks: Uint8Array[] = [];
  let total = 0;
  while (true) {
    const { value, done } = await reader.read();
    if (done) break;
    if (!value) continue;
    total += value.byteLength;
    if (total > maxBytes) {
      throw new Error(`Upstream body exceeded ${maxBytes} bytes`);
    }
    chunks.push(value);
  }

  const combined = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    combined.set(chunk, offset);
    offset += chunk.byteLength;
  }

  return new TextDecoder().decode(combined);
}

