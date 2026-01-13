<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    createRecipeNote,
    deleteRecipeNote,
    updateRecipeNote,
    type GetRecipeResponse,
    type RecipeNote
  } from "$lib/api/recipes";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
    DrawerTrigger
  } from "$lib/components/ui/drawer";
  import { Input } from "$lib/components/ui/input";

  export let data: {
    recipe: GetRecipeResponse;
    notes: RecipeNote[];
    notesError?: string;
  };

  let recipe: GetRecipeResponse | null = null;
  let notes: RecipeNote[] = [];
  let notesError = "";
  let newNote = "";
  let editingNoteId: string | null = null;
  let editingBody = "";
  let notesBusy = false;
  let convertOpen = false;
  let convertValue = "";
  let convertFrom = "cups";
  let convertTo = "ml";

  function formatIsoDuration(raw: string | null) {
    if (!raw) return null;
    if (!raw.startsWith("P")) return raw;

    const match = raw.match(
      /^P(?:(\d+)D)?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?)?$/
    );
    if (!match) return raw;

    const days = Number.parseInt(match[1] ?? "0", 10) || 0;
    const hours = Number.parseInt(match[2] ?? "0", 10) || 0;
    const minutes = Number.parseInt(match[3] ?? "0", 10) || 0;
    const seconds = Number.parseInt(match[4] ?? "0", 10) || 0;

    const parts: string[] = [];
    if (days) parts.push(`${days} day${days === 1 ? "" : "s"}`);
    if (hours) parts.push(`${hours} hr${hours === 1 ? "" : "s"}`);
    if (minutes) parts.push(`${minutes} min`);
    if (!parts.length && seconds) parts.push(`${seconds} sec`);
    return parts.length ? parts.join(" ") : raw;
  }

  $: recipe = data.recipe;
  $: notes = data.notes;
  $: notesError = data.notesError ?? "";
  $: recipeId = recipe?.id ?? "";
  $: infoItems =
    recipe?.recipe
      ? [
          { label: "Servings", value: recipe.recipe.servings },
          { label: "Prep", value: formatIsoDuration(recipe.recipe.prep_time) },
          { label: "Cook", value: formatIsoDuration(recipe.recipe.cook_time) },
          { label: "Total", value: formatIsoDuration(recipe.recipe.total_time) }
        ].filter((item) => item.value)
      : [];

  const unitLabels: Record<string, string> = {
    tsp: "Teaspoons (tsp)",
    tbsp: "Tablespoons (tbsp)",
    cups: "Cups",
    ml: "Milliliters (ml)",
    l: "Liters (l)",
    floz: "Fluid oz (fl oz)"
  };

  const conversions: Record<string, Record<string, number>> = {
    tsp: { tbsp: 1 / 3, cups: 1 / 48, ml: 4.92892, l: 0.00492892, floz: 1 / 6 },
    tbsp: { tsp: 3, cups: 1 / 16, ml: 14.7868, l: 0.0147868, floz: 1 / 2 },
    cups: { tsp: 48, tbsp: 16, ml: 236.588, l: 0.236588, floz: 8 },
    ml: { tsp: 0.202884, tbsp: 0.067628, cups: 1 / 236.588, l: 0.001, floz: 0.033814 },
    l: { tsp: 202.884, tbsp: 67.628, cups: 4.22675, ml: 1000, floz: 33.814 },
    floz: { tsp: 6, tbsp: 2, cups: 1 / 8, ml: 29.5735, l: 0.0295735 }
  };

  $: convertAmount = Number.parseFloat(convertValue);
  $: convertResult =
    Number.isFinite(convertAmount) && convertFrom !== convertTo
      ? convertAmount * (conversions[convertFrom]?.[convertTo] ?? 1)
      : Number.isFinite(convertAmount)
        ? convertAmount
        : null;

  function goBack() {
    goto("/");
  }

  async function handleAddNote() {
    const body = newNote.trim();
    if (!body || !recipeId) return;
    notesBusy = true;
    try {
      const created = await createRecipeNote(recipeId, body);
      notes = [created, ...notes];
      newNote = "";
    } catch (err) {
      notesError = "Failed to add note.";
    } finally {
      notesBusy = false;
    }
  }

  async function addConversionNote() {
    if (convertResult === null || !Number.isFinite(convertAmount) || !recipeId) return;
    const body = `${convertAmount} ${convertFrom} = ${convertResult.toFixed(2)} ${convertTo}`;
    notesBusy = true;
    try {
      const created = await createRecipeNote(recipeId, body);
      notes = [created, ...notes];
    } catch (err) {
      notesError = "Failed to add note.";
    } finally {
      notesBusy = false;
    }
  }

  function startEdit(note: RecipeNote) {
    editingNoteId = note.id;
    editingBody = note.body;
  }

  function cancelEdit() {
    editingNoteId = null;
    editingBody = "";
  }

  async function saveEdit(noteId: string) {
    const body = editingBody.trim();
    if (!body || !recipeId) return;
    notesBusy = true;
    try {
      const updated = await updateRecipeNote(recipeId, noteId, body);
      notes = notes.map((note) => (note.id === noteId ? updated : note));
      cancelEdit();
    } catch (err) {
      notesError = "Failed to update note.";
    } finally {
      notesBusy = false;
    }
  }

  async function removeNote(noteId: string) {
    if (!recipeId) return;
    notesBusy = true;
    try {
      await deleteRecipeNote(recipeId, noteId);
      notes = notes.filter((note) => note.id !== noteId);
    } catch (err) {
      notesError = "Failed to delete note.";
    } finally {
      notesBusy = false;
    }
  }
</script>

<div class="min-h-screen bg-[color:var(--tone-cream)]">
  <div class="mx-auto max-w-5xl px-6">
    <div class="flex flex-col gap-12 py-10">
      {#if recipe}
        <section class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl outline outline-2 outline-[color:var(--tone-border)]">
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
                Sealed {new Date(recipe.created_at).toLocaleDateString()}
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
                  <div class="rounded-2xl border border-[color:var(--tone-border)] bg-white px-4 py-3 shadow-sm">
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
          <div class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl outline outline-2 outline-[color:var(--tone-border)]">
            <h2 class="font-display text-2xl text-[color:var(--tone-ink)]">
              Vault inventory
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

          <div class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl outline outline-2 outline-[color:var(--tone-border)]">
            <h2 class="font-display text-2xl text-[color:var(--tone-ink)]">
              Release steps
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
        <section class="rounded-3xl border border-white/70 bg-white/80 p-6 shadow-xl outline outline-2 outline-[color:var(--tone-border)]">
          <div class="flex flex-wrap items-start justify-between gap-4">
            <h2 class="font-display text-2xl text-[color:var(--tone-ink)]">
              Vault notes
            </h2>
            <Drawer bind:open={convertOpen}>
              <DrawerTrigger
                class="rounded-full border border-[color:var(--tone-border)] px-4 py-2 text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-ink)] hover:bg-[color:var(--tone-warm-100)]"
              >
                Convert
              </DrawerTrigger>
              <DrawerContent class="border-white/70 bg-white/95 text-[color:var(--tone-ink)]">
                <DrawerHeader>
                  <DrawerTitle class="font-display text-xl text-[color:var(--tone-ink)]">
                    Quick conversion
                  </DrawerTitle>
                  <DrawerDescription class="text-[color:var(--tone-ink-soft)]">
                    Convert common kitchen units without leaving the recipe.
                  </DrawerDescription>
                </DrawerHeader>
                <div class="px-6 pb-6 pt-2">
                  <div class="grid gap-4 sm:grid-cols-[1.2fr_0.8fr]">
                    <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
                      Amount
                      <Input
                        type="number"
                        step="0.01"
                        placeholder="0"
                        bind:value={convertValue}
                        class="h-11 border-[color:var(--tone-border-soft)] bg-white text-[color:var(--tone-ink)] focus-visible:border-[color:var(--tone-gold)] focus-visible:ring-[color:var(--tone-gold)]/20"
                      />
                    </label>
                    <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
                      From
                      <select
                        class="h-11 rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 text-sm text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
                        bind:value={convertFrom}
                      >
                        {#each Object.entries(unitLabels) as [value, label]}
                          <option value={value}>{label}</option>
                        {/each}
                      </select>
                    </label>
                  </div>
                  <label class="mt-4 grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
                    To
                    <select
                      class="h-11 rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 text-sm text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
                      bind:value={convertTo}
                    >
                      {#each Object.entries(unitLabels) as [value, label]}
                        <option value={value}>{label}</option>
                      {/each}
                    </select>
                  </label>
                  <div class="mt-6 rounded-2xl border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-200)] px-4 py-3 text-sm text-[color:var(--tone-ink)]">
                    {#if convertResult === null}
                      Enter an amount to see the conversion.
                    {:else}
                      {convertAmount} {convertFrom} = {convertResult.toFixed(2)} {convertTo}
                    {/if}
                  </div>
                  <button
                    class="shadow-ink mt-4 h-10 w-fit rounded-full bg-[color:var(--tone-ink)] px-4 text-xs font-semibold uppercase tracking-[0.12em] text-white transition hover:-translate-y-0.5 hover:bg-[color:var(--tone-ink-hover)] disabled:cursor-not-allowed disabled:opacity-70"
                    type="button"
                    on:click={addConversionNote}
                    disabled={notesBusy || convertResult === null}
                  >
                    Add note
                  </button>
                </div>
              </DrawerContent>
            </Drawer>
          </div>

          <form class="mt-4 grid gap-3" on:submit|preventDefault={handleAddNote}>
            <label class="grid gap-2 text-sm font-medium text-[color:var(--tone-ink-muted)]">
              Add a note
              <textarea
                class="min-h-[96px] rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 py-2 text-sm text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
                placeholder="Write something to remember for next time..."
                bind:value={newNote}
              ></textarea>
            </label>
            <button
              class="shadow-ink h-10 w-fit rounded-full bg-[color:var(--tone-ink)] px-4 text-xs font-semibold uppercase tracking-[0.12em] text-white transition hover:-translate-y-0.5 hover:bg-[color:var(--tone-ink-hover)] disabled:cursor-not-allowed disabled:opacity-70"
              type="submit"
              disabled={notesBusy || !newNote.trim()}
            >
              Add note
            </button>
          </form>

          {#if notesError}
            <p class="mt-4 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {notesError}
            </p>
          {/if}

          {#if notes.length === 0}
            <p class="mt-4 rounded-2xl border border-[color:var(--tone-border)] bg-[color:var(--tone-warm-200)] px-4 py-3 text-sm text-[color:var(--tone-ink-soft)]">
              No notes yet.
            </p>
          {:else}
            <div class="mt-6 grid gap-4">
              {#each notes as note}
                <div class="rounded-2xl border border-[color:var(--tone-border)] bg-white p-4 shadow-sm">
                  <div class="flex items-start justify-between gap-4">
                    <p class="text-xs uppercase tracking-[0.2em] text-[color:var(--tone-ink-soft)]">
                      {new Date(note.created_at).toLocaleDateString()}
                    </p>
                    <div class="flex items-center gap-2">
                      {#if editingNoteId === note.id}
                        <button
                          class="text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-ink)] hover:text-[color:var(--tone-ink-hover)]"
                          type="button"
                          on:click={() => saveEdit(note.id)}
                          disabled={notesBusy || !editingBody.trim()}
                        >
                          Save
                        </button>
                        <button
                          class="text-xs font-semibold uppercase tracking-[0.12em] text-[color:var(--tone-ink-soft)] hover:text-[color:var(--tone-ink)]"
                          type="button"
                          on:click={cancelEdit}
                          disabled={notesBusy}
                        >
                          Cancel
                        </button>
                      {:else}
                        <button
                          class="rounded-full p-1 text-[color:var(--tone-ink-soft)] hover:text-[color:var(--tone-ink)]"
                          type="button"
                          on:click={() => startEdit(note)}
                          disabled={notesBusy}
                          aria-label="Edit note"
                        >
                          <PencilIcon class="h-4 w-4" />
                        </button>
                        <button
                          class="rounded-full p-1 text-red-600 hover:text-red-700"
                          type="button"
                          on:click={() => removeNote(note.id)}
                          disabled={notesBusy}
                          aria-label="Delete note"
                        >
                          <Trash2Icon class="h-4 w-4" />
                        </button>
                      {/if}
                    </div>
                  </div>
                  {#if editingNoteId === note.id}
                    <textarea
                      class="mt-3 min-h-[96px] w-full rounded-xl border border-[color:var(--tone-border-soft)] bg-white px-3 py-2 text-sm text-[color:var(--tone-ink)] shadow-sm focus:border-[color:var(--tone-gold)] focus:outline-none"
                      bind:value={editingBody}
                    ></textarea>
                  {:else}
                    <p class="mt-3 whitespace-pre-wrap text-sm text-[color:var(--tone-ink)]">
                      {note.body}
                    </p>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </section>
      {/if}
    </div>
  </div>
</div>
