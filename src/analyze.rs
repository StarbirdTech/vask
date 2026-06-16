use crate::gemini::GeminiClient;
use crate::types::{AskAnswer, Chapter, Summary};
use serde_json::json;

pub fn validate_youtube_url(url: &str) -> anyhow::Result<()> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| anyhow::anyhow!("URL must start with http(s)://: {url}"))?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.strip_prefix("www.").unwrap_or(host);
    let ok = matches!(host, "youtube.com" | "m.youtube.com" | "youtu.be");
    if ok {
        Ok(())
    } else {
        anyhow::bail!("not a recognized YouTube URL (host '{host}'): {url}")
    }
}

pub async fn ask<C: GeminiClient>(
    client: &C,
    model: &str,
    url: &str,
    question: &str,
) -> anyhow::Result<AskAnswer> {
    validate_youtube_url(url)?;
    let prompt = format!(
        "Answer this question about the video. Be concise and specific, and cite timestamps where useful.\n\nQuestion: {question}"
    );
    let answer = client.generate(model, &prompt, url, None).await?;
    Ok(AskAnswer { answer })
}

pub async fn summarize<C: GeminiClient>(
    client: &C,
    model: &str,
    url: &str,
) -> anyhow::Result<Summary> {
    validate_youtube_url(url)?;
    let schema = json!({
        "type": "object",
        "properties": {
            "overview": { "type": "string" },
            "key_points": { "type": "array", "items": { "type": "string" } },
            "topics": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["overview", "key_points", "topics"]
    });
    let prompt = "Summarize this video. Provide a one-paragraph overview, a list of key points, and a list of topics covered.";
    let text = client.generate(model, prompt, url, Some(schema)).await?;
    let summary: Summary = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("could not parse summary JSON: {e}; raw: {text}"))?;
    Ok(summary)
}

pub async fn chapters<C: GeminiClient>(
    client: &C,
    model: &str,
    url: &str,
) -> anyhow::Result<Vec<Chapter>> {
    validate_youtube_url(url)?;
    let schema = json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "start_seconds": { "type": "integer" },
                "end_seconds": { "type": "integer" },
                "title": { "type": "string" },
                "summary": { "type": "string" }
            },
            "required": ["start_seconds", "end_seconds", "title", "summary"]
        }
    });
    let prompt = "Break this video into logical chapters. For each chapter provide start_seconds, end_seconds, a short title, and a one-sentence summary.";
    let text = client.generate(model, prompt, url, Some(schema)).await?;
    let chapters: Vec<Chapter> = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("could not parse chapters JSON: {e}; raw: {text}"))?;
    Ok(chapters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gemini::GeminiClient;
    use serde_json::Value;

    struct MockClient {
        canned: String,
    }

    #[async_trait::async_trait]
    impl GeminiClient for MockClient {
        async fn generate(
            &self,
            _m: &str,
            _p: &str,
            _u: &str,
            _s: Option<Value>,
        ) -> anyhow::Result<String> {
            Ok(self.canned.clone())
        }
    }

    #[test]
    fn validate_accepts_youtube_and_rejects_other() {
        assert!(validate_youtube_url("https://www.youtube.com/watch?v=abc").is_ok());
        assert!(validate_youtube_url("https://youtu.be/abc").is_ok());
        assert!(validate_youtube_url("https://m.youtube.com/watch?v=abc").is_ok());
        assert!(validate_youtube_url("https://youtube.com/shorts/abc").is_ok());
        assert!(validate_youtube_url("https://vimeo.com/123").is_err());
        assert!(validate_youtube_url("https://evil.com/youtu.be/x").is_err());
        assert!(validate_youtube_url("ftp://youtu.be/x").is_err());
    }

    #[tokio::test]
    async fn ask_wraps_raw_text() {
        let client = MockClient {
            canned: "it is about rust".into(),
        };
        let a = ask(&client, "m", "https://youtu.be/abc", "what?")
            .await
            .unwrap();
        assert_eq!(a.answer, "it is about rust");
    }

    #[tokio::test]
    async fn summarize_parses_structured_json() {
        let client = MockClient {
            canned: r#"{"overview":"a talk","key_points":["x"],"topics":["rust"]}"#.into(),
        };
        let s = summarize(&client, "m", "https://youtu.be/abc")
            .await
            .unwrap();
        assert_eq!(s.overview, "a talk");
        assert_eq!(s.topics, vec!["rust".to_string()]);
    }

    #[tokio::test]
    async fn chapters_parses_array() {
        let client = MockClient {
            canned: r#"[{"start_seconds":0,"end_seconds":60,"title":"Intro","summary":"s"}]"#
                .into(),
        };
        let c = chapters(&client, "m", "https://youtu.be/abc")
            .await
            .unwrap();
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].title, "Intro");
    }

    #[tokio::test]
    async fn ask_rejects_non_youtube_before_calling() {
        let client = MockClient {
            canned: "unused".into(),
        };
        assert!(ask(&client, "m", "https://vimeo.com/1", "q").await.is_err());
    }
}
