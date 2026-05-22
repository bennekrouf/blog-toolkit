use crate::api;
use crate::state::{AppConfig, LlmProvider, Post, ProjectPosts, load_posts};
use dioxus::prelude::*;
use pulldown_cmark::{Parser, Options, html};

// ── Root ─────────────────────────────────────────────────────────────────────

#[component]
pub fn App() -> Element {
    let config = use_signal(|| AppConfig::load());
    let posts = use_signal(|| ProjectPosts::empty());
    let selected = use_signal(|| None::<Post>);
    let editor_mode = use_signal(|| false);
    let show_settings = use_signal(|| false);

    let has_project = config.read().project_path.is_some();

    rsx! {
        style { {include_str!("../assets/main.css")} }
        if has_project {
            MainLayout { config, posts, selected, editor_mode, show_settings }
        } else {
            SetupScreen { config, posts }
        }
    }
}

// ── Setup screen ──────────────────────────────────────────────────────────────

#[component]
fn SetupScreen(config: Signal<AppConfig>, posts: Signal<ProjectPosts>) -> Element {
    rsx! {
        div { class: "setup-screen",
            div { class: "setup-card",
                h1 { "Blog Manager" }
                p { class: "setup-subtitle",
                    "Select your blog project folder. You can configure the LLM provider afterwards."
                }
                p { class: "setup-hint",
                    "Expected structure: "
                    code { "content/{{fr,en}}/{{queue,blog}}/*.md" }
                }
                button {
                    class: "btn-primary",
                    onclick: move |_| {
                        spawn(async move {
                            if let Some(path) = rfd::AsyncFileDialog::new()
                                .set_title("Select blog project folder")
                                .pick_folder()
                                .await
                            {
                                let path_str = path.path().to_string_lossy().to_string();
                                *posts.write() = load_posts(&path_str);
                                let mut cfg = config.read().clone();
                                cfg.project_path = Some(path_str);
                                cfg.save();
                                *config.write() = cfg;
                            }
                        });
                    },
                    "Open Project Folder"
                }
            }
        }
    }
}

// ── Main layout ───────────────────────────────────────────────────────────────

#[component]
fn MainLayout(
    config: Signal<AppConfig>,
    posts: Signal<ProjectPosts>,
    selected: Signal<Option<Post>>,
    editor_mode: Signal<bool>,
    show_settings: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "layout",
            div { class: "topbar",
                span { class: "topbar-title", "Blog Manager" }
                span { class: "topbar-path",
                    { config.read().project_path.as_deref().unwrap_or("") }
                }
                // provider badge
                span { class: "provider-badge",
                    { config.read().provider.label() }
                }
                button {
                    class: "btn-small",
                    onclick: move |_| {
                        let p = config.read().project_path.clone().unwrap_or_default();
                        *posts.write() = load_posts(&p);
                    },
                    "↻"
                }
                button {
                    class: "btn-small btn-danger",
                    onclick: move |_| {
                        let mut cfg = config.write();
                        cfg.project_path = None;
                        cfg.save();
                    },
                    "Change folder"
                }
                // gear icon button
                button {
                    class: "btn-icon",
                    title: "Settings",
                    onclick: move |_| {
                        let v = *show_settings.read();
                        *show_settings.write() = !v;
                    },
                    "⚙"
                }
            }

            div { class: "panels",
                PostList { config, posts, selected }
                CenterPanel { config, posts, selected, editor_mode }
                PreviewPanel { selected }
            }

            // Settings modal overlay
            if *show_settings.read() {
                SettingsModal { config, show_settings }
            }
        }
    }
}

// ── Settings modal ────────────────────────────────────────────────────────────

#[component]
fn SettingsModal(config: Signal<AppConfig>, show_settings: Signal<bool>) -> Element {
    let cfg = config.read().clone();

    let mut selected_provider = use_signal(|| cfg.provider.clone());
    let mut api_key_input = use_signal(|| cfg.api_key.clone().unwrap_or_default());
    let mut show_key = use_signal(|| false);
    let mut saved_msg = use_signal(|| String::new());

    rsx! {
        // backdrop
        div {
            class: "modal-backdrop",
            onclick: move |_| *show_settings.write() = false,
        }
        div { class: "modal",
            div { class: "modal-header",
                h2 { "Settings" }
                button {
                    class: "btn-icon",
                    onclick: move |_| *show_settings.write() = false,
                    "✕"
                }
            }

            div { class: "modal-body",
                // LLM provider
                div { class: "settings-field",
                    label { "LLM Provider" }
                    div { class: "provider-selector",
                        button {
                            class: if *selected_provider.read() == LlmProvider::DeepSeek
                                { "provider-btn active" } else { "provider-btn" },
                            onclick: move |_| *selected_provider.write() = LlmProvider::DeepSeek,
                            div { class: "provider-btn-name", "DeepSeek" }
                            div { class: "provider-btn-model", "deepseek-chat" }
                        }
                        button {
                            class: if *selected_provider.read() == LlmProvider::Claude
                                { "provider-btn active" } else { "provider-btn" },
                            onclick: move |_| *selected_provider.write() = LlmProvider::Claude,
                            div { class: "provider-btn-name", "Claude" }
                            div { class: "provider-btn-model", "claude-sonnet-4-6" }
                        }
                    }
                }

                // API key
                div { class: "settings-field",
                    label {
                        {match *selected_provider.read() {
                            LlmProvider::DeepSeek => "DeepSeek API Key",
                            LlmProvider::Claude   => "Anthropic API Key",
                        }}
                    }
                    div { class: "key-input-row",
                        input {
                            r#type: if *show_key.read() { "text" } else { "password" },
                            placeholder: match *selected_provider.read() {
                                LlmProvider::DeepSeek => "sk-...",
                                LlmProvider::Claude   => "sk-ant-...",
                            },
                            value: "{api_key_input}",
                            oninput: move |e| *api_key_input.write() = e.value(),
                        }
                        button {
                            class: "btn-icon btn-toggle-key",
                            title: if *show_key.read() { "Hide" } else { "Show" },
                            onclick: move |_| {
                                let v = *show_key.read();
                                *show_key.write() = !v;
                            },
                            if *show_key.read() { "🙈" } else { "👁" }
                        }
                    }
                    p { class: "settings-hint",
                        "Leave empty to use the "
                        code {
                            {match *selected_provider.read() {
                                LlmProvider::DeepSeek => "DEEPSEEK_API_KEY",
                                LlmProvider::Claude   => "ANTHROPIC_API_KEY",
                            }}
                        }
                        " environment variable."
                    }
                }

                if !saved_msg.read().is_empty() {
                    p { class: "settings-saved", "{saved_msg}" }
                }
            }

            div { class: "modal-footer",
                button {
                    class: "btn-primary",
                    onclick: move |_| {
                        let mut cfg = config.write();
                        cfg.provider = selected_provider.read().clone();
                        let key = api_key_input.read().trim().to_string();
                        cfg.api_key = if key.is_empty() { None } else { Some(key) };
                        cfg.save();
                        drop(cfg);
                        *saved_msg.write() = "Saved ✓".to_string();
                    },
                    "Save"
                }
            }
        }
    }
}

// ── Left panel ────────────────────────────────────────────────────────────────

#[component]
fn PostList(
    config: Signal<AppConfig>,
    posts: Signal<ProjectPosts>,
    selected: Signal<Option<Post>>,
) -> Element {
    let mut active_lang = use_signal(|| "fr".to_string());

    let p = posts.read();
    let lang = active_lang.read().clone();
    let (queue, published) = if lang == "fr" {
        (p.queue_fr.clone(), p.published_fr.clone())
    } else {
        (p.queue_en.clone(), p.published_en.clone())
    };
    drop(p);

    rsx! {
        div { class: "panel panel-left",
            div { class: "lang-tabs",
                button {
                    class: if active_lang.read().as_str() == "fr" { "tab active" } else { "tab" },
                    onclick: move |_| *active_lang.write() = "fr".to_string(),
                    "FR"
                }
                button {
                    class: if active_lang.read().as_str() == "en" { "tab active" } else { "tab" },
                    onclick: move |_| *active_lang.write() = "en".to_string(),
                    "EN"
                }
            }

            div { class: "section-label", "Queue ({queue.len()})" }
            for post in queue {
                PostItem {
                    post: post.clone(),
                    is_selected: selected.read().as_ref().map(|s| s.filename == post.filename).unwrap_or(false),
                    config, posts, selected,
                    active_lang,
                    is_queue: true,
                }
            }

            div { class: "section-label section-label-published", "Published ({published.len()})" }
            for post in published {
                PostItem {
                    post: post.clone(),
                    is_selected: selected.read().as_ref().map(|s| s.filename == post.filename).unwrap_or(false),
                    config, posts, selected,
                    active_lang,
                    is_queue: false,
                }
            }
        }
    }
}

#[component]
fn PostItem(
    post: Post,
    is_selected: bool,
    config: Signal<AppConfig>,
    posts: Signal<ProjectPosts>,
    selected: Signal<Option<Post>>,
    active_lang: Signal<String>,
    is_queue: bool,
) -> Element {
    let post_click = post.clone();
    let post_pub   = post.clone();
    let post_del   = post.clone();
    let lang_pub   = active_lang.read().clone();
    let lang_del   = lang_pub.clone();

    rsx! {
        div {
            class: if is_selected { "post-item selected" } else { "post-item" },
            onclick: move |_| *selected.write() = Some(post_click.clone()),

            div { class: "post-item-title", "{post.title}" }
            div { class: "post-item-date",  "{post.date}" }

            if is_queue {
                div { class: "post-item-actions",
                    button {
                        class: "btn-micro btn-publish",
                        onclick: move |e| {
                            e.stop_propagation();
                            let project = config.read().project_path.clone().unwrap_or_default();
                            if api::publish_post(&project, &lang_pub, &post_pub.filename).is_ok() {
                                *posts.write() = load_posts(&project);
                            }
                        },
                        "Publish"
                    }
                    button {
                        class: "btn-micro btn-delete",
                        onclick: move |e| {
                            e.stop_propagation();
                            let project = config.read().project_path.clone().unwrap_or_default();
                            if api::delete_queued(&project, &lang_del, &post_del.filename).is_ok() {
                                *posts.write() = load_posts(&project);
                            }
                        },
                        "✕"
                    }
                }
            }
        }
    }
}

// ── Center panel ──────────────────────────────────────────────────────────────

#[component]
fn CenterPanel(
    config: Signal<AppConfig>,
    posts: Signal<ProjectPosts>,
    selected: Signal<Option<Post>>,
    editor_mode: Signal<bool>,
) -> Element {
    let mut title          = use_signal(|| String::new());
    let mut summary        = use_signal(|| String::new());
    let mut lang           = use_signal(|| "fr".to_string());
    let mut generating     = use_signal(|| false);
    let mut status_msg     = use_signal(|| String::new());
    let mut editor_content = use_signal(|| String::new());

    use_effect(move || {
        if let Some(post) = selected.read().clone() {
            *title.write()          = post.title.clone();
            *summary.write()        = post.description.clone();
            *lang.write()           = post.lang.clone();
            *editor_content.write() = post.full_content.clone();
            *editor_mode.write()    = true;
        }
    });

    rsx! {
        div { class: "panel panel-center",
            div { class: "mode-tabs",
                button {
                    class: if !*editor_mode.read() { "tab active" } else { "tab" },
                    onclick: move |_| { *editor_mode.write() = false; *selected.write() = None; },
                    "+ New post"
                }
                button {
                    class: if *editor_mode.read() { "tab active" } else { "tab" },
                    disabled: selected.read().is_none(),
                    onclick: move |_| *editor_mode.write() = true,
                    "Edit selected"
                }
            }

            if !*editor_mode.read() {
                div { class: "form",
                    div { class: "field",
                        label { "Title" }
                        input {
                            placeholder: "Votre carrière est-elle un carré dans un rond ?",
                            value: "{title}",
                            oninput: move |e| *title.write() = e.value(),
                        }
                    }
                    div { class: "field",
                        label { "Summary / angle" }
                        textarea {
                            placeholder: "Explorer le concept du P-E Fit...",
                            value: "{summary}",
                            oninput: move |e| *summary.write() = e.value(),
                            rows: "4",
                        }
                    }
                    div { class: "field field-row",
                        label { "Language" }
                        div { class: "lang-tabs",
                            button {
                                class: if lang.read().as_str() == "fr" { "tab active" } else { "tab" },
                                onclick: move |_| *lang.write() = "fr".to_string(),
                                "FR"
                            }
                            button {
                                class: if lang.read().as_str() == "en" { "tab active" } else { "tab" },
                                onclick: move |_| *lang.write() = "en".to_string(),
                                "EN"
                            }
                        }
                    }
                    if !status_msg.read().is_empty() {
                        p { class: "status-msg", "{status_msg}" }
                    }
                    button {
                        class: "btn-primary btn-generate",
                        disabled: *generating.read() || title.read().is_empty() || summary.read().is_empty(),
                        onclick: move |_| {
                            let t = title.read().clone();
                            let s = summary.read().clone();
                            let l = lang.read().clone();
                            let cfg = config.read().clone();
                            let project = cfg.project_path.clone().unwrap_or_default();
                            let provider = cfg.provider.clone();
                            let api_key = cfg.resolved_api_key();

                            if api_key.is_empty() {
                                *status_msg.write() = "No API key configured — open ⚙ Settings.".to_string();
                                return;
                            }

                            spawn(async move {
                                *generating.write() = true;
                                *status_msg.write() = "Generating…".to_string();
                                match api::generate_post(&t, &s, &l, &provider, &api_key).await {
                                    Ok(body) => match api::save_to_queue(&project, &t, &s, &l, &body) {
                                        Ok(path) => {
                                            *status_msg.write() = format!(
                                                "Saved: {}",
                                                path.split('/').last().unwrap_or("")
                                            );
                                            *posts.write() = load_posts(&project);
                                        }
                                        Err(e) => *status_msg.write() = format!("Save error: {e}"),
                                    },
                                    Err(e) => *status_msg.write() = format!("Error: {e}"),
                                }
                                *generating.write() = false;
                            });
                        },
                        if *generating.read() { "Generating…" } else { "Generate & queue" }
                    }
                }
            } else {
                div { class: "editor-wrap",
                    textarea {
                        class: "md-editor",
                        value: "{editor_content}",
                        oninput: move |e| {
                            let val = e.value();
                            *editor_content.write() = val.clone();
                            let updated = selected.read().clone().map(|mut p| {
                                p.full_content = val;
                                p
                            });
                            *selected.write() = updated;
                        },
                        spellcheck: "false",
                    }
                    if !status_msg.read().is_empty() {
                        p { class: "status-msg", "{status_msg}" }
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            if let Some(post) = selected.read().clone() {
                                let project = config.read().project_path.clone().unwrap_or_default();
                                let path = format!("{}/content/{}/queue/{}", project, post.lang, post.filename);
                                match std::fs::write(&path, editor_content.read().as_str()) {
                                    Ok(_) => {
                                        *status_msg.write() = "Saved.".to_string();
                                        *posts.write() = load_posts(&project);
                                    }
                                    Err(e) => *status_msg.write() = format!("Save error: {e}"),
                                }
                            }
                        },
                        "Save changes"
                    }
                }
            }
        }
    }
}

// ── Right panel — preview ──────────────────────────────────────────────────────

#[component]
fn PreviewPanel(selected: Signal<Option<Post>>) -> Element {
    let post = selected.read().clone();
    rsx! {
        div { class: "panel panel-right",
            if let Some(p) = post {
                div { class: "preview-meta",
                    span { class: "preview-lang-badge", "{p.lang.to_uppercase()}" }
                    span { class: "preview-date", "{p.date}" }
                    span {
                        class: if p.status == "published" { "status-badge published" } else { "status-badge draft" },
                        "{p.status}"
                    }
                }
                div {
                    class: "preview-content blog-content",
                    dangerous_inner_html: "{md_to_html(&p.full_content)}",
                }
            } else {
                div { class: "preview-empty",
                    p { "Select a post or generate a new one to preview it here." }
                }
            }
        }
    }
}

fn md_to_html(content: &str) -> String {
    let body = if content.starts_with("---") {
        let rest = &content[3..];
        if let Some(end) = rest.find("\n---") { rest[end + 4..].trim_start_matches('\n') }
        else { content }
    } else { content };

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(body, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}
