import { apiFetch } from "./http";

export type AuthUser = {
  id: string;
  username: string;
  role: string;
  must_change_password: boolean;
  tag_colors: Record<string, string>;
};

export type LoginResponse = {
  user: AuthUser;
};

export function login(username: string, password: string) {
  return apiFetch<LoginResponse>("/auth/login", {
    method: "POST",
    body: JSON.stringify({ username, password })
  });
}

export function logout() {
  return apiFetch<void>("/auth/logout", {
    method: "POST"
  });
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
