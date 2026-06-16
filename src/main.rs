use clap::Parser;
use vask::cli::{self, Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if let Command::Serve = args.command {
        return vask::mcp::serve().await;
    }

    let cfg = vask::config::resolve(args.model.clone(), args.api_key.clone())?;
    let client = vask::gemini::HttpGeminiClient::new(cfg.api_key.clone());

    let out = match &args.command {
        Command::Ask { url, question } => cli::format_ask(
            &vask::analyze::ask(&client, &cfg.model, url, question).await?,
            args.json,
        ),
        Command::Summarize { url } => cli::format_summary(
            &vask::analyze::summarize(&client, &cfg.model, url).await?,
            args.json,
        ),
        Command::Chapters { url } => cli::format_chapters(
            &vask::analyze::chapters(&client, &cfg.model, url).await?,
            args.json,
        ),
        Command::Serve => unreachable!("handled above"),
    };

    println!("{out}");
    Ok(())
}
