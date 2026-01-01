use crate::{Recipe, RecipeError};
use recipe_scraper::{Extract, SchemaOrgEntry, SchemaOrgRecipe, Scrape};
use url::Url;

pub fn try_extract_recipe_from_jsonld(
    html: &str,
    page_url: &Url,
) -> Result<Option<Recipe>, RecipeError> {
    let entries: Vec<SchemaOrgEntry> = SchemaOrgEntry::scrape_html(html);

    let mut found: Vec<SchemaOrgRecipe> = entries.
        into_iter()
        .flat_map(|entry| entry.extract_recipes())
        .collect();
    
    let Some(schema_recipe) = found.pop() else {
        return Ok(None);
    };

    let recipe = map_schema_org_recipe(&schema_recipe, page_url);
    Ok(Some(recipe))
}

fn map_schema_org_recipe(src: &SchemaOrgRecipe, page_url: &Url) -> Recipe {
    let ingredients = src.ingredients().clone().into_iter().collect();

    let instructions = src
        .directions()
        .as_ref()
        .and_then(|list| list.directions())
        .map(|items| {
            items
                .into_iter()
                .map(|inst| inst.to_string())
                .collect()

        })
        .unwrap_or_default();
    
    let prep_time = src
        .prep_time()
        .as_ref()
        .and_then(|d| d.human_readable());

    let cook_time = src
        .cook_time()
        .as_ref()
        .and_then(|d| d.human_readable());

    let total_time = src
        .total_time()
        .as_ref()
        .and_then(|d| d.human_readable());
    
    let servings = src
        .yields()
        .as_ref()
        .map(|y| y.to_string());
    
    let description = {
        let desc = src.description();
        if desc.trim().is_empty() {
            None
        } else {
            Some(desc.clone())
        }
    };

    Recipe {
        title: src.name().clone(),
        description,
        ingredients,
        instructions,
        servings,
        prep_time,
        cook_time,
        total_time,
        image_url: None,
        source_url: Some(page_url.clone()),
        tags: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn extracts_simple_jsonld_recipe() {
        // Minimal but realistic HTML skeleton with JSON-LD.
        let html = r#"
        <!doctype html>
        <html lang="en">
          <head>
            <meta charset="utf-8">
            <title>Test Pancakes</title>
            <script type="application/ld+json">
            {
              "@context": "https://schema.org",
              "@type": "Recipe",
              "name": "Test Pancakes",
              "description": "Fluffy test pancakes for unit testing.",
              "recipeIngredient": [
                "1 cup flour",
                "2 tbsp sugar",
                "1 cup milk"
              ],
              "recipeInstructions": [
                {
                  "@type": "HowToStep",
                  "text": "Mix all ingredients in a bowl."
                },
                {
                  "@type": "HowToStep",
                  "text": "Cook on a hot griddle until golden."
                }
              ],
              "prepTime": "PT10M",
              "cookTime": "PT20M",
              "totalTime": "PT30M",
              "recipeYield": "4 servings"
            }
            </script>
          </head>
          <body>
            <h1>Test Pancakes</h1>
          </body>
        </html>
        "#;

        let url = Url::parse("https://example.com/test-pancakes").unwrap();

        let result = try_extract_recipe_from_jsonld(html, &url)
            .expect("jsonld extraction should not error");

        let recipe = result.expect("expected Some(recipe), got None");

        // Basic field checks
        assert_eq!(recipe.title, "Test Pancakes");
        assert_eq!(
            recipe.description.as_deref(),
            Some("Fluffy test pancakes for unit testing.")
        );

        assert_eq!(recipe.ingredients.len(), 3);
        assert_eq!(recipe.ingredients[0], "1 cup flour");
        assert_eq!(recipe.ingredients[1], "2 tbsp sugar");
        assert_eq!(recipe.ingredients[2], "1 cup milk");

        assert_eq!(recipe.instructions.len(), 2);
        assert_eq!(
            recipe.instructions[0],
            "Mix all ingredients in a bowl."
        );
        assert_eq!(
            recipe.instructions[1],
            "Cook on a hot griddle until golden."
        );

        assert_eq!(recipe.servings.as_deref(), Some("4 servings"));
        assert!(recipe.prep_time.is_some());
        assert!(recipe.cook_time.is_some());
        assert!(recipe.total_time.is_some());


        // We set source_url = Some(page_url) in the mapper
        assert_eq!(
            recipe.source_url.as_ref().unwrap().as_str(),
            "https://example.com/test-pancakes"
        );
    }
}