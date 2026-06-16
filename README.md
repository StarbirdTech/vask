# vask

Ask questions about YouTube videos using Gemini's native video understanding.

[![CI](https://github.com/StarbirdTech/vask/actions/workflows/ci.yml/badge.svg)](https://github.com/StarbirdTech/vask/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What it does

`vask` passes a YouTube URL directly to the Gemini API and lets Gemini analyze the actual video - frames and audio - without downloading anything or scraping transcripts. This means it understands visual content that captions never capture.

For example, asking "What animals are mentioned and what is notable about them?" about the first-ever YouTube upload returns:

> The video mentions elephants (visible in the background from 0:00 to 0:19). The speaker notes that they have "really, really, really long trunks" (0:05 - 0:13).

That detail comes from watching the video, not reading a caption.

## Install

**From source (works today):**

```sh
git clone https://github.com/StarbirdTech/vask
cd vask
cargo install --path .
```

**Planned (not yet published):**

- `cargo install vask` (crates.io) - once the crate is published
- Homebrew tap - once the crate is published
- Binary download from GitHub Releases (planned)

## Auth

`vask` needs a Gemini API key. You can get one for free from [Google AI Studio](https://aistudio.google.com/apikey) - create a project without billing attached and the free tier is sufficient for all three commands.

**Three ways to supply it (highest priority first):**

1. `--api-key <key>` flag
2. `GEMINI_API_KEY` environment variable
3. `~/.config/vask/config.toml`:
   ```toml
   api_key = "AIza..."
   default_model = "gemini-3.5-flash"
   ```

The default model is `gemini-3.5-flash`.

## Usage

All commands share three optional global flags:

| Flag | Description |
|------|-------------|
| `--model <name>` | Override the Gemini model |
| `--api-key <key>` | Override the API key |
| `--json` | Output raw JSON instead of formatted text |

---

### `vask ask`

Ask a free-form question about a video. Gemini analyzes frames and audio to answer.

```sh
vask ask "https://www.youtube.com/watch?v=jNQXAC9IVRw" "What animals are mentioned and what is notable about them?"
```

```
The video mentions elephants (visible in the background from 0:00 to 0:19).
The speaker notes that they have "really, really, really long trunks" (0:05 - 0:13).
```

With `--json`:

```sh
vask ask --json "https://www.youtube.com/watch?v=jNQXAC9IVRw" "How long is this video roughly?"
```

```json
{
  "answer": "This video is roughly 19 seconds long."
}
```

---

### `vask summarize`

Generate a structured summary: an overview paragraph, key points, and topic tags.

```sh
vask summarize "https://www.youtube.com/watch?v=jNQXAC9IVRw"
```

```
In this historic video, YouTube co-founder Jawed Karim stands in front of the elephant exhibit at the San Diego Zoo. He briefly comments on the elephants behind him, noting their exceptionally long trunks and expressing that it is a cool feature before concluding his short message. This simple clip represents a major milestone as the very first video uploaded to YouTube.

Key points:
  - Jawed Karim stands in front of elephants at the San Diego Zoo.
  - He highlights the elephants' long trunks as a cool and unique feature.
  - The video is remarkably brief, ending with a simple sign-off.
  - This is historically significant as the first-ever video uploaded to YouTube on April 23, 2005.

Topics: YouTube History, San Diego Zoo, Elephants, Jawed Karim, First YouTube Video
```

---

### `vask chapters`

Get timestamped chapters with a one-line summary of each segment.

```sh
vask chapters "https://www.youtube.com/watch?v=jNQXAC9IVRw"
```

```
00:00  Introduction to the Elephants
    The speaker introduces himself and the elephants standing in the background at the zoo.
00:07  About Elephant Trunks
    The speaker discusses how cool elephants are because of their long trunks before concluding the short video.
```

---

## MCP server

`vask serve` starts an MCP server over stdio. This exposes three tools to any MCP-compatible client (Claude Desktop, Zed, VS Code with the MCP extension, etc.):

- `ask_video { url, question }` - same as `vask ask`
- `summarize_video { url }` - same as `vask summarize`
- `get_chapters { url }` - same as `vask chapters`

Add this to your MCP client config (the `vask` binary must be on `PATH`; `cargo install --path .` puts it there):

```json
{ "mcpServers": { "vask": { "command": "vask", "args": ["serve"] } } }
```

## How it works

`vask` uses the Gemini `fileData` field to pass a YouTube URL directly to the model. Gemini fetches and processes the video server-side - no local download, no frame extraction, no caption scraping. The model sees real frames and audio, which is why it can answer questions about visual content.

Flash is the default model (`gemini-3.5-flash`) for speed and cost. Pass `--model gemini-2.5-pro` (or set `default_model` in your config) to use a more capable model for longer or more complex videos.

## License

MIT - see [LICENSE](LICENSE).
