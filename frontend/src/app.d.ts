// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			user?: {
				id: string;
				username: string;
				role: string;
				must_change_password: boolean;
				tag_colors: Record<string, string>;
			};
		}
		// interface PageData {}
		// interface PageState {}
		interface Platform {
			env: {
				DB: D1Database;
				ALLOW_BOOTSTRAP?: string;
			};
		}
	}
}

// Minimal D1 types (avoids needing @cloudflare/workers-types for now).
type D1Result<T> = { results: T[] };
interface D1PreparedStatement {
	bind(...values: unknown[]): D1PreparedStatement;
	first<T = unknown>(): Promise<T | null>;
	all<T = unknown>(): Promise<D1Result<T>>;
	run(): Promise<unknown>;
}
interface D1Database {
	prepare(query: string): D1PreparedStatement;
}

export {};
