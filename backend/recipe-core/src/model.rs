use serde::{Deserialize, Serialize};
use url::Url;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub title: String,
    pub description: Option<String>,
    pub ingredients: Vec<String>,
    pub instructions: Vec<String>,
    pub servings: Option<String>,
    pub prep_time: Option<String>,
    pub cook_time: Option<String>,
    pub total_time: Option<String>,
    pub image_url: Option<Url>,
    pub source_url: Option<Url>,
    pub tags: Vec<String>,
}