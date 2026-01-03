<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authMe, logout, type AuthUser } from "$lib/api/auth";
  import {
    importRecipe,
    listRecipes,
    type RecipeListItem,
    type ImportRecipeResponse
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
  import { Input } from "$lib/components/ui/input";
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

  const sortLabels: Record<SortKey, string> = {
    updated_desc: "Recently added",
    title_asc: "Alphabetical (A → Z)"
  };

  $: sortedRecipes = [...recipes].sort((a, b) => {
    if (sort === "title_asc") {
      return a.title.localeCompare(b.title);
    }
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
  });
  $: filteredRecipes = sortedRecipes.filter((recipe) => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) return true;
    const inTitle = recipe.title.toLowerCase().includes(query);
    const inTags = (recipe.tags ?? []).some((tag) =>
      tag.toLowerCase().includes(query)
    );
    return inTitle || inTags;
  });

  onMount(async () => {
    loading = true;
    loadError = "";
    try {
      user = await authMe();
      recipes = await listRecipes();
    } catch (err) {
      loadError = err instanceof ApiError ? err.message : "Failed to load recipes.";
    } finally {
      loading = false;
    }
  });

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

  function openRecipe(id: string) {
    goto(`/recipes/${id}`);
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
      tags: created.recipe.tags ?? []
    };
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
          Collect, sort, and revisit every recipe in your library.
        </p>
      </div>
      <button
        class="rounded-full border border-[color:var(--tone-border)] px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-gold)] hover:bg-[color:var(--tone-warm-100)]"
        on:click={handleLogout}
        type="button"
      >
        Sign out
      </button>
    </header>

    <section class="grid gap-6">
      <div class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl">
        <div class="relative">
          <div class="space-y-3 pr-12 sm:pr-14">
            <h2 class="font-display text-2xl text-[color:var(--tone-ink)]">
              Your recipes
            </h2>
            <p class="text-sm text-[color:var(--tone-ink-soft)]">
              {#if searchQuery.trim()}
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
            <div class="max-w-sm">
              <Input
                type="search"
                placeholder="Search recipes or tags..."
                bind:value={searchQuery}
                class="h-10 border-[color:var(--tone-border-soft)] bg-white text-[color:var(--tone-ink)] focus-visible:border-[color:var(--tone-gold)] focus-visible:ring-[color:var(--tone-gold)]/20"
              />
            </div>
          </div>
          <div class="absolute right-0 top-0">
            <Dialog bind:open={showImport}>
              <DialogTrigger
                class="flex h-10 w-10 items-center justify-center rounded-full bg-[color:var(--tone-ink)] text-2xl font-semibold text-white shadow-sm transition hover:-translate-y-0.5 hover:bg-[color:var(--tone-ink-hover)]"
              >
                +
              </DialogTrigger>
              <DialogContent class="border-white/70 bg-white/95 text-[color:var(--tone-ink)] shadow-xl">
                <DialogHeader>
                  <DialogTitle class="font-display text-xl text-[color:var(--tone-ink)]">
                    Add a new recipe
                  </DialogTitle>
                  <DialogDescription class="text-[color:var(--tone-ink-soft)]">
                    Paste a link and tag it for later.
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
            {#if searchQuery.trim()}
              No recipes match this search.
            {:else}
              No recipes yet. Import one to get started.
            {/if}
          </p>
        {:else}
          <div class="mt-6 grid gap-4">
            {#each filteredRecipes as recipe}
              <article class="flex flex-col gap-4 rounded-2xl border border-[color:var(--tone-border)] bg-white p-4 shadow-sm transition hover:shadow-md md:flex-row md:items-center">
                <a
                  class="flex flex-1 flex-col gap-4 md:flex-row md:items-center"
                  href={`/recipes/${recipe.id}`}
                  aria-label={`Open ${recipe.title}`}
                >
                  <div
                    class="h-20 w-full overflow-hidden rounded-xl bg-[color:var(--tone-warm-300)] md:h-20 md:w-28"
                  >
                    {#if recipe.image_url}
                      <img
                        src={recipe.image_url}
                        alt={recipe.title}
                        class="h-full w-full object-cover"
                        loading="lazy"
                      />
                    {/if}
                  </div>
                  <div class="flex-1">
                    <h3 class="font-display text-lg text-[color:var(--tone-ink)]">
                      {recipe.title}
                    </h3>
                    <p class="mt-1 text-xs uppercase tracking-[0.2em] text-[color:var(--tone-ink-soft)]">
                      {new Date(recipe.created_at).toLocaleDateString()}
                    </p>
                    {#if recipe.tags?.length}
                      <div class="mt-2 flex flex-wrap gap-2">
                        {#each recipe.tags as tag}
                          <span class="rounded-full border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-200)] px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-[color:var(--tone-ink)]">
                            {tag}
                          </span>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </a>
                <div class="flex flex-wrap items-center gap-3">
                  <a
                    class="rounded-full border border-[color:var(--tone-border)] px-3 py-1 text-xs font-semibold text-[color:var(--tone-gold)] hover:bg-[color:var(--tone-warm-100)]"
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
