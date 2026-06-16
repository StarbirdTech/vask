# vask - Design Spec

**Date:** 2026-06-16
**Status:** Approved, pre-implementation
**Crate / command:** `vask`
**Repo:** `StarbirdTech/vask` (public, MIT)

## Overview

`vask` is a fast, native command-line tool and MCP server for asking questions about videos using Gemini's native video understanding. Point it at a YouTube URL and get answers, summaries, or timestamped chapters back, with no download, no transcript scraping, and no local media pipeline.

The differentiator: Gemini 3.5 accepts a YouTube URL directly as a `fileData` part and processes the actual video (visual frames plus audio) server-side. vask wraps that one capability in a clean CLI and an MCP server so it is usable both from a terminal and from inside Claude Code / Claude Desktop.

### Problem it solves

Research into the existing landscape (June 2026) found:

- The **transcript-extraction** space is saturated (5+ MCP servers with 400-800+ stars).
- The **Gemini-native YouTube-URL** space is nearly empty. The best existing Gemini-native MCP server has 6 stars and was last touched April 2025; the runner-up has 3 stars and requires a manual clone.
- The **official Gemini CLI explicitly declined** to add YouTube URL support (issues #1691, #15634, both closed "not planned").
- The only tool doing native Gemini YouTube ingestion well is Simon Willison's general-purpose `llm + llm-gemini`, which has no video-specific UX (no chapters, no structured output, no MCP).

The gap: a purpose-built, provider-flexible tool that ships both a CLI and an MCP server from one codebase, returns structured output, and leads with Gemini-native video understanding. Nobody has built it. vask does.

## Why Rust

- **Single static binary.** Distribution is `cargo install vask`, a Homebrew tap, or a downloaded binary from GitHub Releases. No runtime, no `node_modules`, no npm registry scope-name dance.
- **The crate name `vask` is free** on crates.io (verified 2026-06-16).
- **Differentiated.** Nearly every tool in this space is a TypeScript or Python MCP server. A fast native-Rust tool doing Gemini-native video understanding stands out.
- **No SDK dependency needed.** The Gemini API is plain REST/JSON; the native YouTube-URL feature is a single JSON field (`fileData.fileUri`). A thin `reqwest` client is cleaner than a heavy SDK.
- **Official Rust MCP SDK exists.** `rmcp` (modelcontextprotocol/rust-sdk) is production-ready, async on tokio, and builds servers exposing tools/resources/prompts.

**Accepted tradeoff:** the MCP install is binary-on-PATH (`"command": "vask", "args": ["serve"]`) rather than npx auto-fetch. This requires `vask` to be installed first (cargo/brew). This is clean, starts faster than npx, and is an acceptable cost for the single-binary distribution win.

## Scope

### v1 (this spec)

- Three operations: `ask`, `summarize`, `chapters`.
- Two front doors over one shared core: a CLI and an MCP stdio server.
- Gemini-native provider only (direct REST, YouTube URL passthrough).
- Structured JSON output (via Gemini structured-output mode) plus human-readable CLI formatting.
- Public YouTube videos.

### Explicit non-goals (v1.1+, documented not built)

- OpenRouter provider + frame-extraction path (yt-dlp + ffmpeg). This is the path to unified billing and to non-Gemini models; deferred because it pulls in heavy local dependencies and loses the zero-dependency elegance.
- Local video file input (same frame-extraction path as above).
- Additional tools: `highlights` (best N-second moments), `visual` (extract on-screen slides/code/text).
- Private / age-gated videos.
- Remote / hosted MCP (Streamable HTTP, OAuth). v1 is local stdio only. See MCP Transport below.

## Architecture

```
vask/                          # crate: vask, command: vask
├── Cargo.toml
├── src/
│   ├── main.rs                # entry; clap dispatches subcommands + `serve`
│   ├── cli.rs                 # arg parsing, output formatting (pretty | --json)
│   ├── config.rs              # key resolution: $GEMINI_API_KEY, ~/.config/vask/config.toml
│   ├── gemini.rs              # reqwest client: builds generateContent w/ fileData.fileUri
│   ├── analyze.rs             # the 3 ops: ask / summarize / chapters -> typed results
│   ├── types.rs               # serde structs: Chapter, Summary, AskAnswer
│   └── mcp.rs                 # rmcp server exposing the same 3 ops as MCP tools
├── tests/                     # parsing + request-shape tests against fixtures
└── README.md
```

**Single source of truth:** `analyze.rs` holds the three operations. Both the CLI and the MCP server call the same functions. The CLI formats results for humans; the MCP server returns them as structured JSON tool results. No business logic is duplicated between interfaces.

**Gemini client is trait-abstracted** so tests inject canned responses and CI runs fully offline with no API key.

## Interfaces

### CLI

```bash
vask ask <url> "what are the three main arguments?"
vask summarize <url>
vask chapters <url>
vask ask <url> "..." --json          # machine-readable output
vask --model gemini-3.5-pro ...      # override default flash
```

### MCP server (stdio)

```bash
vask serve                            # speaks MCP over stdio
```

Tools exposed: `ask_video {url, question}`, `summarize_video {url}`, `get_chapters {url}`. Each returns structured JSON.

README MCP config block:

```json
{ "mcpServers": { "vask": { "command": "vask", "args": ["serve"] } } }
```

## Data flow

`vask ask <url> "q"` -> `cli.rs` parses -> `analyze::ask(url, q)` -> `gemini.rs` builds a `generateContent` POST with the YouTube URL as a `fileData` part plus the prompt -> Gemini processes the video natively -> response parsed into a typed struct -> `cli.rs` prints (pretty or `--json`).

MCP tool `ask_video {url, question}` -> `analyze::ask(url, q)` -> returns the typed struct serialized as the tool result. Same function, same output.

## Gemini integration

- Endpoint: `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key=KEY`
- Body `contents[].parts`: `[{fileData:{fileUri: url, mimeType: "video/mp4"}}, {text: prompt}]`
- `summarize` and `chapters` use Gemini structured-output mode (`responseMimeType: "application/json"` + `responseSchema`) so output parses cleanly into `Summary` / `Vec<Chapter>` with no fragile string-munging.
- `ask` returns a free-form answer, optionally with cited timestamps.
- Default model `gemini-3.5-flash` (GA May 2026; native video understanding with timestamped insights and a 1M-token context, well suited to chapter/timestamp extraction). `--model` / config can bump to `gemini-3.5-pro` for long or complex videos once it is GA, or pin any other Gemini model. The model is configurable precisely so the default can track the current best model without a code change; "3.5-flash" is the right default as of 2026-06, not a permanent commitment.

## Data types (serde)

- `AskAnswer { answer: String, citations: Vec<TimestampCite> }` (citations optional/best-effort)
- `Summary { overview: String, key_points: Vec<String>, topics: Vec<String> }`
- `Chapter { start_seconds: u32, end_seconds: u32, title: String, summary: String }`

Exact fields finalized during implementation; schemas drive both the Gemini `responseSchema` and the Rust structs.

## Config and auth

- `GEMINI_API_KEY` environment variable (primary).
- `~/.config/vask/config.toml`: `api_key`, `default_model`.
- Flag overrides: `--model`, `--json`.
- Resolution precedence: flag > env > config file.

## Error handling

- No API key -> clear message naming the exact env var and config path to set.
- Gemini 4xx/5xx -> surface status + message. Private / age-gated video -> explain v1 supports public videos only.
- Non-YouTube or malformed URL -> validate and reject early with a helpful message.
- Structured-output parse failure -> fall back to returning raw text with a warning rather than crashing.
- MCP mode -> errors returned as MCP tool errors with the same helpful messages.

## MCP transport decision

vask v1 uses the **stdio transport**, which is the correct and standard transport for a local MCP server. Prior research (`~/git/research/2026/06-mcp-state-of-art-2026/`) confirms MCP has stabilized to exactly two standard transports: stdio (local) and Streamable HTTP (remote); the older standalone HTTP+SSE transport is deprecated and removed. Because vask is local-first, stdio lets it avoid the entire remote-server surface (OAuth 2.1, Streamable HTTP, session management) that dominates the current MCP landscape. The spec is actively evolving (2026-07-28 RC), but that churn is concentrated in the HTTP/remote layer; stdio has stayed stable, which de-risks the `rmcp` dependency for this use case.

A future remote / hosted mode, if ever wanted, would use Streamable HTTP per the spec and is explicitly out of scope for v1.

## Testing

- Gemini client trait-abstracted; tests inject canned responses. **CI runs fully offline**, no API key required.
- Unit tests: request-body shape, response-fixture -> typed-struct parsing, config-resolution precedence.
- One optional live test gated behind `$GEMINI_API_KEY` (off in CI) hitting a short known public video.

## Distribution and "looks good on GitHub" (v1 deliverables)

- README with **real example output** (e.g. chapters pulled from a known tech talk), a copy-paste MCP config block, and install one-liners (`cargo install`, Homebrew, binary download).
- MIT license.
- GitHub Actions CI: `cargo fmt --check`, `cargo clippy`, `cargo test`.
- Release workflow producing macOS arm64/x64 + Linux binaries.
- Clean `--help` and a documented `examples` section.

## Related research

- `~/git/research/2026/06-mcp-state-of-art-2026/` - MCP transport model, SDK landscape, stdio vs Streamable HTTP (informs the transport decision above).
- `~/git/research/2026/06-video-editing-ai-concept/` - sibling concept exploring Gemini-as-edit-decision-engine; shares the Gemini-native video understanding foundation.

## Open questions / future

- v1.1: OpenRouter + frame-extraction path for unified billing and non-Gemini models, plus local-file input.
- v1.2: `highlights` and `visual` tools.
- Whether to add a Homebrew tap at v1 or wait for traction.
- **Remote, Cloudflare-hostable MCP variant (future, wanted).** A hosted version exposing vask over Streamable HTTP (the standard remote MCP transport) so it can be added to clients without a local binary. Research already on hand: `~/git/research/2026/06-mcp-state-of-art-2026/` covers Streamable HTTP, stateless-HTTP load balancing, OAuth 2.1, and the `EdgeFastMCP` path for Cloudflare Workers. This would reuse the same `analyze` core; only a new transport/front-door and auth layer are added. The Gemini call itself is plain HTTPS and runs fine from a Worker.
