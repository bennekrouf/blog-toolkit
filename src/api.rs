use anyhow::{Context, Result};
use crate::state::LlmProvider;

pub async fn generate_post(
    title: &str,
    summary: &str,
    lang: &str,
    provider: &LlmProvider,
    api_key: &str,
) -> Result<String> {
    let lang_instruction = if lang == "fr" { "Write entirely in French." } else { "Write entirely in English." };

    let system = format!(
        "You are a professional blog writer for Cvenom, a career and CV platform. \
        You write engaging, insightful posts about career development, workplace psychology, \
        and professional growth. {lang_instruction} \
        Your tone is empathetic, intelligent, and slightly provocative — like a good career coach. \
        Avoid corporate jargon. Use concrete examples and reference established frameworks \
        (Holland's theory, Person-Environment Fit, etc.) when relevant."
    );
    let user = format!(
        "Write a blog post with this title: \"{title}\"\n\
        Summary / angle: {summary}\n\n\
        Output ONLY the markdown body (no frontmatter). Start directly with the first paragraph. \
        Structure: intro, 3-4 sections with ## headings, a conclusion. Aim for ~600-800 words."
    );

    match provider {
        LlmProvider::DeepSeek => call_deepseek(&system, &user, api_key).await,
        LlmProvider::Claude   => call_claude(&system, &user, api_key).await,
    }
}

async fn call_deepseek(system: &str, user: &str, api_key: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": LlmProvider::DeepSeek.model(),
        "messages": [
            {"role": "system", "content": system},
            {"role": "user",   "content": user}
        ],
        "temperature": 0.75
    });

    let resp = client
        .post(LlmProvider::DeepSeek.api_url())
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await.context("DeepSeek request failed")?;

    if !resp.status().is_success() {
        let s = resp.status();
        let e = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("DeepSeek error {s}: {e}"));
    }

    let json: serde_json::Value = resp.json().await.context("DeepSeek parse failed")?;
    json["choices"][0]["message"]["content"]
        .as_str().map(String::from)
        .context("Empty DeepSeek response")
}

async fn call_claude(system: &str, user: &str, api_key: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": LlmProvider::Claude.model(),
        "max_tokens": 2048,
        "system": system,
        "messages": [{"role": "user", "content": user}]
    });

    let resp = client
        .post(LlmProvider::Claude.api_url())
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await.context("Claude request failed")?;

    if !resp.status().is_success() {
        let s = resp.status();
        let e = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Claude error {s}: {e}"));
    }

    let json: serde_json::Value = resp.json().await.context("Claude parse failed")?;
    json["content"][0]["text"]
        .as_str().map(String::from)
        .context("Empty Claude response")
}

pub fn save_to_queue(project_path: &str, title: &str, summary: &str, lang: &str, body: &str) -> Result<String> {
    let slug = slugify(title);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let queue_dir = format!("{}/content/{}/queue", project_path, lang);
    std::fs::create_dir_all(&queue_dir)?;

    let content = format!(
        "---\ntitle: \"{title}\"\ndescription: \"{summary}\"\ndate: \"{today}\"\n\
        author: \"Cvenom Team\"\ntags: [\"carrière\", \"développement-professionnel\"]\n\
        lang: \"{lang}\"\nstatus: \"draft\"\n---\n\n# {title}\n\n{body}"
    );
    let path = format!("{}/{}.md", queue_dir, slug);
    std::fs::write(&path, content)?;
    Ok(path)
}

pub fn publish_post(project_path: &str, lang: &str, filename: &str) -> Result<()> {
    let src = format!("{}/content/{}/queue/{}", project_path, lang, filename);
    let dst_dir = format!("{}/content/{}/blog", project_path, lang);
    std::fs::create_dir_all(&dst_dir)?;
    let dst = format!("{}/{}", dst_dir, filename);

    let content = std::fs::read_to_string(&src)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let updated = content.replace("status: \"draft\"", "status: \"published\"");
    let updated = if let Some(line) = updated.lines().find(|l| l.starts_with("date:")) {
        updated.replace(line, &format!("date: \"{today}\""))
    } else { updated };

    std::fs::write(&dst, updated)?;
    std::fs::remove_file(&src)?;
    let _ = std::process::Command::new("node")
        .arg("scripts/generate-blog-data.js")
        .current_dir(project_path)
        .status();
    Ok(())
}

pub fn delete_queued(project_path: &str, lang: &str, filename: &str) -> Result<()> {
    std::fs::remove_file(format!("{}/content/{}/queue/{}", project_path, lang, filename))?;
    Ok(())
}

fn slugify(title: &str) -> String {
    title.to_lowercase()
        .chars()
        .map(|c| match c {
            'à'|'â'|'ä' => 'a', 'é'|'è'|'ê'|'ë' => 'e',
            'î'|'ï' => 'i', 'ô'|'ö' => 'o', 'ù'|'û'|'ü' => 'u',
            'ç' => 'c', ' '|'_' => '-', c => c,
        })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-")
}
