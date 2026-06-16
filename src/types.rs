use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AskAnswer {
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub overview: String,
    pub key_points: Vec<String>,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub start_seconds: u32,
    pub end_seconds: u32,
    pub title: String,
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_deserializes_from_gemini_json() {
        let json = r#"[{"start_seconds":0,"end_seconds":90,"title":"Intro","summary":"Sets up the talk."}]"#;
        let chapters: Vec<Chapter> = serde_json::from_str(json).unwrap();
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].start_seconds, 0);
        assert_eq!(chapters[0].end_seconds, 90);
        assert_eq!(chapters[0].title, "Intro");
    }

    #[test]
    fn summary_roundtrips() {
        let s = Summary {
            overview: "A talk.".into(),
            key_points: vec!["one".into(), "two".into()],
            topics: vec!["rust".into()],
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Summary = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
