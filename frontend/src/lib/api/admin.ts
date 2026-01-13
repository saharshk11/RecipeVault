import { apiFetch } from "./http";
import type { AuthUser } from "./auth";

export type BootstrapResponse = {
  user: AuthUser;
  generated_credentials: {
    username: string;
    password: string;
  };
};

export type ListUsersResponse = {
  users: AuthUser[];
};

export type CreateUserResponse = {
  user: AuthUser;
  generated_password: string | null;
};

export function bootstrapAdmin() {
  return apiFetch<BootstrapResponse>("/api/admin/bootstrap", { method: "POST" });
}

export function listUsers() {
  return apiFetch<ListUsersResponse>("/api/admin/users");
}

export function createUser(username: string, password: string, role: string) {
  return apiFetch<CreateUserResponse>("/api/admin/users", {
    method: "POST",
    body: JSON.stringify({ username, password: password || undefined, role })
  });
}

