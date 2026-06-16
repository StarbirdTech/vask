# vask

Ask questions about YouTube videos using Gemini's native video understanding.

[![CI](https://github.com/StarbirdTech/vask/actions/workflows/ci.yml/badge.svg)](https://github.com/StarbirdTech/vask/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What it does

`vask` passes a YouTube URL directly to the Gemini API and lets Gemini analyze the actual video - frames and audio - without downloading anything or scraping transcripts. This means it understands visual content that captions never capture.

For example, asking "What animals are in this video?" about the first-ever YouTube upload returns:

> At the zoo, there are elephants visible in the background while the narrator stands in front of their enclosure.

That detail comes from watching the video, not reading a caption.

## Install

**From source (works today):**

```sh
git clone https://github.com/StarbirdTech/vask
cd vask
cargo install --path .
```

**Planned (not yet published):** `cargo install vask` (crates.io) and a Homebrew tap will be available once the crate is published.

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
vask ask https://youtu.be/jNQXAC9IVRw "What animals are visible in the video?"
```

```
At the zoo, there are elephants visible in the background while the narrator
stands in front of their enclosure and briefly describes them.
```

With `--json`:

```sh
vask ask --json https://youtu.be/jNQXAC9IVRw "What animals are visible in the video?"
```

```json
{
  "answer": "At the zoo, there are elephants visible in the background while the narrator stands in front of their enclosure and briefly describes them."
}
```

---

### `vask summarize`

Generate a structured summary: an overview paragraph, key points, and topic tags.

```sh
vask summarize https://youtu.be/jNQXAC9IVRw
```

```
A short clip filmed at a zoo, featuring a narrator standing in front of an
elephant enclosure and describing the animals to the camera.

Key points:
  - The narrator is at a zoo
  - Elephants are visible in the background
  - The clip is informal and candid in style

Topics: zoo, elephants, early internet, vlogging
```

---

### `vask chapters`

Get timestamped chapters with a one-line summary of each segment.

```sh
vask chapters https://youtu.be/jNQXAC9IVRw
```

```
00:00  Introduction at the Zoo
    Narrator introduces himself in front of the elephant enclosure.
00:19  Elephant Enclosure
    Brief description of the elephants and their surroundings.
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
