use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT_MODEL: &str = "gemini-3.5-flash";

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct FileConfig {
    pub api_key: Option<String>,
    pub default_model: Option<String>,
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("vask").join("config.toml"))
}

fn load_file_config() -> FileConfig {
    let Some(path) = config_path() else {
        return FileConfig::default();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return FileConfig::default();
    };
    toml::from_str(&text).unwrap_or_default()
}

fn resolve_inner(
    model_flag: Option<String>,
    key_flag: Option<String>,
    env_key: Option<String>,
    env_model: Option<String>,
    file: FileConfig,
) -> anyhow::Result<Config> {
    let api_key = key_flag.or(env_key).or(file.api_key).ok_or_else(|| {
        anyhow::anyhow!(
            "no API key found. Set GEMINI_API_KEY, pass --api-key, or add api_key to ~/.config/vask/config.toml"
        )
    })?;
    let model = model_flag
        .or(env_model)
        .or(file.default_model)
        .unwrap_or_else(|| DEFAULT_MODEL.to_string());
    Ok(Config { api_key, model })
}

pub fn resolve(model_flag: Option<String>, key_flag: Option<String>) -> anyhow::Result<Config> {
    resolve_inner(
        model_flag,
        key_flag,
        std::env::var("GEMINI_API_KEY").ok(),
        std::env::var("VASK_MODEL").ok(),
        load_file_config(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_beats_env_and_file() {
        let file = FileConfig {
            api_key: Some("file-key".into()),
            default_model: Some("file-model".into()),
        };
        let cfg = resolve_inner(
            Some("flag-model".into()),
            Some("flag-key".into()),
            Some("env-key".into()),
            Some("env-model".into()),
            file,
        )
        .unwrap();
        assert_eq!(cfg.api_key, "flag-key");
        assert_eq!(cfg.model, "flag-model");
    }

    #[test]
    fn env_beats_file_and_default_model_applies() {
        let file = FileConfig {
            api_key: None,
            default_model: None,
        };
        let cfg = resolve_inner(None, None, Some("env-key".into()), None, file).unwrap();
        assert_eq!(cfg.api_key, "env-key");
        assert_eq!(cfg.model, DEFAULT_MODEL);
    }

    #[test]
    fn missing_key_is_an_error() {
        let file = FileConfig {
            api_key: None,
            default_model: None,
        };
        let err = resolve_inner(None, None, None, None, file).unwrap_err();
        assert!(err.to_string().contains("GEMINI_API_KEY"));
    }
}
