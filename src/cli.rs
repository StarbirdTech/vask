use crate::types::{AskAnswer, Chapter, Summary};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "vask",
    version,
    about = "Ask questions about YouTube videos with Gemini"
)]
pub struct Cli {
    /// Override the Gemini model (default: gemini-3.5-flash)
    #[arg(long, global = true)]
    pub model: Option<String>,

    /// Override the API key (otherwise GEMINI_API_KEY or config file)
    #[arg(long = "api-key", global = true)]
    pub api_key: Option<String>,

    /// Emit machine-readable JSON instead of formatted text
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Ask a free-form question about a video
    Ask { url: String, question: String },
    /// Summarize a video
    Summarize { url: String },
    /// Get timestamped chapters for a video
    Chapters { url: String },
    /// Run as an MCP server over stdio
    Serve,
}

pub fn format_ask(a: &AskAnswer, json: bool) -> String {
    if json {
        serde_json::to_string_pretty(a).expect("serializing a plain struct to JSON cannot fail")
    } else {
        a.answer.clone()
    }
}

pub fn format_summary(s: &Summary, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(s)
            .expect("serializing a plain struct to JSON cannot fail");
    }
    let mut out = String::new();
    out.push_str(&s.overview);
    out.push_str("\n\nKey points:\n");
    for p in &s.key_points {
        out.push_str(&format!("  - {p}\n"));
    }
    out.push_str("\nTopics: ");
    out.push_str(&s.topics.join(", "));
    out
}

pub fn format_chapters(c: &[Chapter], json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(c)
            .expect("serializing a plain struct to JSON cannot fail");
    }
    let mut out = String::new();
    for ch in c {
        let m = ch.start_seconds / 60;
        let s = ch.start_seconds % 60;
        out.push_str(&format!(
            "{m:02}:{s:02}  {}\n    {}\n",
            ch.title, ch.summary
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AskAnswer, Chapter, Summary};
    use clap::Parser;

    #[test]
    fn parses_ask_subcommand() {
        let cli = Cli::try_parse_from(["vask", "ask", "https://youtu.be/x", "why?"]).unwrap();
        match cli.command {
            Command::Ask { url, question } => {
                assert_eq!(url, "https://youtu.be/x");
                assert_eq!(question, "why?");
            }
            _ => panic!("wrong subcommand"),
        }
    }

    #[test]
    fn format_chapters_pretty_uses_mmss() {
        let c = vec![Chapter {
            start_seconds: 75,
            end_seconds: 120,
            title: "Setup".into(),
            summary: "Configures things.".into(),
        }];
        let out = format_chapters(&c, false);
        assert!(out.contains("01:15"));
        assert!(out.contains("Setup"));
    }

    #[test]
    fn format_ask_json_is_object() {
        let a = AskAnswer {
            answer: "hi".into(),
        };
        let out = format_ask(&a, true);
        assert!(out.contains("\"answer\""));
    }

    #[test]
    fn format_summary_pretty_lists_points() {
        let s = Summary {
            overview: "o".into(),
            key_points: vec!["a".into()],
            topics: vec!["t".into()],
        };
        let out = format_summary(&s, false);
        assert!(out.contains("- a"));
        assert!(out.contains("Topics: t"));
    }
}
