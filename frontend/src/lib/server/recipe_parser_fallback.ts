import type { Recipe } from "./recipe_types";

type JsonValue = null | boolean | number | string | JsonValue[] | { [k: string]: JsonValue };

function isObject(value: JsonValue): value is Record<string, JsonValue> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asString(value: JsonValue): string | null {
  return typeof value === "string" ? value : null;
}

function asStringArray(value: JsonValue): string[] {
  if (Array.isArray(value)) return value.map(asString).filter((v): v is string => !!v);
  const s = asString(value);
  return s ? [s] : [];
}

function collectJsonLdNodes(root: JsonValue): Record<string, JsonValue>[] {
  const out: Record<string, JsonValue>[] = [];

  const visit = (node: JsonValue) => {
    if (!node) return;
    if (Array.isArray(node)) {
      for (const item of node) visit(item);
      return;
    }
    if (!isObject(node)) return;

    out.push(node);
    if (node["@graph"]) visit(node["@graph"]);
    for (const v of Object.values(node)) visit(v);
  };

  visit(root);
  return out;
}

function isRecipeType(value: JsonValue) {
  if (typeof value === "string") return value.toLowerCase() === "recipe";
  if (Array.isArray(value)) return value.some(isRecipeType);
  return false;
}

function extractInstructions(value: JsonValue) {
  if (!value) return [];
  if (typeof value === "string") return [value];
  if (Array.isArray(value)) {
    const lines: string[] = [];
    for (const item of value) {
      if (typeof item === "string") {
        lines.push(item);
        continue;
      }
      if (isObject(item)) {
        const text = asString(item.text) ?? asString(item.name);
        if (text) lines.push(text);
      }
    }
    return lines;
  }
  if (isObject(value)) {
    const text = asString(value.text) ?? asString(value.name);
    return text ? [text] : [];
  }
  return [];
}

function firstImageUrl(value: JsonValue): string | null {
  if (!value) return null;
  if (typeof value === "string") return value;
  if (Array.isArray(value)) {
    for (const item of value) {
      const url = firstImageUrl(item);
      if (url) return url;
    }
    return null;
  }
  if (isObject(value)) {
    return asString(value.url) ?? asString(value.contentUrl) ?? null;
  }
  return null;
}

export function extractRecipeFromHtmlFallback(html: string, baseUrl: string): Recipe | null {
  const re = /<script[^>]*type=["']application\/ld\+json["'][^>]*>([\s\S]*?)<\/script>/gi;
  const blocks: string[] = [];
  for (const match of html.matchAll(re)) {
    const raw = match[1]?.trim();
    if (raw) blocks.push(raw);
  }

  for (const block of blocks) {
    let parsed: JsonValue;
    try {
      parsed = JSON.parse(block) as JsonValue;
    } catch {
      continue;
    }

    const nodes = collectJsonLdNodes(parsed);
    const recipeNode = nodes.find((n) => isRecipeType(n["@type"] ?? null));
    if (!recipeNode) continue;

    const title = asString(recipeNode.name) ?? asString(recipeNode.headline) ?? "";
    if (!title) continue;

    const recipe: Recipe = {
      title,
      description: asString(recipeNode.description),
      ingredients: asStringArray(recipeNode.recipeIngredient),
      instructions: extractInstructions(recipeNode.recipeInstructions),
      servings: asString(recipeNode.recipeYield),
      prep_time: asString(recipeNode.prepTime),
      cook_time: asString(recipeNode.cookTime),
      total_time: asString(recipeNode.totalTime),
      image_url: firstImageUrl(recipeNode.image),
      source_url: baseUrl,
      tags: []
    };

    return recipe;
  }

  return null;
}

