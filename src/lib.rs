mod model;
mod error;
mod jsonld;
mod html_fallback;

pub use model::Recipe;
pub use error::RecipeError;

use url::Url;

pub fn extract_recipe(html: &str, base_url: &Url) -> Result<Recipe, RecipeError> {
    if let Some(recipe) = jsonld::try_extract_recipe_from_jsonld(html, base_url)? {
        return Ok(recipe);
    }

    if let Some(recipe) = html_fallback::try_extract_recipe_from_html(html, base_url)? {
        return Ok(recipe);
    }

    Err(RecipeError::NotFound)
}