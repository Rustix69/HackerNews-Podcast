use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use tracing::{info, error};

#[derive(Debug, Serialize, Deserialize)]
struct HNStory {
    id: u32,
    title: Option<String>,
    url: Option<String>,
    text: Option<String>,
    score: Option<u32>,
    by: Option<String>,
    time: Option<u64>,
    descendants: Option<u32>,
    kids: Option<Vec<u32>>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

// HackerNews API client
struct HNClient {
    client: reqwest::Client,
    base_url: String,
}

impl HNClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://hacker-news.firebaseio.com/v0".to_string(),
        }
    }

    async fn get_top_stories(&self) -> Result<Vec<u32>, anyhow::Error> {
        let url = format!("{}/topstories.json", self.base_url);
        let response = self.client.get(&url).send().await?;
        let story_ids: Vec<u32> = response.json().await?;
        Ok(story_ids)
    }

    async fn get_story(&self, id: u32) -> Result<HNStory, anyhow::Error> {
        let url = format!("{}/item/{}.json", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        let story: HNStory = response.json().await?;
        Ok(story)
    }

    async fn get_stories_batch(&self, ids: &[u32]) -> Result<Vec<HNStory>, anyhow::Error> {
        let futures: Vec<_> = ids.iter().map(|&id| self.get_story(id)).collect();
        let results = futures::future::try_join_all(futures).await?;
        Ok(results)
    }
}

// Global client instance
static HN_CLIENT: std::sync::OnceLock<HNClient> = std::sync::OnceLock::new();

fn get_hn_client() -> &'static HNClient {
    HN_CLIENT.get_or_init(|| HNClient::new())
}

// API Handlers
async fn get_top_stories() -> Result<Json<Vec<HNStory>>, (StatusCode, Json<ApiError>)> {
    let client = get_hn_client();
    
    match client.get_top_stories().await {
        Ok(story_ids) => {
            // Get first 50 stories for performance
            let limited_ids = &story_ids[..std::cmp::min(50, story_ids.len())];
            
            match client.get_stories_batch(limited_ids).await {
                Ok(stories) => {
                    // Filter out stories without titles
                    let valid_stories: Vec<HNStory> = stories
                        .into_iter()
                        .filter(|story| story.title.is_some() && !story.title.as_ref().unwrap().is_empty())
                        .collect();
                    
                    info!("Successfully fetched {} top stories", valid_stories.len());
                    Ok(Json(valid_stories))
                }
                Err(e) => {
                    error!("Failed to fetch story details: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiError {
                            error: "Failed to fetch story details".to_string(),
                        }),
                    ))
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch top stories: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Failed to fetch top stories".to_string(),
                }),
            ))
        }
    }
}

async fn get_story_by_id(Path(id): Path<u32>) -> Result<Json<HNStory>, (StatusCode, Json<ApiError>)> {
    let client = get_hn_client();
    
    match client.get_story(id).await {
        Ok(story) => {
            info!("Successfully fetched story {}", id);
            Ok(Json(story))
        }
        Err(e) => {
            error!("Failed to fetch story {}: {}", id, e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: format!("Story {} not found", id),
                }),
            ))
        }
    }
}

async fn health_check() -> Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("status".to_string(), "healthy".to_string());
    response.insert("service".to_string(), "hackernews-backend".to_string());
    Json(response)
}

// Website metadata endpoint
#[derive(Serialize)]
struct WebsiteMetadata {
    url: String,
    title: Option<String>,
    description: Option<String>,
    domain: String,
    favicon: Option<String>,
}

async fn get_website_metadata(Query(params): Query<HashMap<String, String>>) -> Result<Json<WebsiteMetadata>, StatusCode> {
    let url = match params.get("url") {
        Some(url) => url,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Extract domain
    let domain = url
        .split('/')
        .nth(2)
        .unwrap_or("Unknown")
        .to_string();

    // Create client with timeout and user agent
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch the website HTML
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        // Return basic metadata if we can't fetch the page
        return Ok(Json(WebsiteMetadata {
            url: url.clone(),
            title: None,
            description: None,
            domain,
            favicon: None,
        }));
    }

    let html = response
        .text()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse basic metadata from HTML
    let title = extract_html_tag(&html, "title");
    let description = extract_meta_content(&html, "description")
        .or_else(|| extract_meta_property(&html, "og:description"));
    
    // Try to get favicon
    let favicon = extract_favicon(&html, &domain);

    Ok(Json(WebsiteMetadata {
        url: url.clone(),
        title,
        description,
        domain,
        favicon,
    }))
}

fn extract_html_tag(html: &str, tag: &str) -> Option<String> {
    let pattern = format!("<{}[^>]*>([^<]*)</{}>", tag, tag);
    let re = regex::Regex::new(&pattern).ok()?;
    re.captures(html)?.get(1)?.as_str().trim().to_string().into()
}

fn extract_meta_content(html: &str, name: &str) -> Option<String> {
    let pattern = format!(r#"<meta[^>]*name=["']{}["'][^>]*content=["']([^"']*)["'][^>]*>"#, name);
    let re = regex::Regex::new(&pattern).ok()?;
    re.captures(html)?.get(1)?.as_str().to_string().into()
}

fn extract_meta_property(html: &str, property: &str) -> Option<String> {
    let pattern = format!(r#"<meta[^>]*property=["']{}["'][^>]*content=["']([^"']*)["'][^>]*>"#, property);
    let re = regex::Regex::new(&pattern).ok()?;
    re.captures(html)?.get(1)?.as_str().to_string().into()
}

fn extract_favicon(html: &str, domain: &str) -> Option<String> {
    // Look for favicon in link tags
    let patterns = [
        r#"<link[^>]*rel=["']icon["'][^>]*href=["']([^"']*)["'][^>]*>"#,
        r#"<link[^>]*rel=["']shortcut icon["'][^>]*href=["']([^"']*)["'][^>]*>"#,
    ];
    
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(captures) = re.captures(html) {
                if let Some(href) = captures.get(1) {
                    let favicon_url = href.as_str();
                    if favicon_url.starts_with("http") {
                        return Some(favicon_url.to_string());
                    } else if favicon_url.starts_with("/") {
                        return Some(format!("https://{}{}", domain, favicon_url));
                    }
                }
            }
        }
    }
    
    // Default favicon location
    Some(format!("https://{}/favicon.ico", domain))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/stories", get(get_top_stories))
        .route("/api/stories/:id", get(get_story_by_id))
        .route("/api/metadata", get(get_website_metadata))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    info!("Starting HackerNews backend server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
