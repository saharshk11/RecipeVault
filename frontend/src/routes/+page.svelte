<script lang="ts">
  import { onMount } from "svelte";
  import {
    authMe,
    changeCredentials,
    login,
    logout,
    type AuthUser
  } from "$lib/api/auth";
  import { ApiError } from "$lib/api/http";

  let username = "";
  let password = "";
  let currentPassword = "";
  let newUsername = "";
  let newPassword = "";

  let user: AuthUser | null = null;
  let error = "";
  let notice = "";
  let loginBusy = false;
  let changeBusy = false;

  onMount(async () => {
    try {
      user = await authMe();
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
</script>

<div class="relative min-h-screen overflow-hidden bg-[#f7f0e8]">
  <div
    class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,_rgba(255,206,117,0.35),_transparent_55%),radial-gradient(circle_at_20%_20%,_rgba(92,129,255,0.25),_transparent_40%),radial-gradient(circle_at_80%_80%,_rgba(255,108,129,0.22),_transparent_45%)]"
  ></div>
  <div
    class="pointer-events-none absolute -left-20 top-16 h-56 w-56 rounded-full bg-[#f4c163] opacity-40 blur-3xl"
  ></div>
  <div
    class="pointer-events-none absolute -right-28 bottom-12 h-64 w-64 rounded-full bg-[#ff8fa4] opacity-35 blur-3xl"
  ></div>

  <div
    class="relative mx-auto grid max-w-6xl gap-12 px-6 py-16 md:grid-cols-[1.1fr_0.9fr] md:items-center"
  >
    <section class="reveal [--delay:60ms]">
      <div
        class="inline-flex items-center gap-2 rounded-full border border-[#e7c897] bg-white/80 px-4 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-[#8a5b00] shadow-sm"
      >
        <span class="h-2 w-2 rounded-full bg-[#8a5b00]"></span>
        Recipe Vault
      </div>
      <h1 class="font-display mt-6 text-4xl leading-tight text-[#1f1b16] md:text-5xl">
        Welcome, new cook.
        <span class="block text-[#b3513c]">Your pantry starts here.</span>
      </h1>
      <p class="mt-4 max-w-xl text-base text-[#3f3a33] md:text-lg">
        Sign in to save recipes, import new favorites, and keep every ingredient
        within reach. If this is your first visit, use your admin credentials to
        unlock the vault.
      </p>

      <div class="mt-8 grid gap-4 sm:grid-cols-2">
        <div
          class="reveal rounded-2xl border border-white/60 bg-white/70 p-4 text-sm text-[#3f3a33] shadow-sm [--delay:140ms]"
        >
          <p class="font-semibold text-[#1f1b16]">Import from any site</p>
          <p class="mt-1">
            Drop in a URL and let the parser pull ingredients, steps, and
            images.
          </p>
        </div>
        <div
          class="reveal rounded-2xl border border-white/60 bg-white/70 p-4 text-sm text-[#3f3a33] shadow-sm [--delay:220ms]"
        >
          <p class="font-semibold text-[#1f1b16]">Stay organized</p>
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
          <h2 class="font-display text-2xl text-[#1f1b16]">Sign in</h2>
          <p class="mt-1 text-sm text-[#5f564b]">
            Use your recipe vault credentials.
          </p>
        </div>
        {#if user}
          <button
            class="rounded-full border border-[#e7c897] px-3 py-1 text-xs font-semibold text-[#8a5b00] hover:bg-[#fff6e8]"
            on:click={handleLogout}
            type="button"
          >
            Sign out
          </button>
        {/if}
      </div>

      <form class="mt-6 grid gap-4" on:submit|preventDefault={handleLogin}>
        <label class="grid gap-2 text-sm font-medium text-[#3f3a33]">
          Username
          <input
            class="h-11 rounded-xl border border-[#e3d2b9] bg-white px-3 text-base text-[#1f1b16] shadow-sm focus:border-[#8a5b00] focus:outline-none"
            autocomplete="username"
            bind:value={username}
            placeholder="admin"
            required
          />
        </label>
        <label class="grid gap-2 text-sm font-medium text-[#3f3a33]">
          Password
          <input
            class="h-11 rounded-xl border border-[#e3d2b9] bg-white px-3 text-base text-[#1f1b16] shadow-sm focus:border-[#8a5b00] focus:outline-none"
            type="password"
            autocomplete="current-password"
            bind:value={password}
            placeholder="••••••••"
            required
          />
        </label>
        <button
          class="mt-2 h-11 rounded-xl bg-[#1f1b16] text-sm font-semibold text-white shadow-lg shadow-[#1f1b16]/20 transition hover:-translate-y-0.5 hover:bg-[#2f2a24] disabled:cursor-not-allowed disabled:opacity-70"
          disabled={loginBusy}
          type="submit"
        >
          {loginBusy ? "Signing in..." : "Enter the kitchen"}
        </button>
      </form>

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
        <div class="mt-6 border-t border-dashed border-[#e3d2b9] pt-6">
          <h3 class="font-display text-lg text-[#1f1b16]">
            Set your new credentials
          </h3>
          <p class="mt-1 text-sm text-[#5f564b]">
            The admin account requires a fresh username and password.
          </p>

          <form
            class="mt-4 grid gap-4"
            on:submit|preventDefault={handleChangeCredentials}
          >
            <label class="grid gap-2 text-sm font-medium text-[#3f3a33]">
              Current password
              <input
                class="h-11 rounded-xl border border-[#e3d2b9] bg-white px-3 text-base text-[#1f1b16] shadow-sm focus:border-[#8a5b00] focus:outline-none"
                type="password"
                autocomplete="current-password"
                bind:value={currentPassword}
                required
              />
            </label>
            <label class="grid gap-2 text-sm font-medium text-[#3f3a33]">
              New username
              <input
                class="h-11 rounded-xl border border-[#e3d2b9] bg-white px-3 text-base text-[#1f1b16] shadow-sm focus:border-[#8a5b00] focus:outline-none"
                autocomplete="username"
                bind:value={newUsername}
                required
              />
            </label>
            <label class="grid gap-2 text-sm font-medium text-[#3f3a33]">
              New password
              <input
                class="h-11 rounded-xl border border-[#e3d2b9] bg-white px-3 text-base text-[#1f1b16] shadow-sm focus:border-[#8a5b00] focus:outline-none"
                type="password"
                autocomplete="new-password"
                bind:value={newPassword}
                required
              />
            </label>
            <button
              class="h-11 rounded-xl border border-[#1f1b16] bg-[#fef7ec] text-sm font-semibold text-[#1f1b16] transition hover:-translate-y-0.5 hover:bg-[#fff1da] disabled:cursor-not-allowed disabled:opacity-70"
              disabled={changeBusy}
              type="submit"
            >
              {changeBusy ? "Updating..." : "Update credentials"}
            </button>
          </form>
        </div>
      {:else if user}
        <div class="mt-6 rounded-2xl border border-[#e7c897] bg-[#fff7e5] p-4 text-sm text-[#5f564b]">
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
</style>
