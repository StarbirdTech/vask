use crate::{analyze, config, gemini::HttpGeminiClient};
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct VideoArg {
    /// A public YouTube video URL
    pub url: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct AskArg {
    /// A public YouTube video URL
    pub url: String,
    /// The question to ask about the video
    pub question: String,
}

#[derive(Clone)]
pub struct VaskServer {
    model: String,
    api_key: String,
    // Held to satisfy the rmcp tool-router convention. The #[tool_handler] macro
    // dispatches through Self::tool_router(), so the instance field is not read
    // directly and would otherwise trip dead_code.
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl VaskServer {
    pub fn new(model: String, api_key: String) -> Self {
        Self {
            model,
            api_key,
            tool_router: Self::tool_router(),
        }
    }

    fn client(&self) -> HttpGeminiClient {
        HttpGeminiClient::new(self.api_key.clone())
    }

    #[tool(description = "Ask a free-form question about a public YouTube video")]
    async fn ask_video(
        &self,
        Parameters(a): Parameters<AskArg>,
    ) -> Result<CallToolResult, McpError> {
        let res = analyze::ask(&self.client(), &self.model, &a.url, &a.question)
            .await
            .map_err(internal)?;
        Ok(CallToolResult::success(vec![
            Content::json(res).map_err(internal)?
        ]))
    }

    #[tool(description = "Summarize a public YouTube video")]
    async fn summarize_video(
        &self,
        Parameters(a): Parameters<VideoArg>,
    ) -> Result<CallToolResult, McpError> {
        let res = analyze::summarize(&self.client(), &self.model, &a.url)
            .await
            .map_err(internal)?;
        Ok(CallToolResult::success(vec![
            Content::json(res).map_err(internal)?
        ]))
    }

    #[tool(description = "Get timestamped chapters for a public YouTube video")]
    async fn get_chapters(
        &self,
        Parameters(a): Parameters<VideoArg>,
    ) -> Result<CallToolResult, McpError> {
        let res = analyze::chapters(&self.client(), &self.model, &a.url)
            .await
            .map_err(internal)?;
        Ok(CallToolResult::success(vec![
            Content::json(res).map_err(internal)?
        ]))
    }
}

#[tool_handler]
impl ServerHandler for VaskServer {
    fn get_info(&self) -> ServerInfo {
        // ServerInfo (InitializeResult) is #[non_exhaustive], so start from the
        // default and mutate the fields we care about instead of a struct literal.
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some("Ask questions about YouTube videos using Gemini.".into());
        info.server_info = rmcp::model::Implementation::new("vask", env!("CARGO_PKG_VERSION"));
        info
    }
}

fn internal<E: std::fmt::Display>(e: E) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

pub async fn serve() -> anyhow::Result<()> {
    let cfg = config::resolve(None, None)?;
    let server = VaskServer::new(cfg.model, cfg.api_key);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
