const encoder = new TextEncoder();

function bytesToBase64(bytes: Uint8Array) {
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary);
}

function base64ToBytes(value: string) {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

function timingSafeEqual(a: Uint8Array, b: Uint8Array) {
  if (a.length !== b.length) return false;
  let result = 0;
  for (let i = 0; i < a.length; i++) result |= a[i] ^ b[i];
  return result === 0;
}

const DEFAULT_ITERATIONS = 210_000;

export async function hashPassword(password: string, iterations = DEFAULT_ITERATIONS) {
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const hash = await pbkdf2Sha256(password, salt, iterations, 32);
  return `pbkdf2_sha256$${iterations}$${bytesToBase64(salt)}$${bytesToBase64(hash)}`;
}

export async function verifyPassword(stored: string, password: string) {
  const parts = stored.split("$");
  if (parts.length !== 4) return false;
  const [scheme, iterRaw, saltB64, hashB64] = parts;
  if (scheme !== "pbkdf2_sha256") return false;

  const iterations = Number.parseInt(iterRaw, 10);
  if (!Number.isFinite(iterations) || iterations <= 0) return false;

  const salt = base64ToBytes(saltB64);
  const expected = base64ToBytes(hashB64);
  const actual = await pbkdf2Sha256(password, salt, iterations, expected.length);

  return timingSafeEqual(actual, expected);
}

async function pbkdf2Sha256(
  password: string,
  salt: Uint8Array,
  iterations: number,
  lengthBytes: number
) {
  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    encoder.encode(password),
    { name: "PBKDF2" },
    false,
    ["deriveBits"]
  );

  const saltCopy = new Uint8Array(salt.byteLength);
  saltCopy.set(salt);
  const saltBuf = saltCopy.buffer as ArrayBuffer;

  const bits = await crypto.subtle.deriveBits(
    { name: "PBKDF2", hash: "SHA-256", salt: saltBuf, iterations },
    keyMaterial,
    lengthBytes * 8
  );

  return new Uint8Array(bits);
}
