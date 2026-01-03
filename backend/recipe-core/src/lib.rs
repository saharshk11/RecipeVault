mod model;
mod error;
mod jsonld;
mod html_fallback;
mod image;

pub use model::Recipe;
pub use error::RecipeError;

use url::Url;
use html_escape::decode_html_entities;

pub fn extract_recipe(html: &str, base_url: &Url) -> Result<Recipe, RecipeError> {
    let mut recipe = if let Some(r) = jsonld::try_extract_recipe_from_jsonld(html, base_url)? {
        r
    } else if let Some(r) = html_fallback::try_extract_recipe_from_html(html, base_url)? {
        r
    } else {
        return Err(RecipeError::NotFound);
    };

    // pull image URL from meta tags if missing
    if recipe.image_url.is_none() {
        recipe.image_url = image::extract_image_url(html, base_url);
    }

    normalize_recipe_text(&mut recipe);

    Ok(recipe)
}

fn normalize_recipe_text(recipe: &mut Recipe) {
    recipe.title = decode_html_entities(&recipe.title).to_string();
    recipe.description = recipe
        .description
        .as_ref()
        .map(|value| decode_html_entities(value).to_string());
    recipe.ingredients = recipe
        .ingredients
        .iter()
        .map(|value| decode_html_entities(value).to_string())
        .collect();
    recipe.instructions = recipe
        .instructions
        .iter()
        .map(|value| decode_html_entities(value).to_string())
        .collect();
    recipe.servings = recipe
        .servings
        .as_ref()
        .map(|value| decode_html_entities(value).to_string());
    recipe.prep_time = recipe
        .prep_time
        .as_ref()
        .map(|value| decode_html_entities(value).to_string());
    recipe.cook_time = recipe
        .cook_time
        .as_ref()
        .map(|value| decode_html_entities(value).to_string());
    recipe.total_time = recipe
        .total_time
        .as_ref()
        .map(|value| decode_html_entities(value).to_string());
    recipe.tags = recipe
        .tags
        .iter()
        .map(|value| decode_html_entities(value).to_string())
        .collect();
}
