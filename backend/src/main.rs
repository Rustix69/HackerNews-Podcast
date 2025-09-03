use axum::{
    extract::{Json, Query, Path},
    http::StatusCode,
    response::{Json as AxumJson, Sse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use tracing::{info, error};
use std::env;
use axum::response::sse::{Event, KeepAlive};

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

#[derive(Debug, Serialize, Deserialize)]
struct HNComment {
    id: u32,
    by: Option<String>,
    time: Option<u64>,
    text: Option<String>,
    kids: Option<Vec<u32>>,
    parent: Option<u32>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

#[derive(Debug, Deserialize)]
struct ContentGenerationRequest {
    story_id: u32,
    comments: Vec<String>,
}

// Update the ContentGenerationResponse struct to match the new implementation
#[derive(Debug, Serialize, Deserialize)]
struct ContentGenerationResponse {
    message: String,
    context_added: bool,
    story_id: u32,
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

    async fn get_comment(&self, id: u32) -> Result<HNComment, anyhow::Error> {
        let url = format!("{}/item/{}.json", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        let comment: HNComment = response.json().await?;
        Ok(comment)
    }

    async fn get_stories_batch(&self, ids: &[u32]) -> Result<Vec<HNStory>, anyhow::Error> {
        let futures: Vec<_> = ids.iter().map(|&id| self.get_story(id)).collect();
        let results = futures::future::try_join_all(futures).await?;
        Ok(results)
    }

    async fn get_comments_for_story(&self, story: &HNStory) -> Result<Vec<HNComment>, anyhow::Error> {
        if let Some(kids) = &story.kids {
            // Fetch ALL comments without limit
            let comment_futures: Vec<_> = kids.iter().map(|&id| self.get_comment(id)).collect();
            let comments = futures::future::try_join_all(comment_futures).await?;
            Ok(comments.into_iter().filter(|c| c.text.is_some()).collect())
        } else {
            Ok(vec![])
        }
    }
}

// Global client instances
static HN_CLIENT: std::sync::OnceLock<HNClient> = std::sync::OnceLock::new();

fn get_hn_client() -> &'static HNClient {
    HN_CLIENT.get_or_init(|| HNClient::new())
}

// API Handlers
async fn get_top_stories() -> Result<AxumJson<Vec<HNStory>>, (StatusCode, AxumJson<ApiError>)> {
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
                    Ok(AxumJson(valid_stories))
                }
                Err(e) => {
                    error!("Failed to fetch story details: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AxumJson(ApiError {
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
                AxumJson(ApiError {
                    error: "Failed to fetch top stories".to_string(),
                }),
            ))
        }
    }
}

async fn get_story_by_id(Path(id): Path<u32>) -> Result<AxumJson<HNStory>, (StatusCode, AxumJson<ApiError>)> {
    let client = get_hn_client();
    
    match client.get_story(id).await {
        Ok(story) => {
            info!("Successfully fetched story {}", id);
            Ok(AxumJson(story))
        }
        Err(e) => {
            error!("Failed to fetch story {}: {}", id, e);
            Err((
                StatusCode::NOT_FOUND,
                AxumJson(ApiError {
                    error: format!("Story {} not found", id),
                }),
            ))
        }
    }
}

async fn get_story_comments(Path(id): Path<u32>) -> Result<AxumJson<Vec<HNComment>>, (StatusCode, AxumJson<ApiError>)> {
    let client = get_hn_client();
    
    match client.get_story(id).await {
        Ok(story) => {
            match client.get_comments_for_story(&story).await {
                Ok(comments) => {
                    info!("Successfully fetched {} comments for story {}", comments.len(), id);
                    Ok(AxumJson(comments))
                }
                Err(e) => {
                    error!("Failed to fetch comments for story {}: {}", id, e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AxumJson(ApiError {
                            error: format!("Failed to fetch comments for story {}", id),
                        }),
                    ))
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch story {}: {}", id, e);
            Err((
                StatusCode::NOT_FOUND,
                AxumJson(ApiError {
                    error: format!("Story {} not found", id),
                }),
            ))
        }
    }
}

async fn generate_content(
    Json(payload): Json<ContentGenerationRequest>
) -> Result<AxumJson<ContentGenerationResponse>, (StatusCode, AxumJson<ApiError>)> {
    let story_id = payload.story_id;
    let comments: Vec<String> = payload.comments
        .into_iter()
        .filter_map(|comment| Some(comment))
        .filter(|c: &String| !c.is_empty())
        .collect();

    if comments.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST, 
            AxumJson(ApiError { error: "No comments provided".to_string() })
        ));
    }

    // Combine all comments into a single text
    let combined_comments = comments.join("\n\n---\n\n");
    let source = format!("HackerNews Story #{} Comments", story_id);

    // Send to Alchemyst AI context add endpoint
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/context/add", env::var("ALCHEMYST_API_URL").unwrap_or_else(|_| "https://platform-backend.getalchemystai.com".to_string())))
        .header("Authorization", format!("Bearer {}", env::var("ALCHEMYST_API_KEY").unwrap_or_default()))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "documents": [{
                "content": combined_comments
            }],
            "context_type": "resource",
            "source": source,
            "metadata": {
                "fileName": format!("story_{}_comments_{}.txt", story_id, chrono::Utc::now().timestamp()),
                "fileSize": combined_comments.len(),
                "fileType": "text/plain",
                "lastModified": chrono::Utc::now().to_rfc3339()
            }
        }))
        .send()
        .await
        .map_err(|e| {
            error!("Context add request failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR, 
                AxumJson(ApiError { error: "Failed to send context add request".to_string() })
            )
        })?;

    // Check response - 500 is acceptable as per user request
    let status = response.status();
    let response_text = response.text().await.map_err(|e| {
        error!("Failed to read response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR, 
            AxumJson(ApiError { error: "Failed to read response".to_string() })
        )
    })?;

    if status.is_success() || status.as_u16() == 500 {
        // Both 200 and 500 are acceptable
        let message = if status.is_success() {
            format!("Successfully added context for story {}. Response: {}", story_id, response_text)
        } else {
            format!("Context add completed for story {} (status: {}). Response: {}", story_id, status, response_text)
        };

        Ok(AxumJson(ContentGenerationResponse {
            message,
            context_added: true,
            story_id
        }))
    } else {
        Err((
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), 
            AxumJson(ApiError { 
                error: format!("Context add failed with status: {}. Response: {}", status, response_text) 
            })
        ))
    }
}

async fn health_check() -> AxumJson<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("status".to_string(), "healthy".to_string());
    response.insert("service".to_string(), "hackernews-backend".to_string());
    
    // Check if Alchemyst AI is configured
    let alchemyst_configured = env::var("ALCHEMYST_API_URL").is_ok() && env::var("ALCHEMYST_API_KEY").is_ok();
    response.insert("alchemyst_ai_configured".to_string(), alchemyst_configured.to_string());
    
    AxumJson(response)
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

async fn get_website_metadata(Query(params): Query<HashMap<String, String>>) -> Result<AxumJson<WebsiteMetadata>, StatusCode> {
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
        return Ok(AxumJson(WebsiteMetadata {
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

    Ok(AxumJson(WebsiteMetadata {
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

// --- New: Podcast generation endpoint ---
#[derive(Debug, Deserialize)]
struct PodcastGenerationRequest {
    persona: Option<String>,
    scope: Option<String>,
    title: Option<String>,
}

async fn generate_podcast(
    Json(payload): Json<PodcastGenerationRequest>
) -> Result<(StatusCode, AxumJson<serde_json::Value>), (StatusCode, AxumJson<ApiError>)> {
    let api_url = env::var("ALCHEMYST_API_URL").unwrap_or_else(|_| "https://platform-backend.getalchemystai.com".to_string());
    let api_key = env::var("ALCHEMYST_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(ApiError { error: "ALCHEMYST_API_KEY is not configured".to_string() })
        ));
    }

    let persona = payload.persona.unwrap_or_else(|| "maya".to_string());
    let scope = payload.scope.unwrap_or_else(|| "internal".to_string());
    let title = payload.title.unwrap_or_else(|| "HackerNews Podcast".to_string());

    // Read podcast prompt as system message
    let system_prompt: &str = include_str!("prompt.md");

    let chat_history = vec![
        serde_json::json!({ "role": "system", "content": system_prompt }),
        serde_json::json!({ "role": "user", "content": format!(
            "Generate a podcast episode titled \"{}\" using the most relevant available context from my workspace (HackerNews stories, comments, and any added documents). Automatically retrieve context as needed and produce the full script per the instructions.",
            title
        ) }),
    ];

    let body = serde_json::json!({
        "chat_history": chat_history,
        "persona": persona,
        "scope": scope,
        "stream": false,
        "tools": {
            "researchIcps": false
        }
    });

    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/chat/generate", api_url);
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            error!("Podcast generation request failed: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                AxumJson(ApiError { error: "Upstream request failed".to_string() })
            )
        })?;

    let status = resp.status();
    let response_text = resp.text().await.map_err(|e| {
        error!("Failed to read upstream response text: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(ApiError { error: "Failed to read upstream response".to_string() })
        )
    })?;

    info!("Alchemyst response status: {}", status);
    info!("Alchemyst response body: {}", response_text);

    // Try to parse as JSON
    let value: serde_json::Value = match serde_json::from_str(&response_text) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse upstream response as JSON: {}. Raw response: {}", e, response_text);
            // Return the raw response as a string in a JSON wrapper
            return Ok((StatusCode::OK, AxumJson(serde_json::json!({
                "raw_response": response_text,
                "status": status.as_u16(),
                "parse_error": e.to_string()
            }))));
        }
    };

    if status.is_success() || status.as_u16() == 201 {
        // Platform API returns: { result: { response: { content: "..." } }, chatId, title, researchMode, stream }
        if let Some(result) = value.get("result") {
            if let Some(response_content) = result.get("response") {
                if let Some(content) = response_content.get("content") {
                    return Ok((StatusCode::OK, AxumJson(serde_json::json!({
                        "podcast_script": content,
                        "title": value.get("title").unwrap_or(&serde_json::Value::String(title.clone())),
                        "chat_id": value.get("chatId"),
                        "research_mode": value.get("researchMode"),
                        "status": "success"
                    }))));
                }
            }
            // Handle case where result.response is the content directly
            else if let Some(content) = result.get("content") {
                return Ok((StatusCode::OK, AxumJson(serde_json::json!({
                    "podcast_script": content,
                    "title": value.get("title").unwrap_or(&serde_json::Value::String(title.clone())),
                    "chat_id": value.get("chatId"),
                    "research_mode": value.get("researchMode"),
                    "status": "success"
                }))));
            }
        }
        // Fallback: return the full response for debugging
        Ok((StatusCode::OK, AxumJson(serde_json::json!({
            "raw_platform_response": value,
            "status": "success_but_unexpected_format"
        }))))
    } else {
        error!("Upstream returned error status: {} body: {}", status, value);
        Err((
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            AxumJson(ApiError { error: format!("Podcast generation failed: {}", response_text) })
        ))
    }
}

// New structs for the generate endpoint
#[derive(Debug, Deserialize)]
struct LangChainMessage {
    content: String,
    role: String,
}


#[derive(Debug, Deserialize)]
struct GenerateRequest {
    chat_history: Vec<LangChainMessage>,
    persona: Option<String>,
    scope: Option<String>,

}

#[derive(Debug, Serialize)]
struct StreamingResponse {
    r#type: String,
    content: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

// Generate streaming endpoint
async fn generate_stream(
    Json(payload): Json<GenerateRequest>
) -> Result<Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, AxumJson<ApiError>)> {
    let api_url = env::var("ALCHEMYST_API_URL").unwrap_or_else(|_| "https://platform-backend.getalchemystai.com".to_string());
    let api_key = env::var("ALCHEMYST_API_KEY").unwrap_or_default();
    
    if api_key.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(ApiError { error: "ALCHEMYST_API_KEY is not configured".to_string() })
        ));
    }

    let persona = payload.persona.unwrap_or_else(|| "maya".to_string());
    let scope = payload.scope.unwrap_or_else(|| "internal".to_string());

    // Read podcast prompt as system message
    let system_prompt: &str = include_str!("prompt.md");

    // Convert user messages and add system prompt for podcast generation
    let mut chat_history: Vec<serde_json::Value> = vec![
        serde_json::json!({ "role": "system", "content": system_prompt })
    ];
    
    // Add user messages but modify them to focus on podcast generation
    for msg in payload.chat_history {
        if msg.role == "user" {
            let podcast_content = format!(
                "Generate a podcast episode using the available context from my workspace. User request: {}. Automatically retrieve relevant context and produce the full podcast script per the instructions.",
                msg.content
            );
            chat_history.push(serde_json::json!({
                "content": podcast_content,
                "role": "user"
            }));
        } else {
            chat_history.push(serde_json::json!({
                "content": msg.content,
                "role": msg.role
            }));
        }
    }

    let body = serde_json::json!({
        "chat_history": chat_history,
        "persona": persona,
        "scope": scope,
        "stream": true,
        "tools": {
            "researchIcps": false,
            "deepResearch": false,
            "webSearch": true,
            "organizationSearch": true
        }
    });

    info!("Sending request to Alchemyst API: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/chat/generate/stream", api_url);
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            error!("Generate stream request failed: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                AxumJson(ApiError { error: "Upstream request failed".to_string() })
            )
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("Upstream API returned error: {} - {}", status, error_text);
        return Err((
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            AxumJson(ApiError { error: format!("Upstream API error: {}", error_text) })
        ));
    }

    // Get the response text and process it
    let response_text = response.text().await.map_err(|e| {
        error!("Failed to read response text: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AxumJson(ApiError { error: "Failed to read response".to_string() })
        )
    })?;

    info!("Received response from Alchemyst API: {}", response_text);

    // Create a stream from the response text
    let stream = async_stream::stream! {
        let lines: Vec<&str> = response_text.lines().collect();
        
        for line in lines {
            let line = line.trim();
            
            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }
            
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data.trim() == "[DONE]" {
                    break;
                }
                
                // Try to parse and process the data
                match serde_json::from_str::<serde_json::Value>(data) {
                    Ok(json_data) => {
                        // Extract and display only the content from specific message types
                        if let Some(message_type) = json_data.get("type").and_then(|t| t.as_str()) {
                            match message_type {
                                "thinking_update" => {
                                    if let Some(content) = json_data.get("content") {
                                        let content_str = if content.is_string() {
                                            content.as_str().unwrap_or("").to_string()
                                        } else {
                                            serde_json::to_string(content).unwrap_or_default()
                                        };
                                        info!("🤔 Thinking: {}", content_str);
                                        yield Ok(Event::default().data(serde_json::to_string(&json_data).unwrap_or_default()));
                                    }
                                }
                                "final_response" => {
                                    if let Some(content) = json_data.get("content") {
                                        let content_str = if content.is_string() {
                                            content.as_str().unwrap_or("").to_string()
                                        } else {
                                            serde_json::to_string(content).unwrap_or_default()
                                        };
                                        info!("💬 Response: {}", content_str);
                                        yield Ok(Event::default().data(serde_json::to_string(&json_data).unwrap_or_default()));
                                    }
                                }
                                "metadata" => {
                                    if let Some(content) = json_data.get("content") {
                                        info!("📊 Metadata: {}", serde_json::to_string(content).unwrap_or_default());
                                        yield Ok(Event::default().data(serde_json::to_string(&json_data).unwrap_or_default()));
                                    }
                                }
                                _ => {
                                    // Forward other types as-is but log them
                                    info!("📤 Other message type '{}': {}", message_type, serde_json::to_string(&json_data).unwrap_or_default());
                                    yield Ok(Event::default().data(serde_json::to_string(&json_data).unwrap_or_default()));
                                }
                            }
                        } else {
                            // Forward messages without type as-is
                            yield Ok(Event::default().data(serde_json::to_string(&json_data).unwrap_or_default()));
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON from stream: {} - Data: {}", e, data);
                        // Send error as thinking update
                        let error_response = StreamingResponse {
                            r#type: "thinking_update".to_string(),
                            content: serde_json::json!(format!("Error parsing stream data: {}", e)),
                            icon: None,
                            error: None,
                        };
                        yield Ok(Event::default().data(serde_json::to_string(&error_response).unwrap_or_default()));
                    }
                }
            }
        }
        
        // Send completion signal
        yield Ok(Event::default().data("[DONE]"));
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/stories", get(get_top_stories))
        .route("/api/stories/:id", get(get_story_by_id))
        .route("/api/stories/:id/comments", get(get_story_comments))
        .route("/api/generate-content", post(generate_content))
        .route("/api/metadata", get(get_website_metadata))
        .route("/api/podcast/generate", post(generate_podcast))
        .route("/api/v1/chat/generate/stream", post(generate_stream))
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
