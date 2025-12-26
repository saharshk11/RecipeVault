mod model;
mod error;
mod jsonld;

pub use model::Recipe;
pub use error::RecipeError;

use url::Url;

pub fn extract_recipe(html: &str, base_url: &Url) -> Result<Recipe, RecipeError> {
    if let Some(recipe) = jsonld::try_extract_recipe_from_jsonld(html, base_url)? {
        return Ok(recipe);
    }

    Err(RecipeError::NotFound)
}