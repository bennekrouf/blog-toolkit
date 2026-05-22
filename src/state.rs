use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Post {
    pub filename: String,
    pub title: String,
    pub description: String,
    pub date: String,
    pub lang: String,
    pub status: String,
    pub body: String,
    pub full_content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectPosts {
    pub queue_fr: Vec<Post>,
    pub queue_en: Vec<Post>,
    pub published_fr: Vec<Post>,
    pub published_en: Vec<Post>,
}

impl ProjectPosts {
    pub fn empty() -> Self {
        Self { queue_fr: vec![], queue_en: vec![], published_fr: vec![], published_en: vec![] }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum LlmProvider {
    #[default]
    DeepSeek,
    Claude,
}

impl LlmProvider {
    pub fn label(&self) -> &'static str {
        match self { LlmProvider::DeepSeek => "DeepSeek", LlmProvider::Claude => "Claude" }
    }
    pub fn model(&self) -> &'static str {
        match self {
            LlmProvider::DeepSeek => "deepseek-chat",
            LlmProvider::Claude   => "claude-sonnet-4-6",
        }
    }
    pub fn api_url(&self) -> &'static str {
        match self {
            LlmProvider::DeepSeek => "https://api.deepseek.com/chat/completions",
            LlmProvider::Claude   => "https://api.anthropic.com/v1/messages",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub project_path: Option<String>,
    pub provider: LlmProvider,
    pub api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { project_path: None, provider: LlmProvider::default(), api_key: None }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "mayorana", "blog-manager")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
        if let Ok(json) = serde_json::to_string_pretty(self) { let _ = std::fs::write(path, json); }
    }

    pub fn resolved_api_key(&self) -> String {
        if let Some(k) = &self.api_key {
            if !k.is_empty() { return k.clone(); }
        }
        // fallback to env
        match self.provider {
            LlmProvider::DeepSeek => std::env::var("DEEPSEEK_API_KEY").unwrap_or_default(),
            LlmProvider::Claude   => std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
        }
    }
}

pub fn load_posts(project_path: &str) -> ProjectPosts {
    let root = PathBuf::from(project_path).join("content");
    ProjectPosts {
        queue_fr: read_posts(&root.join("fr").join("queue")),
        queue_en: read_posts(&root.join("en").join("queue")),
        published_fr: read_posts(&root.join("fr").join("blog")),
        published_en: read_posts(&root.join("en").join("blog")),
    }
}

fn read_posts(dir: &PathBuf) -> Vec<Post> {
    let Ok(entries) = std::fs::read_dir(dir) else { return vec![] };
    let mut posts: Vec<Post> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
        .filter_map(|e| parse_post(e.path()))
        .collect();
    posts.sort_by(|a, b| a.filename.cmp(&b.filename));
    posts
}

fn parse_post(path: PathBuf) -> Option<Post> {
    let full_content = std::fs::read_to_string(&path).ok()?;
    let filename = path.file_name()?.to_string_lossy().to_string();

    let (frontmatter, body) = if full_content.starts_with("---") {
        let rest = &full_content[3..];
        if let Some(end) = rest.find("\n---") {
            (rest[..end].to_string(), rest[end + 4..].trim_start_matches('\n').to_string())
        } else {
            (String::new(), full_content.clone())
        }
    } else {
        (String::new(), full_content.clone())
    };

    let title       = extract_field(&frontmatter, "title").unwrap_or_else(|| filename.clone());
    let description = extract_field(&frontmatter, "description").unwrap_or_default();
    let date        = extract_field(&frontmatter, "date").unwrap_or_default();
    let lang        = extract_field(&frontmatter, "lang").unwrap_or_else(|| "fr".to_string());
    let status      = extract_field(&frontmatter, "status").unwrap_or_else(|| "draft".to_string());

    Some(Post { filename, title, description, date, lang, status, body, full_content })
}

fn extract_field(frontmatter: &str, key: &str) -> Option<String> {
    for line in frontmatter.lines() {
        if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}
