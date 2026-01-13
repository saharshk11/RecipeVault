use wasm_bindgen::prelude::*;
use url::Url;

#[wasm_bindgen]
pub fn extract_recipe_json(html: &str, base_url: &str) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    let url = Url::parse(base_url).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let recipe = recipe_core::extract_recipe(html, &url)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_json::to_string(&recipe).map_err(|e| JsValue::from_str(&e.to_string()))
}

