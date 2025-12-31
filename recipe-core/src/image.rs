use scraper::{Html, Selector};
use url::Url;

pub fn extract_image_url(html: &str, page_url: &Url) -> Option<Url> {
    let doc = Html::parse_document(html);

    // og:image
    if let Some(u) = meta_content(&doc, r#"meta[property="og:image"]"#, "content", page_url) {
        return Some(u);
    }

    // twitter:image
    if let Some(u) = meta_content(&doc, r#"meta[name="twitter:image"]"#, "content", page_url) {
        return Some(u);
    }

    None
}

fn meta_content(doc: &Html, selector: &str, attr: &str, base: &Url) -> Option<Url> {
    let sel = Selector::parse(selector).ok()?;
    let el = doc.select(&sel).next()?;
    let raw = el.value().attr(attr)?.trim();
    if raw.is_empty() {
        return None;
    }

    // Normalize relative -> absolute
    base.join(raw).ok().or_else(|| Url::parse(raw).ok())
}