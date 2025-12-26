use recipe_core::extract_recipe;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url_str = std::env::args().nth(1).expect("usage: cli <url>");
    let url = Url::parse(&url_str)?;

    let html = reqwest::get(url.clone()).await?.text().await?;
    let recipe = extract_recipe(&html, &url)?;

    println!("{}", serde_json::to_string_pretty(&recipe)?);
    Ok(())
}
