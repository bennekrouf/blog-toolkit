# Blog Manager

Desktop app for writing, queuing, and publishing blog posts — powered by DeepSeek or Claude.

Built with [Dioxus](https://dioxuslabs.com/) (Rust, runs natively on macOS, Linux, Windows).

---

## Installation

### macOS (Apple Silicon)
```bash
curl -L https://github.com/Bennekrouf/blog-toolkit/releases/latest/download/blog-toolkit-macos-arm64.tar.gz | tar xz
cd blog-toolkit-macos-arm64
./setup-mac.sh
./blog-toolkit
```

### Windows
Download **blog-toolkit-setup.exe** from the [latest release](https://github.com/Bennekrouf/blog-toolkit/releases/latest) and run it. The installer handles Node.js and WebView2 automatically.

### Linux (x86_64)
```bash
curl -L https://github.com/Bennekrouf/blog-toolkit/releases/latest/download/blog-toolkit-linux-x86_64.tar.gz | tar xz
cd blog-toolkit-linux-x86_64
sudo ./setup-linux.sh
./blog-toolkit
```

> **Runtime requirement (Linux):** `libwebkit2gtk-4.1` — installed automatically by `setup-linux.sh` on Debian/Ubuntu, Fedora, and Arch.

---

## Project structure requirement

Blog Manager expects your blog project to follow this folder layout:

```
your-site/
└── content/
    ├── fr/
    │   ├── queue/   ← generated drafts waiting to be published
    │   └── blog/    ← published posts (visible on the site)
    └── en/
        ├── queue/
        └── blog/
```

On first launch you'll be asked to select the project root folder. The app remembers it across sessions.

---

## Features

### Three-panel layout

| Panel | What it does |
|---|---|
| **Left** | Lists queued and published posts per language (FR / EN). Click a post to load it. Publish or delete queued posts directly. |
| **Center** | New post form (title + summary + language → AI generation) or raw markdown editor for the selected post. |
| **Right** | Live rendered markdown preview with status badge (draft / published). |

### AI generation

Fill in a title and a short summary, choose a language, and click **Generate & queue**. The app calls your configured LLM, receives the full post body, and saves it to `content/{lang}/queue/` with `status: "draft"`.

### Queue → publish flow

Posts in the queue are hidden from your live site. When you're ready to publish:
- Click **Publish** next to a queued post in the left panel, or
- Use the weekly auto-publish cron (see below).

Publishing moves the file to `content/{lang}/blog/`, sets `status: "published"`, updates the date to today, and runs `node scripts/generate-blog-data.js` to rebuild the site index.

### Markdown editor

Click a queued post to open it in the center editor. Edit the raw markdown, then **Save changes**. The preview on the right updates as you type.

---

## LLM configuration

Click the **⚙** gear icon in the top bar to open Settings.

| Provider | Model | Key env fallback |
|---|---|---|
| DeepSeek | `deepseek-chat` | `DEEPSEEK_API_KEY` |
| Claude | `claude-sonnet-4-6` | `ANTHROPIC_API_KEY` |

You can paste the key directly in Settings (stored in `~/.config/blog-toolkit/config.json`) or leave the field empty and set the environment variable instead. The app checks the env var automatically if no key is saved.

---

## Weekly auto-publish (macOS)

A launchd agent fires every **Monday at 08:00 local time** and publishes the oldest queued post from each language queue.

```bash
# Load (run once after install)
launchctl load ~/Library/LaunchAgents/com.cvenom.blog-publisher.plist

# Unload (pause)
launchctl unload ~/Library/LaunchAgents/com.cvenom.blog-publisher.plist

# Trigger immediately
bash /path/to/rust-doc-creator/scripts/publish-cvenom-blog.sh

# Check logs
tail -f ~/Library/Logs/cvenom-blog-publish.log
```

---

## Development

### Prerequisites

- Rust (stable) — [rustup.rs](https://rustup.rs)
- Dioxus CLI — `cargo install dioxus-cli`
- Node.js ≥ 18 — for `generate-blog-data.js` when publishing

### Run in dev mode

```bash
cd blog-toolkit
DEEPSEEK_API_KEY=sk-... dx serve --platform desktop
```

### Build release binary

```bash
cargo build --release
# binary: target/release/blog-toolkit
```

### Cut a release

```bash
./scripts/release.sh            # patch bump (0.1.0 → 0.1.1)
./scripts/release.sh --minor    # 0.1.0 → 0.2.0
./scripts/release.sh 1.0.0      # explicit version
./scripts/release.sh --dry-run  # preview, don't execute
```

Pushing the tag triggers GitHub Actions, which builds macOS arm64, Linux x86_64, and the Windows installer, then creates the GitHub Release automatically.

---

## Content file format

Each post is a markdown file with YAML frontmatter:

```markdown
---
title: "Votre carrière est-elle un carré dans un rond ?"
description: "Explorer le concept du P-E Fit..."
date: "2026-05-20"
author: "Cvenom Team"
tags: ["carrière", "développement-professionnel"]
lang: "fr"
status: "draft"
---

# Votre carrière est-elle un carré dans un rond ?

Post body here...
```

`status: "draft"` → invisible on the site.  
`status: "published"` → picked up by `generate-blog-data.js` and shown on the blog.
