import { apiFetch } from "./http";
import { clearToken, setToken } from "./token";

export type AuthUser = {
  id: string;
  username: string;
  role: string;
  must_change_password: boolean;
  tag_colors: Record<string, string>;
};

export type LoginResponse = {
  token: string;
  user: AuthUser;
};

export async function login(username: string, password: string) {
  const result = await apiFetch<LoginResponse>("/auth/login", {
    method: "POST",
    body: JSON.stringify({ username, password })
  });
  setToken(result.token);
  return result;
}

export async function logout() {
  try {
    await apiFetch<void>("/auth/logout", {
      method: "POST"
    });
  } finally {
    clearToken();
  }
}

export function authMe() {
  return apiFetch<AuthUser>("/auth/me");
}

export function changeCredentials(
  currentPassword: string,
  newUsername: string,
  newPassword: string
) {
  return apiFetch<AuthUser>("/auth/change-credentials", {
    method: "POST",
    body: JSON.stringify({
      current_password: currentPassword,
      new_username: newUsername,
      new_password: newPassword
    })
  });
}

export function updateTagColors(tagColors: Record<string, string>) {
  return apiFetch<AuthUser>("/auth/tag-colors", {
    method: "PATCH",
    body: JSON.stringify({ tag_colors: tagColors })
  });
}
