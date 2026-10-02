# Changelog

What changed in each release of **Blog Toolkit**, the desktop app for writing,
queuing and publishing blog posts with DeepSeek or Claude.

The public version of this page — with the download for each release — lives at
<https://mayorana.ch/en/apps/blog-toolkit/releases>. It is generated from this
file by `scripts/changelog_to_json.py`, so this file is the only place a
release note is written.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Each heading is dated on the day its tag was pushed. Releases that carried only
build or packaging work say so rather than being hidden: the version numbers a
user sees in the update prompt should all be accounted for.

## [0.1.8] - 2026-10-02

### Changed

- Packaging only — no user-visible change.

## [0.1.7] - 2026-08-28

### Changed

- The macOS download is a signed and notarized `.dmg` with a regular app
  bundle, which opens with a normal double-click. It used to be a bare program
  that macOS refused to open because the developer could not be verified. The
  disk image also carries `setup-mac.sh`, which installs Node.js for
  publishing if you need it.

## [0.1.6] - 2026-08-28

### Changed

- Packaging only — no user-visible change.

## [0.1.5] - 2026-08-27

### Added

- A LinkedIn tab: generate a LinkedIn post from a product or topic and the
  points you want to make, in French or English, keep it in a queue, and mark
  it as posted once you have published it on LinkedIn.
- Each generated blog post comes with a LinkedIn and an X teaser, shown under
  the post with a copy button.
- Per-project settings in a `blog-toolkit.yaml` file at the project root, so
  several sites can each have their own prompts and author. The content
  folders for each language are detected the first time a project is opened.
- A light and dark theme toggle, following the system theme at startup.

### Changed

- Downloads now come from mayorana.ch instead of GitHub.
- Blog Toolkit is source-available under the PolyForm Noncommercial licence:
  free for personal, educational and noncommercial use.

## [0.1.4] - 2026-05-22

### Changed

- Packaging only — no user-visible change.

## [0.1.3] - 2026-05-22

### Changed

- Blog Manager is now called Blog Toolkit. Settings are stored under the new
  name, so API keys saved in an earlier version have to be entered again.

## [0.1.2] - 2026-05-22

### Changed

- Packaging only — no user-visible change.

## [0.1.1] - 2026-05-22

### Added

- First release. Open a blog project folder and see its queued and published
  posts per language, with a rendered preview of the selected post.
- Generate a post from a title and a short summary with DeepSeek or Claude,
  and save it to the queue as a draft.
- Edit a queued post's markdown, then publish it to the site or delete it.
