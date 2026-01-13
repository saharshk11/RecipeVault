<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authMe, logout, updateTagColors, type AuthUser } from "$lib/api/auth";
  import { createUser, listUsers } from "$lib/api/admin";
  import {
    importRecipe,
    listRecipes,
    type RecipeListItem,
    type ImportRecipeResponse,
    updateRecipeFavorite,
    updateRecipeTitle
  } from "$lib/api/recipes";
  import { ApiError } from "$lib/api/http";
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogTrigger
  } from "$lib/components/ui/dialog";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import StarIcon from "@lucide/svelte/icons/star";
  import { Input } from "$lib/components/ui/input";
  import {
    Popover,
    PopoverContent,
    PopoverTrigger
  } from "$lib/components/ui/popover";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger
  } from "$lib/components/ui/select";

  type SortKey = "updated_desc" | "title_asc";

  let user: AuthUser | null = null;
  let recipes: RecipeListItem[] = [];
  let loading = true;
  let loadError = "";

  let sort: SortKey = "updated_desc";
  let importUrl = "";
  let importTags = "";
  let importBusy = false;
  let importError = "";
  let importNotice = "";
  let showImport = false;
  let searchQuery = "";
  let filterTags: string[] = [];
  let tagColors: Record<string, string> = {};
  let tagColorsBusy = false;
  let editingTitleId: string | null = null;
  let editingTitle = "";

  let adminOpen = false;
  let users: AuthUser[] = [];
  let usersBusy = false;
  let newUserUsername = "";
  let newUserPassword = "";
  let newUserRole: "user" | "admin" = "user";
  let createUserBusy = false;
  let createUserError = "";
  let createUserNotice = "";
  let generatedPassword: string | null = null;

  const sortLabels: Record<SortKey, string> = {
    updated_desc: "Recently added",
    title_asc: "Alphabetical (A → Z)"
  };

  const tagPalette = [
    "#D6A13A",
    "#C76B45",
    "#6F9B78",
    "#4F87A1",
    "#B85C3C",
    "#9C8B68"
  ];

  $: sortedRecipes = [...recipes].sort((a, b) => {
    if (sort === "title_asc") {
      return a.title.localeCompare(b.title);
    }
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
  });
  $: availableTags = Array.from(
    new Set(recipes.flatMap((recipe) => recipe.tags ?? []))
  ).sort((a, b) => a.localeCompare(b));
  $: filteredRecipes = sortedRecipes.filter((recipe) => {
    const query = searchQuery.trim().toLowerCase();
    const matchesSearch = !query
      ? true
      : recipe.title.toLowerCase().includes(query) ||
        (recipe.description ?? "").toLowerCase().includes(query) ||
        (recipe.ingredients ?? []).some((item) =>
          item.toLowerCase().includes(query)
        ) ||
        (recipe.instructions ?? []).some((item) =>
          item.toLowerCase().includes(query)
        );
    const matchesTags =
      filterTags.length === 0 ||
      filterTags.some((tag) => recipe.tags?.includes(tag));
    return matchesTags && matchesSearch;
  });

  onMount(async () => {
    loading = true;
    loadError = "";
    try {
      user = await authMe();
      if (user.must_change_password) {
        goto("/login");
        return;
      }
      tagColors = user.tag_colors ?? {};
      recipes = await listRecipes();
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        goto("/login");
        return;
      }
      loadError = err instanceof ApiError ? err.message : "Failed to load recipes.";
    } finally {
      loading = false;
    }
  });

  $: if (adminOpen && user?.role === "admin") {
    void refreshUsers();
  }

  async function refreshUsers() {
    usersBusy = true;
    try {
      const result = await listUsers();
      users = result.users ?? [];
    } catch {
      users = [];
    } finally {
      usersBusy = false;
    }
  }

  async function handleCreateUser() {
    createUserError = "";
    createUserNotice = "";
    generatedPassword = null;
    const username = newUserUsername.trim();
    if (!username) {
      createUserError = "Username is required.";
      return;
    }

    createUserBusy = true;
    try {
      const result = await createUser(username, newUserPassword.trim(), newUserRole);
      users = [result.user, ...users];
      generatedPassword = result.generated_password;
      createUserNotice = generatedPassword
        ? "User created. Copy the generated password now (it won't be shown again)."
        : "User created.";
      newUserUsername = "";
      newUserPassword = "";
      newUserRole = "user";
    } catch (err) {
      createUserError = err instanceof ApiError ? err.message : "Failed to create user.";
    } finally {
      createUserBusy = false;
    }
  }

  async function handleImport() {
    importError = "";
    importNotice = "";
    if (!importUrl.trim()) {
      importError = "Paste a recipe URL first.";
      return;
    }
    importBusy = true;
    const tags = importTags
      .split(",")
      .map((tag) => tag.trim())
      .filter(Boolean);
    try {
      const created = await importRecipe(importUrl.trim(), tags);
      recipes = [toListItem(created, importUrl.trim()), ...recipes];
      importNotice = `Imported “${created.recipe.title}”.`;
      importUrl = "";
      importTags = "";
    } catch (err) {
      importError = err instanceof ApiError ? err.message : "Import failed.";
    } finally {
      importBusy = false;
    }
  }

  async function handleLogout() {
    await logout();
    goto("/login");
  }

  function toListItem(
    created: ImportRecipeResponse,
    fallbackUrl: string
  ): RecipeListItem {
    return {
      id: created.id,
      title: created.recipe.title,
      source_url: created.recipe.source_url ?? fallbackUrl,
      image_url: created.recipe.image_url ?? null,
      created_at: created.created_at,
      updated_at: created.updated_at,
      tags: created.recipe.tags ?? [],
      description: created.recipe.description ?? null,
      ingredients: created.recipe.ingredients ?? [],
      instructions: created.recipe.instructions ?? [],
      favorite: false
    };
  }

  async function toggleFavorite(recipeId: string) {
    const current = recipes.find((recipe) => recipe.id === recipeId);
    if (!current) return;
    const nextValue = !current.favorite;
    recipes = recipes.map((recipe) =>
      recipe.id === recipeId ? { ...recipe, favorite: nextValue } : recipe
    );
    try {
      await updateRecipeFavorite(recipeId, nextValue);
    } catch (err) {
      recipes = recipes.map((recipe) =>
        recipe.id === recipeId ? { ...recipe, favorite: !nextValue } : recipe
      );
    }
  }

  function tagChipStyle(
    tag: string,
    active: boolean,
    colors: Record<string, string>
  ) {
    const color = colors[tag];
    if (!color) {
      return active
        ? "border-color: var(--tone-gold); background-color: var(--tone-warm-100); color: var(--tone-ink);"
        : "border-color: var(--tone-border); background-color: rgba(255, 255, 255, 0.7); color: var(--tone-ink-soft);";
    }
    const bg = hexToRgba(color, active ? 0.22 : 0.12);
    const text = color;
    return `border-color: ${color}; background-color: ${bg}; color: ${text};`;
  }

  function hexToRgba(hex: string, alpha: number) {
    const cleaned = hex.replace("#", "");
    if (cleaned.length !== 6) return `rgba(0, 0, 0, ${alpha})`;
    const r = Number.parseInt(cleaned.slice(0, 2), 16);
    const g = Number.parseInt(cleaned.slice(2, 4), 16);
    const b = Number.parseInt(cleaned.slice(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }

  async function setTagColor(tag: string, color: string) {
    const previous = tagColors[tag];
    const next = { ...tagColors, [tag]: color };
    tagColors = next;
    tagColorsBusy = true;
    try {
      const updated = await updateTagColors(next);
      tagColors = updated.tag_colors ?? next;
    } catch (err) {
      if (previous) {
        tagColors = { ...tagColors, [tag]: previous };
      } else {
        const { [tag]: _, ...rest } = tagColors;
        tagColors = rest;
      }
    } finally {
      tagColorsBusy = false;
    }
  }

  function startTitleEdit(recipe: RecipeListItem) {
    editingTitleId = recipe.id;
    editingTitle = recipe.title;
  }

  function cancelTitleEdit() {
    editingTitleId = null;
    editingTitle = "";
  }

  async function saveTitleEdit(recipeId: string) {
    const title = editingTitle.trim();
    if (!title) return;
    const current = recipes.find((recipe) => recipe.id === recipeId);
    if (!current) return;
    const previous = current.title;
    recipes = recipes.map((recipe) =>
      recipe.id === recipeId ? { ...recipe, title } : recipe
    );
    try {
      await updateRecipeTitle(recipeId, title);
      cancelTitleEdit();
    } catch (err) {
      recipes = recipes.map((recipe) =>
        recipe.id === recipeId ? { ...recipe, title: previous } : recipe
      );
    }
  }
</script>

<div class="min-h-screen bg-[color:var(--tone-cream)]">
  <div class="mx-auto flex max-w-6xl flex-col gap-10 px-6 py-16">
    <header class="flex flex-wrap items-center justify-between gap-6">
      <div class="space-y-2">
        <div
          class="inline-flex items-center gap-2 rounded-full border border-[color:var(--tone-border)] bg-white/80 px-4 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-[color:var(--tone-gold)]"
        >
          <span class="h-2 w-2 rounded-full bg-[color:var(--tone-gold)]"></span>
          Recipe Vault
        </div>
        <h1 class="font-display text-3xl text-[color:var(--tone-ink)] md:text-4xl">
          {user ? `Welcome back, ${user.username}.` : "Welcome back."}
        </h1>
        <p class="text-sm text-[color:var(--tone-ink-muted)] md:text-base">
          Curate your vault, seal your favorites, and revisit every recipe.
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-3">
        {#if user?.role === "admin"}
          <Dialog bind:open={adminOpen}>
            <DialogTrigger
              class="rounded-full border border-[color:var(--tone-border)] bg-white/80 px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-ink)] hover:bg-white"
              type="button"
            >
              Users
            </DialogTrigger>
            <DialogContent class="border-white/70 bg-white/95 text-[color:var(--tone-ink)] shadow-xl">
              <DialogHeader>
                <DialogTitle class="font-display">User management</DialogTitle>
                <DialogDescription>
                  Create users and share credentials. Generated passwords are only shown once.
                </DialogDescription>
              </DialogHeader>

              <div class="mt-4 grid gap-3">
                <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
                  Username
                  <Input bind:value={newUserUsername} placeholder="new_user" />
                </label>
                <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
                  Password (optional)
                  <Input bind:value={newUserPassword} placeholder="Leave blank to generate" />
                </label>
                <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
                  Role
                  <Select type="single" bind:value={newUserRole}>
                    <SelectTrigger class="w-full rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 py-2 text-sm shadow-sm">
                      <span data-slot="select-value">{newUserRole}</span>
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="user">user</SelectItem>
                      <SelectItem value="admin">admin</SelectItem>
                    </SelectContent>
                  </Select>
                </label>

                <button
                  class="shadow-ink mt-2 h-11 rounded-xl bg-[color:var(--tone-ink)] text-sm font-semibold text-white transition hover:-translate-y-0.5 hover:bg-[color:var(--tone-ink-hover)] disabled:cursor-not-allowed disabled:opacity-70"
                  type="button"
                  disabled={createUserBusy}
                  on:click={handleCreateUser}
                >
                  {createUserBusy ? "Creating..." : "Create user"}
                </button>

                {#if createUserError}
                  <p class="rounded-xl border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                    {createUserError}
                  </p>
                {/if}
                {#if createUserNotice}
                  <p class="rounded-xl border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
                    {createUserNotice}
                  </p>
                {/if}
                {#if generatedPassword}
                  <p class="rounded-xl border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-100)] px-3 py-2 text-sm">
                    Generated password: <span class="font-mono">{generatedPassword}</span>
                  </p>
                {/if}

                <div class="mt-2 border-t border-dashed border-[color:var(--tone-border-soft)] pt-4">
                  <p class="text-xs font-semibold uppercase tracking-[0.16em] text-[color:var(--tone-ink-soft)]">
                    Existing users
                  </p>
                  {#if usersBusy}
                    <p class="mt-2 text-sm text-[color:var(--tone-ink-soft)]">Loading…</p>
                  {:else if users.length === 0}
                    <p class="mt-2 text-sm text-[color:var(--tone-ink-soft)]">No users found.</p>
                  {:else}
                    <div class="mt-3 grid gap-2">
                      {#each users as u (u.id)}
                        <div class="flex items-center justify-between rounded-xl border border-[color:var(--tone-border)] bg-white px-3 py-2">
                          <div>
                            <p class="text-sm font-semibold text-[color:var(--tone-ink)]">{u.username}</p>
                            <p class="text-xs text-[color:var(--tone-ink-soft)]">
                              role: {u.role}{u.must_change_password ? " • must change password" : ""}
                            </p>
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            </DialogContent>
          </Dialog>
        {/if}

        <button
          class="rounded-full border border-[color:var(--tone-border)] px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-gold)] hover:bg-[color:var(--tone-warm-100)]"
          on:click={handleLogout}
          type="button"
        >
          Lock vault
        </button>
      </div>
    </header>

    <section class="grid gap-6">
      <div class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl outline outline-2 outline-[color:var(--tone-border)]">
        <div class="relative">
          <div class="space-y-3">
            <div class="space-y-3 pr-12 sm:pr-14">
              <h2 class="font-display text-2xl text-[color:var(--tone-ink)]">
              Your vault
            </h2>
              <p class="text-sm text-[color:var(--tone-ink-soft)]">
                {#if searchQuery.trim() || filterTags.length}
                  {filteredRecipes.length} of {recipes.length} recipe{recipes.length === 1 ? "" : "s"}
                {:else}
                  {recipes.length} recipe{recipes.length === 1 ? "" : "s"} saved.
                {/if}
              </p>
              <Select type="single" bind:value={sort}>
                <SelectTrigger
                  class="w-full rounded-full border border-[color:var(--tone-border)] bg-white/90 px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-ink)] shadow-sm transition hover:bg-white focus-visible:border-[color:var(--tone-gold)] focus-visible:ring-[3px] focus-visible:ring-[color:var(--tone-gold)]/20 sm:w-56"
                  aria-label="Sort recipes"
                >
                <span data-slot="select-value">
                  {sortLabels[sort]}
                </span>
              </SelectTrigger>
                <SelectContent class="sm:w-56">
                  {#each Object.entries(sortLabels) as [value, label]}
                    <SelectItem value={value} class="text-xs font-semibold uppercase tracking-[0.12em]">
                      {label}
                    </SelectItem>
                  {/each}
                </SelectContent>
              </Select>
            </div>
            <div class="flex w-full flex-col gap-2">
              <Popover>
                <PopoverTrigger
                  class="w-fit rounded-full border border-[color:var(--tone-border)] bg-white/90 px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-ink)] shadow-sm transition hover:bg-white"
                >
                  Filter{filterTags.length ? ` (${filterTags.length})` : ""}
                </PopoverTrigger>
                <PopoverContent class="border-white/70 bg-white/95 text-[color:var(--tone-ink)] shadow-xl">
                  <div class="flex items-center justify-between">
                    <p class="text-xs font-semibold uppercase tracking-[0.16em] text-[color:var(--tone-ink-soft)]">
                      Tags
                    </p>
                    {#if filterTags.length}
                      <button
                        class="text-xs font-semibold uppercase tracking-[0.16em] text-[color:var(--tone-ink-soft)] hover:text-[color:var(--tone-ink)]"
                        type="button"
                        on:click={() => (filterTags = [])}
                      >
                        Clear
                      </button>
                    {/if}
                  </div>
                  {#if availableTags.length === 0}
                    <p class="mt-3 rounded-lg border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-200)] px-3 py-2 text-xs text-[color:var(--tone-ink-soft)]">
                      No tags yet.
                    </p>
                  {:else}
                      <div class="mt-3 grid gap-3">
                        {#each availableTags as tag}
                          <div class="flex flex-wrap items-center justify-between gap-2 text-sm text-[color:var(--tone-ink)]">
                            <label class="flex items-center gap-2">
                          <Checkbox
                            checked={filterTags.includes(tag)}
                            onCheckedChange={(checked) => {
                              filterTags = checked
                                ? [...filterTags, tag]
                                : filterTags.filter((value) => value !== tag);
                            }}
                          />
                              <span class="text-xs font-semibold uppercase tracking-[0.12em]">
                                {tag}
                              </span>
                            </label>
                            <div class="flex items-center gap-1">
                              {#each tagPalette as color}
                                <button
                                  class={`h-5 w-5 rounded-full border ${
                                    tagColors[tag] === color
                                      ? "border-[color:var(--tone-ink)]"
                                      : "border-transparent"
                                  }`}
                                  style={`background-color: ${color};`}
                                  type="button"
                                  aria-label={`Set ${tag} color`}
                                  on:click={() => setTagColor(tag, color)}
                                  disabled={tagColorsBusy}
                                ></button>
                              {/each}
                            </div>
                          </div>
                        {/each}
                      </div>
                    {/if}
                </PopoverContent>
              </Popover>
              <Input
                type="search"
                placeholder="Search recipes"
                bind:value={searchQuery}
                list="recipe-title-suggestions"
                class="h-10 w-full border-[color:var(--tone-border-soft)] bg-white text-[color:var(--tone-ink)] focus-visible:border-[color:var(--tone-gold)] focus-visible:ring-[color:var(--tone-gold)]/20"
              />
              <datalist id="recipe-title-suggestions">
                {#each recipes as recipe}
                  <option value={recipe.title}></option>
                {/each}
              </datalist>
            </div>
          </div>
          <div class="absolute right-0 top-0">
            <Dialog bind:open={showImport}>
              <DialogTrigger
                class="group flex h-10 w-10 items-center justify-center overflow-hidden rounded-full bg-[color:var(--tone-ink)] text-white shadow-sm transition-[width,transform,background-color,padding] duration-600 ease-out hover:w-44 hover:-translate-y-0.5 hover:bg-[color:var(--tone-ink-hover)] hover:justify-start hover:pl-4"
              >
                <span class="text-2xl font-semibold leading-none transition-transform duration-600 ease-out group-hover:-rotate-180">+</span>
                <span class="max-w-0 flex-1 overflow-hidden whitespace-nowrap text-xs font-semibold uppercase tracking-[0.12em] opacity-0 transition-[max-width,opacity] duration-600 ease-out group-hover:max-w-[10rem] group-hover:opacity-100">
                  Add to vault
                </span>
              </DialogTrigger>
              <DialogContent class="border-white/70 bg-white/95 text-[color:var(--tone-ink)] shadow-xl">
                <DialogHeader>
                <DialogTitle class="font-display text-xl text-[color:var(--tone-ink)]">
                    Add to your vault
                </DialogTitle>
                <DialogDescription class="text-[color:var(--tone-ink-soft)]">
                    Seal a new recipe and tag it for later.
                </DialogDescription>
                </DialogHeader>
                <form class="grid gap-4" on:submit|preventDefault={handleImport}>
                  <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
                    Recipe URL
                    <Input
                      type="url"
                      placeholder="https://..."
                      bind:value={importUrl}
                      class="h-11 border-[color:var(--tone-border-soft)] bg-white text-[color:var(--tone-ink)] focus-visible:border-[color:var(--tone-gold)] focus-visible:ring-[color:var(--tone-gold)]/20"
                      required
                    />
                  </label>
                  <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
                    Tags
                    <Input
                      type="text"
                      placeholder="weeknight, italian, vegetarian"
                      bind:value={importTags}
                      class="h-11 border-[color:var(--tone-border-soft)] bg-white text-[color:var(--tone-ink)] focus-visible:border-[color:var(--tone-gold)] focus-visible:ring-[color:var(--tone-gold)]/20"
                    />
                  </label>
                  <button
                    class="shadow-ink h-11 rounded-xl bg-[color:var(--tone-ink)] text-sm font-semibold text-white transition hover:-translate-y-0.5 hover:bg-[color:var(--tone-ink-hover)] disabled:cursor-not-allowed disabled:opacity-70"
                    type="submit"
                    disabled={importBusy}
                  >
                    {importBusy ? "Importing..." : "Import recipe"}
                  </button>
                </form>

                {#if importError}
                  <p class="rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                    {importError}
                  </p>
                {/if}
                {#if importNotice}
                  <p class="rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
                    {importNotice}
                  </p>
                {/if}
              </DialogContent>
            </Dialog>
          </div>
        </div>

        {#if loading}
          <div class="mt-6 space-y-3 text-sm text-[color:var(--tone-ink-soft)]">
            <div class="h-16 rounded-2xl bg-white/70"></div>
            <div class="h-16 rounded-2xl bg-white/70"></div>
            <div class="h-16 rounded-2xl bg-white/70"></div>
          </div>
        {:else if loadError}
          <p class="mt-6 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
            {loadError}
          </p>
        {:else if filteredRecipes.length === 0}
          <p class="mt-6 rounded-2xl border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-200)] px-4 py-3 text-sm text-[color:var(--tone-ink-soft)]">
            {#if searchQuery.trim() || filterTags.length}
              No recipes match these filters.
            {:else}
              No recipes yet. Import one to get started.
            {/if}
          </p>
        {:else}
          <div class="mt-6 grid gap-4">
            {#each filteredRecipes as recipe}
              <article class="flex flex-col gap-4 rounded-2xl border border-[color:var(--tone-border)] bg-white p-4 shadow-sm transition hover:shadow-md md:flex-row md:items-center">
                <div class="flex flex-1 flex-col gap-4 md:flex-row md:items-center">
                  <a
                    class="h-20 w-full overflow-hidden rounded-xl bg-[color:var(--tone-warm-300)] md:h-20 md:w-28"
                    href={`/recipes/${recipe.id}`}
                    aria-label={`Open ${recipe.title}`}
                  >
                    {#if recipe.image_url}
                      <img
                        src={recipe.image_url}
                        alt={recipe.title}
                        class="h-full w-full object-cover"
                        loading="lazy"
                      />
                    {/if}
                  </a>
                  <div class="flex-1">
                    <div class="flex flex-wrap items-center gap-2">
                      {#if editingTitleId === recipe.id}
                        <Input
                          type="text"
                          bind:value={editingTitle}
                          class="h-9 max-w-xs border-[color:var(--tone-border-soft)] bg-white text-[color:var(--tone-ink)] focus-visible:border-[color:var(--tone-gold)] focus-visible:ring-[color:var(--tone-gold)]/20"
                        />
                        <button
                          class="text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-ink)] hover:text-[color:var(--tone-ink-hover)]"
                          type="button"
                          on:click={() => saveTitleEdit(recipe.id)}
                          disabled={!editingTitle.trim()}
                        >
                          Save
                        </button>
                        <button
                          class="text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-ink-soft)] hover:text-[color:var(--tone-ink)]"
                          type="button"
                          on:click={cancelTitleEdit}
                        >
                          Cancel
                        </button>
                      {:else}
                        <a
                          class="font-display text-lg text-[color:var(--tone-ink)] hover:text-[color:var(--tone-ink-hover)]"
                          href={`/recipes/${recipe.id}`}
                        >
                          {recipe.title}
                        </a>
                        <button
                          class="rounded-full p-1 text-[color:var(--tone-ink-soft)] hover:text-[color:var(--tone-ink)]"
                          type="button"
                          on:click={() => startTitleEdit(recipe)}
                          aria-label="Edit title"
                        >
                          <PencilIcon class="h-4 w-4" />
                        </button>
                      {/if}
                    </div>
                    <p class="mt-1 text-xs uppercase tracking-[0.2em] text-[color:var(--tone-ink-soft)]">
                      {new Date(recipe.created_at).toLocaleDateString()}
                    </p>
                    {#if recipe.tags?.length}
                      <div class="mt-2 flex flex-wrap gap-2">
                        {#each recipe.tags as tag}
                          <button
                            class="rounded-full border px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] transition"
                            style={tagChipStyle(tag, filterTags.length === 0 || filterTags.includes(tag), tagColors)}
                            type="button"
                            on:click={() => {
                              filterTags = filterTags.includes(tag) ? [] : [tag];
                            }}
                          >
                            {tag}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>
                <div class="mt-2 flex w-full items-center justify-between gap-3 md:mt-0 md:w-auto md:flex-col md:items-end md:justify-between">
                  <button
                    class="flex items-center justify-center rounded-full border border-[color:var(--tone-border)] p-2 text-[color:var(--tone-ink)] hover:bg-[color:var(--tone-warm-100)]"
                    type="button"
                    aria-label={recipe.favorite ? "Remove favorite" : "Mark as favorite"}
                    on:click={() => toggleFavorite(recipe.id)}
                  >
                    <StarIcon
                      class="h-3.5 w-3.5"
                      fill={recipe.favorite ? "currentColor" : "none"}
                    />
                  </button>
                  <a
                    class="rounded-full border border-[color:var(--tone-border)] px-3 py-1 text-xs font-semibold text-[color:var(--tone-gold)] hover:bg-[color:var(--tone-warm-100)] md:mt-auto"
                    href={recipe.source_url}
                    target="_blank"
                    rel="noreferrer"
                  >
                    Source
                  </a>
                </div>
              </article>
            {/each}
          </div>
        {/if}
      </div>
    </section>
  </div>
</div>

<style>
  .shadow-ink {
    box-shadow: 0 18px 35px -24px rgb(var(--tone-ink-rgb) / 0.2);
  }
</style>
