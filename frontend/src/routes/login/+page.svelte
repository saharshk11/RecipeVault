<script lang="ts">
  import { onMount } from "svelte";
  import {
    authMe,
    changeCredentials,
    login,
    logout,
    type AuthUser
  } from "$lib/api/auth";
  import { bootstrapAdmin } from "$lib/api/admin";
  import { ApiError } from "$lib/api/http";
  import { goto } from "$app/navigation";

  let username = "";
  let password = "";
  let currentPassword = "";
  let newUsername = "";
  let newPassword = "";
  let confirmNewPassword = "";

  let user: AuthUser | null = null;
  let error = "";
  let notice = "";
  let loginBusy = false;
  let changeBusy = false;
  let bootstrapBusy = false;
  let bootstrapError = "";
  let bootstrapCreds: { username: string; password: string } | null = null;
  let showBootstrap = false;

  onMount(async () => {
    try {
      const res = await fetch("/api/admin/bootstrap-status");
      if (res.ok) {
        showBootstrap = true;
      }
    } catch {
      // ignore
    }

    try {
      user = await authMe();
      if (user && !user.must_change_password) {
        goto("/");
      }
    } catch {
      user = null;
    }
  });

  async function handleLogin() {
    error = "";
    notice = "";
    loginBusy = true;
    try {
      const result = await login(username, password);
      user = result.user;
      notice = `Signed in as ${result.user.username}.`;
      password = "";
      if (!result.user.must_change_password) {
        goto("/");
      }
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Login failed.";
    } finally {
      loginBusy = false;
    }
  }

  async function handleChangeCredentials() {
    error = "";
    notice = "";
    changeBusy = true;
    if (newPassword !== confirmNewPassword) {
      error = "New passwords do not match.";
      changeBusy = false;
      return;
    }
    try {
      const updated = await changeCredentials(
        currentPassword,
        newUsername,
        newPassword
      );
      user = updated;
      notice = "Credentials updated. You're good to go.";
      currentPassword = "";
      newUsername = "";
      newPassword = "";
      confirmNewPassword = "";
      goto("/");
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Update failed.";
    } finally {
      changeBusy = false;
    }
  }

  async function handleLogout() {
    error = "";
    notice = "";
    try {
      await logout();
    } finally {
      user = null;
    }
  }

  async function handleBootstrap() {
    bootstrapError = "";
    notice = "";
    bootstrapCreds = null;
    bootstrapBusy = true;
    try {
      const result = await bootstrapAdmin();
      bootstrapCreds = result.generated_credentials;
      username = result.generated_credentials.username;
      password = result.generated_credentials.password;
      notice = "Admin user bootstrapped. Copy the generated password now (it won't be shown again).";
    } catch (err) {
      bootstrapError = err instanceof ApiError ? err.message : "Bootstrap failed.";
    } finally {
      bootstrapBusy = false;
    }
  }
</script>

<div class="relative min-h-screen overflow-hidden bg-[color:var(--tone-cream)]">
  <div
    class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,_var(--tone-glow-sun),_transparent_55%),radial-gradient(circle_at_20%_20%,_var(--tone-glow-sky),_transparent_40%),radial-gradient(circle_at_80%_80%,_var(--tone-glow-rose),_transparent_45%)]"
  ></div>
  <div
    class="pointer-events-none absolute -left-20 top-16 h-56 w-56 rounded-full bg-[color:var(--tone-glow-sun-solid)] opacity-40 blur-3xl"
  ></div>
  <div
    class="pointer-events-none absolute -right-28 bottom-12 h-64 w-64 rounded-full bg-[color:var(--tone-glow-rose-solid)] opacity-35 blur-3xl"
  ></div>

  <div
    class="relative mx-auto grid max-w-6xl gap-12 px-6 py-16 md:grid-cols-[1.1fr_0.9fr] md:items-center"
  >
    <section class="reveal [--delay:60ms]">
      <div
        class="inline-flex items-center gap-2 rounded-full border border-[color:var(--tone-border)] bg-white/80 px-4 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-[color:var(--tone-gold)] shadow-sm"
      >
        <span class="h-2 w-2 rounded-full bg-[color:var(--tone-gold)]"></span>
        Recipe Vault
      </div>
      <h1 class="font-display mt-6 text-4xl leading-tight text-[color:var(--tone-ink)] md:text-5xl">
        Welcome, new cook.
        <span class="block text-[color:var(--tone-accent)]">
          Your pantry starts here.
        </span>
      </h1>
      <p class="mt-4 max-w-xl text-base text-[color:var(--tone-ink-muted)] md:text-lg">
        Sign in to save recipes, import new favorites, and keep every ingredient
        within reach. If this is your first visit, use your admin credentials to
        unlock the vault.
      </p>

      <div class="mt-8 grid gap-4 sm:grid-cols-2">
        <div
          class="reveal rounded-2xl border border-white/60 bg-white/70 p-4 text-sm text-[color:var(--tone-ink-muted)] shadow-sm [--delay:140ms]"
        >
          <p class="font-semibold text-[color:var(--tone-ink)]">
            Import from any site
          </p>
          <p class="mt-1">
            Drop in a URL and let the parser pull ingredients, steps, and
            images.
          </p>
        </div>
        <div
          class="reveal rounded-2xl border border-white/60 bg-white/70 p-4 text-sm text-[color:var(--tone-ink-muted)] shadow-sm [--delay:220ms]"
        >
          <p class="font-semibold text-[color:var(--tone-ink)]">
            Stay organized
          </p>
          <p class="mt-1">
            Tag dishes by mood, meal, or season so the right recipe shows up.
          </p>
        </div>
      </div>
    </section>

    <section
      class="reveal rounded-3xl border border-white/70 bg-white/80 p-8 shadow-xl backdrop-blur [--delay:120ms]"
    >
      <div class="flex items-start justify-between gap-4">
        <div>
          <h2 class="font-display text-2xl text-[color:var(--tone-ink)]">
            Sign in
          </h2>
          <p class="mt-1 text-sm text-[color:var(--tone-ink-soft)]">
            Use your recipe vault credentials.
          </p>
        </div>
        {#if user}
          <button
            class="rounded-full border border-[color:var(--tone-border)] px-3 py-1 text-xs font-semibold text-[color:var(--tone-gold)] hover:bg-[color:var(--tone-warm-100)]"
            on:click={handleLogout}
            type="button"
          >
            Sign out
          </button>
        {/if}
      </div>

      <form class="mt-6 grid gap-4" on:submit|preventDefault={handleLogin}>
        <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
          Username
          <input
            class="h-11 rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 text-base text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
            autocomplete="username"
            bind:value={username}
            placeholder="admin"
            required
          />
        </label>
        <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
          Password
          <input
            class="h-11 rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 text-base text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
            type="password"
            autocomplete="current-password"
            bind:value={password}
            placeholder="••••••••"
            required
          />
        </label>
        <button
          class="shadow-ink mt-2 h-11 rounded-xl bg-[color:var(--tone-ink)] text-sm font-semibold text-white transition hover:-translate-y-0.5 hover:bg-[color:var(--tone-ink-hover)] disabled:cursor-not-allowed disabled:opacity-70"
          disabled={loginBusy}
          type="submit"
        >
          {loginBusy ? "Signing in..." : "Enter the kitchen"}
        </button>
      </form>

      {#if showBootstrap}
        <div class="mt-6 border-t border-dashed border-[color:var(--tone-border-soft)] pt-6">
          <h3 class="font-display text-lg text-[color:var(--tone-ink)]">Bootstrap (admin)</h3>
          <p class="mt-1 text-sm text-[color:var(--tone-ink-soft)]">
            Creates the first admin user (only works when the server sets <code class="font-mono">ALLOW_BOOTSTRAP=1</code>).
          </p>
          <button
            class="shadow-ink mt-4 h-11 w-full rounded-xl bg-[color:var(--tone-ink)] text-sm font-semibold text-white transition hover:-translate-y-0.5 hover:bg-[color:var(--tone-ink-hover)] disabled:cursor-not-allowed disabled:opacity-70"
            disabled={bootstrapBusy}
            type="button"
            on:click={handleBootstrap}
          >
            {bootstrapBusy ? "Bootstrapping..." : "Create admin user"}
          </button>

          {#if bootstrapError}
            <p class="mt-4 rounded-xl border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
              {bootstrapError}
            </p>
          {/if}

          {#if bootstrapCreds}
            <p class="mt-4 rounded-xl border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-100)] px-3 py-2 text-sm">
              Generated password: <span class="font-mono">{bootstrapCreds.password}</span>
            </p>
          {/if}
        </div>
      {/if}

      {#if error}
        <p class="mt-4 rounded-xl border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {error}
        </p>
      {/if}
      {#if notice}
        <p class="mt-4 rounded-xl border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          {notice}
        </p>
      {/if}

      {#if user && user.must_change_password}
        <div class="mt-6 border-t border-dashed border-[color:var(--tone-border-soft)] pt-6">
          <h3 class="font-display text-lg text-[color:var(--tone-ink)]">
            Set your new credentials
          </h3>
          <p class="mt-1 text-sm text-[color:var(--tone-ink-soft)]">
            The admin account requires a fresh username and password.
          </p>

          <form
            class="mt-4 grid gap-4"
            on:submit|preventDefault={handleChangeCredentials}
          >
            <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
              Current password
              <input
                class="h-11 rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 text-base text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
                type="password"
                autocomplete="current-password"
                bind:value={currentPassword}
                required
              />
            </label>
            <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
              New username
              <input
                class="h-11 rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 text-base text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
                autocomplete="username"
                bind:value={newUsername}
                required
              />
            </label>
            <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
              New password
              <input
                class="h-11 rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 text-base text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
                type="password"
                autocomplete="new-password"
                bind:value={newPassword}
                required
              />
            </label>
            <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
              Confirm new password
              <input
                class="h-11 rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 text-base text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
                type="password"
                autocomplete="new-password"
                bind:value={confirmNewPassword}
                required
              />
            </label>
            <button
              class="h-11 rounded-xl border border-[color:var(--tone-ink)] bg-[color:var(--tone-warm-300)] text-sm font-semibold text-[color:var(--tone-ink)] transition hover:-translate-y-0.5 hover:bg-[color:var(--tone-warm-400)] disabled:cursor-not-allowed disabled:opacity-70"
              disabled={changeBusy}
              type="submit"
            >
              {changeBusy ? "Updating..." : "Update credentials"}
            </button>
          </form>
        </div>
      {:else if user}
        <div class="mt-6 rounded-2xl border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-200)] p-4 text-sm text-[color:var(--tone-ink-soft)]">
          You're signed in. Next up: browse recipes or import a new one.
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  @keyframes reveal {
    from {
      opacity: 0;
      transform: translateY(18px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .reveal {
    animation: reveal 0.7s ease both;
    animation-delay: var(--delay, 0ms);
  }

  .shadow-ink {
    box-shadow: 0 18px 35px -24px rgb(var(--tone-ink-rgb) / 0.2);
  }
</style>
