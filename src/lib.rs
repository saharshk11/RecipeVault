mod model;
mod error;
mod jsonld;
mod html_fallback;
mod image;

pub use model::Recipe;
pub use error::RecipeError;

use url::Url;

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

    Ok(recipe)
}