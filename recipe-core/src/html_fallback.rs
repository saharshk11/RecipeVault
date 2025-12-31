use crate::{Recipe, RecipeError};
use scraper::{ElementRef, Html, Selector};
use url::Url;

pub fn try_extract_recipe_from_html(
    html: &str,
    page_url: &Url,
) -> Result<Option<Recipe>, RecipeError> {
    let doc = Html::parse_document(html);

    // --- Title heuristic: first <h1> ---
    let h1_selector = Selector::parse("h1")
        .map_err(|e| RecipeError::Html(e.to_string()))?;
    let mut title = doc
        .select(&h1_selector)
        .next()
        .map(text_of)
        .unwrap_or_default();

    if title.is_empty() {
        // Fallback: use the URL as a last resort
        title = page_url.as_str().to_string();
    }

    // --- Ingredients heuristic ---
    //
    // Common patterns:
    // - <ul class="ingredients"><li>...</li></ul>
    // - <div class="recipe-ingredients"><li>...</li></div>
    // - any parent whose class contains "ingredient"
    let ingredients_selector = Selector::parse(
        r#"
        ul.ingredients li,
        ol.ingredients li,
        [class*="ingredient"] li
        "#,
    )
    .map_err(|e| RecipeError::Html(e.to_string()))?;

    let ingredients: Vec<String> = doc
        .select(&ingredients_selector)
        .map(text_of)
        .filter(|s| !s.is_empty())
        .collect();

    // --- Instructions heuristic ---
    //
    // Common patterns:
    // - <ol class="instructions"><li>Step 1</li>...</ol>
    // - <div class="directions"><li>...</li></div>
    // - any parent whose class contains "instruction" or "direction"
    let instructions_selector = Selector::parse(
        r#"
        ol.instructions li,
        ul.instructions li,
        [class*="instruction"] li,
        [class*="direction"] li
        "#,
    )
    .map_err(|e| RecipeError::Html(e.to_string()))?;

    let instructions: Vec<String> = doc
        .select(&instructions_selector)
        .map(text_of)
        .filter(|s| !s.is_empty())
        .collect();

    // If we didn't find anything meaningful, don't pretend it's a recipe.
    if ingredients.is_empty() || instructions.is_empty() {
        return Ok(None);
    }

    let recipe = Recipe {
        title,
        description: None,
        ingredients,
        instructions,
        servings: None,
        prep_time: None,
        cook_time: None,
        total_time: None,
        image_url: None,
        source_url: Some(page_url.clone()),
        tags: Vec::new(),
    };

    Ok(Some(recipe))
}

/// Helper: collapse all text under an element into a single trimmed string.
fn text_of(element: ElementRef<'_>) -> String {
    element
        .text()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn extracts_recipe_from_simple_html_lists() {
        let html = r#"
        <html>
          <body>
            <h1>Grandma's Cookies</h1>

            <div class="ingredients">
              <ul>
                <li>1 cup sugar</li>
                <li>2 cups flour</li>
              </ul>
            </div>

            <ol class="instructions">
              <li>Mix all ingredients.</li>
              <li>Bake until golden.</li>
            </ol>
          </body>
        </html>
        "#;

        let url = Url::parse("https://example.com/grandma-cookies").unwrap();
        let result = try_extract_recipe_from_html(html, &url)
            .expect("fallback extraction should not error");

        let recipe = result.expect("expected Some(recipe), got None");

        assert_eq!(recipe.title, "Grandma's Cookies");
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.ingredients[0], "1 cup sugar");
        assert_eq!(recipe.ingredients[1], "2 cups flour");

        assert_eq!(recipe.instructions.len(), 2);
        assert_eq!(recipe.instructions[0], "Mix all ingredients.");
        assert_eq!(recipe.instructions[1], "Bake until golden.");

        assert_eq!(
            recipe.source_url.as_ref().unwrap().as_str(),
            "https://example.com/grandma-cookies"
        );
    }
}