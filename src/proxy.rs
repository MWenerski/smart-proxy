// src/proxy.rs
use axum::{http::StatusCode, response::Html, extract::Query};
use reqwest::Client;
use std::collections::HashMap;
use lol_html::{
    HtmlRewriter, Settings, element,
    MemorySettings,
    AsciiCompatibleEncoding,
    errors::RewritingError,
};


pub async fn handle_proxy(
    Query(params): Query<HashMap<String, String>>
) -> Result<Html<String>, StatusCode> {
    // Extract ?url= parameter
    let Some(target_url) = params.get("url") else {
        return Err(StatusCode::BAD_REQUEST);
    };

    // Fetch the target page
    let client = Client::new();
    let res = client
        .get(target_url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let html = res
        .text()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let mut output = Vec::new();

    // HTML Rewriter with ad-stripping and proxying links
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                // Remove all <script> tags
                //element!("script", |el| { el.remove(); Ok(()) }),
                // Remove iframes
                //element!("iframe", |el| { el.remove(); Ok(()) }),
                // Remove elements with IDs/classes containing "ad"
                element!("[class*='ad'], [id*='ad']", |el| { el.remove(); Ok(()) }),
                // Rewrite links to pass through proxy
                element!("a[href]", |el| {
                    if let Some(href) = el.get_attribute("href") {
                        if href.starts_with("http") {
                            el.set_attribute("href", &format!("/proxy?url={}", href)).ok();
                        }
                    }
                    Ok(())
                }),
                // Rewrite images to proxy them too
                element!("img[src]", |el| {
                    if let Some(src) = el.get_attribute("src") {
                        if src.starts_with("http") {
                            el.set_attribute("src", &format!("/proxy?url={}", src)).ok();
                        }
                    }
                    Ok(())
                }),
            ],
            document_content_handlers: vec![],
            encoding: AsciiCompatibleEncoding::utf_8(),
            memory_settings: MemorySettings::default(),
            strict: false,
            enable_esi_tags: false,
            adjust_charset_on_meta_tag: true,
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    // Apply rewriting and handle memory errors
    if let Err(RewritingError::MemoryLimitExceeded(_)) =
        rewriter.write(html.as_bytes()).and_then(|_| rewriter.end())
    {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Html(String::from_utf8_lossy(&output).to_string()))
}
