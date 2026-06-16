use serde_json::{json, Value};

/// Build the Gemini `generateContent` request body for a video URL + prompt.
/// When `schema` is provided, requests structured JSON output.
pub fn build_request_body(prompt: &str, video_url: &str, schema: Option<&Value>) -> Value {
    let mut body = json!({
        "contents": [{
            "parts": [
                { "text": prompt },
                { "file_data": { "file_uri": video_url } }
            ]
        }]
    });
    if let Some(s) = schema {
        body["generationConfig"] = json!({
            "responseMimeType": "application/json",
            "responseSchema": s
        });
    }
    body
}

/// Extract the generated text from a Gemini response. Scans all parts for the
/// first one carrying a `text` field, so Gemini 3.x "thought" parts (which
/// carry only a `thoughtSignature`) are skipped rather than mistaken for output.
pub fn parse_response_text(body: &Value) -> anyhow::Result<String> {
    let parts = body
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .ok_or_else(|| anyhow::anyhow!("no candidates/parts in Gemini response: {body}"))?;
    parts
        .iter()
        .find_map(|p| p.get("text").and_then(|t| t.as_str()))
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("no text part in Gemini response: {body}"))
}

#[async_trait::async_trait]
pub trait GeminiClient {
    async fn generate(
        &self,
        model: &str,
        prompt: &str,
        video_url: &str,
        schema: Option<Value>,
    ) -> anyhow::Result<String>;
}

pub struct HttpGeminiClient {
    api_key: String,
    http: reqwest::Client,
}

impl HttpGeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl GeminiClient for HttpGeminiClient {
    async fn generate(
        &self,
        model: &str,
        prompt: &str,
        video_url: &str,
        schema: Option<Value>,
    ) -> anyhow::Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={}",
            self.api_key
        );
        let body = build_request_body(prompt, video_url, schema.as_ref());
        let resp = self.http.post(&url).json(&body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            anyhow::bail!("Gemini API error {status}: {text}");
        }
        let parsed: Value = serde_json::from_str(&text)?;
        parse_response_text(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_body_has_text_and_file_data_parts() {
        let body = build_request_body("hello", "https://youtu.be/abc", None);
        let parts = &body["contents"][0]["parts"];
        assert_eq!(parts[0]["text"], "hello");
        assert_eq!(parts[1]["file_data"]["file_uri"], "https://youtu.be/abc");
        assert!(body.get("generationConfig").is_none());
    }

    #[test]
    fn request_body_includes_schema_when_present() {
        let schema = json!({"type": "object"});
        let body = build_request_body("hi", "https://youtu.be/abc", Some(&schema));
        assert_eq!(
            body["generationConfig"]["responseMimeType"],
            "application/json"
        );
        assert_eq!(body["generationConfig"]["responseSchema"], schema);
    }

    #[test]
    fn parse_extracts_candidate_text() {
        let body = json!({
            "candidates": [{ "content": { "parts": [{ "text": "the answer" }] } }]
        });
        assert_eq!(parse_response_text(&body).unwrap(), "the answer");
    }

    #[test]
    fn parse_skips_thought_part_finds_text() {
        let body = json!({
            "candidates": [{ "content": { "parts": [
                { "thoughtSignature": "abc" },
                { "text": "the answer", "thoughtSignature": "xyz" }
            ] } }]
        });
        assert_eq!(parse_response_text(&body).unwrap(), "the answer");
    }

    #[test]
    fn parse_errors_on_missing_text() {
        let body = json!({ "candidates": [] });
        assert!(parse_response_text(&body).is_err());
    }

    #[tokio::test]
    async fn live_generate_when_key_present() {
        let Ok(key) = std::env::var("GEMINI_API_KEY") else {
            eprintln!("skipping live test: GEMINI_API_KEY not set");
            return;
        };
        let client = HttpGeminiClient::new(key);
        let out = client
            .generate(
                crate::config::DEFAULT_MODEL,
                "In one word, what language is this video about? Answer with a single word.",
                "https://www.youtube.com/watch?v=9hE5-98ZeCg",
                None,
            )
            .await;
        assert!(out.is_ok(), "live call failed: {out:?}");
    }
}
