<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { getRecipe, type GetRecipeResponse } from "$lib/api/recipes";
  import { ApiError } from "$lib/api/http";

  let recipe: GetRecipeResponse | null = null;
  let loading = true;
  let loadError = "";

  $: recipeId = page.params.id;
  $: infoItems =
    recipe?.recipe
      ? [
          { label: "Servings", value: recipe.recipe.servings },
          { label: "Prep", value: recipe.recipe.prep_time },
          { label: "Cook", value: recipe.recipe.cook_time },
          { label: "Total", value: recipe.recipe.total_time }
        ].filter((item) => item.value)
      : [];

  $: loadRecipe = async (id: string) => {
    loading = true;
    loadError = "";
    try {
      recipe = await getRecipe(id);
    } catch (err) {
      loadError =
        err instanceof ApiError ? err.message : "Failed to load recipe.";
    } finally {
      loading = false;
    }
  };

  $: if (recipeId) {
    loadRecipe(recipeId);
  }

  function goBack() {
    goto("/");
  }
</script>

<div class="min-h-screen bg-[color:var(--tone-cream)]">
  <div class="mx-auto max-w-5xl px-6">
    <div class="flex flex-col gap-12 py-12">
      {#if loading}
        <div class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl">
          <div class="h-8 w-2/3 rounded-xl bg-white/70"></div>
          <div class="mt-4 h-4 w-1/2 rounded-xl bg-white/70"></div>
          <div class="mt-6 h-56 rounded-2xl bg-white/70"></div>
        </div>
      {:else if loadError}
        <p class="rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {loadError}
        </p>
      {:else if recipe}
        <section class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl">
          <div class="flex flex-col gap-6 md:flex-row md:items-start">
          <div class="w-full max-w-sm overflow-hidden rounded-2xl bg-[color:var(--tone-warm-300)]">
            {#if recipe.recipe.image_url}
              <img
                src={recipe.recipe.image_url}
                alt={recipe.recipe.title}
                class="h-full w-full object-cover"
              />
            {:else}
              <div class="flex h-64 items-center justify-center text-sm text-[color:var(--tone-ink-soft)]">
                No image available
              </div>
            {/if}
          </div>

          <div class="flex-1 space-y-4">
            <div>
              <h1 class="font-display text-3xl text-[color:var(--tone-ink)]">
                {recipe.recipe.title}
              </h1>
              <p class="mt-2 text-sm text-[color:var(--tone-ink-soft)]">
                Added {new Date(recipe.created_at).toLocaleDateString()}
              </p>
              {#if recipe.recipe.description}
                <p class="mt-3 text-sm text-[color:var(--tone-ink-muted)]">
                  {recipe.recipe.description}
                </p>
              {/if}
            </div>

            {#if recipe.recipe.tags?.length}
              <div class="flex flex-wrap gap-2">
                {#each recipe.recipe.tags as tag}
                  <span class="rounded-full border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-200)] px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-[color:var(--tone-ink)]">
                    {tag}
                  </span>
                {/each}
              </div>
            {/if}

            {#if infoItems.length}
              <div class="grid gap-3 sm:grid-cols-2">
                {#each infoItems as item}
                  <div class="rounded-2xl border border-white/70 bg-white/70 px-4 py-3">
                    <p class="text-[10px] font-semibold uppercase tracking-[0.2em] text-[color:var(--tone-ink-soft)]">
                      {item.label}
                    </p>
                    <p class="mt-1 text-sm text-[color:var(--tone-ink)]">
                      {item.value}
                    </p>
                  </div>
                {/each}
              </div>
            {/if}

            <div class="flex flex-col gap-3">
              {#if recipe.recipe.source_url}
                <a
                  class="inline-flex w-fit items-center rounded-full border border-[color:var(--tone-border)] px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-gold)] hover:bg-[color:var(--tone-warm-100)]"
                  href={recipe.recipe.source_url}
                  target="_blank"
                  rel="noreferrer"
                >
                  View source
                </a>
              {/if}
              <button
                class="inline-flex w-fit items-center rounded-full border border-[color:var(--tone-border)] px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-gold)] hover:bg-[color:var(--tone-warm-100)]"
                type="button"
                on:click={goBack}
              >
                Back to recipes
              </button>
            </div>
          </div>
        </div>
      </section>

        <section class="grid gap-6 lg:grid-cols-[1.1fr_0.9fr]">
          <div class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl">
            <h2 class="font-display text-2xl text-[color:var(--tone-ink)]">
              Ingredients
            </h2>
            {#if recipe.recipe.ingredients.length}
              <ul class="mt-4 list-disc space-y-2 ps-5 text-sm text-[color:var(--tone-ink)]">
                {#each recipe.recipe.ingredients as item}
                  <li>{item}</li>
                {/each}
              </ul>
            {:else}
              <p class="mt-4 text-sm text-[color:var(--tone-ink-soft)]">
                No ingredients listed.
              </p>
            {/if}
          </div>

          <div class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl">
            <h2 class="font-display text-2xl text-[color:var(--tone-ink)]">
              Instructions
            </h2>
            {#if recipe.recipe.instructions.length}
              <ol class="mt-4 list-decimal space-y-3 ps-5 text-sm text-[color:var(--tone-ink)]">
                {#each recipe.recipe.instructions as step}
                  <li>{step}</li>
                {/each}
              </ol>
            {:else}
              <p class="mt-4 text-sm text-[color:var(--tone-ink-soft)]">
                No instructions listed.
              </p>
            {/if}
          </div>
        </section>
      {/if}
    </div>
  </div>
</div>
