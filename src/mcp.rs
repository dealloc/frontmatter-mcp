//! The MCP server module: binds the three tools to the stdio transport via
//! `rmcp`. All behavior lives in [`crate::tools`]; this module is just
//! wiring.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};

use crate::tools::{
    self, BatchResults, FrontmatterResult, GetFrontmatterPropertiesArgs, PropertyResult,
    ReadFrontmatterArgs, ReadFrontmatterBatchArgs, ResolveError,
};

/// The frontmatter MCP server. Holds the generated tool router; construct
/// with [`FrontmatterServer::new`].
#[derive(Clone)]
pub struct FrontmatterServer {
    /// The `rmcp`-generated router mapping tool names to handlers.
    tool_router: ToolRouter<Self>,
}

impl FrontmatterServer {
    /// Creates a server with all three tools registered.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Resolves the batch file list, mapping a resolution failure to an
    /// `invalid_params` protocol error.
    ///
    /// # Errors
    ///
    /// Returns an `invalid_params` error if the paths/glob rule is
    /// violated or the glob is invalid, or an `internal_error` if the
    /// working directory can't be read.
    fn resolve(
        paths: Option<&[String]>,
        glob: Option<&str>,
        max_files: usize,
    ) -> Result<Vec<String>, ErrorData> {
        let cwd = std::env::current_dir()
            .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
        tools::resolve_paths(paths, glob, max_files, &cwd)
            .map_err(|error: ResolveError| ErrorData::invalid_params(error.message(), None))
    }
}

impl Default for FrontmatterServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl FrontmatterServer {
    /// Reads only the YAML frontmatter of a single markdown file, without
    /// loading the document body. A file with no `---`-delimited
    /// frontmatter reports `hasFrontmatter: false` rather than an error.
    #[tool(
        description = "Reads only the YAML frontmatter of a single markdown file, without loading the document body. Files with no ---delimited frontmatter report hasFrontmatter: false rather than an error."
    )]
    async fn read_frontmatter(
        &self,
        Parameters(args): Parameters<ReadFrontmatterArgs>,
    ) -> Json<FrontmatterResult> {
        Json(tools::read_one(&args.path, args.format).await)
    }

    /// Reads only the YAML frontmatter from many markdown files at once,
    /// given either explicit `paths` or a `glob` pattern. Provide exactly
    /// one.
    #[tool(
        description = "Reads only the YAML frontmatter from multiple markdown files, given either explicit paths or a glob pattern (** recurses). Provide exactly one of paths or glob."
    )]
    async fn read_frontmatter_batch(
        &self,
        Parameters(args): Parameters<ReadFrontmatterBatchArgs>,
    ) -> Result<Json<BatchResults<FrontmatterResult>>, ErrorData> {
        let paths = Self::resolve(args.paths.as_deref(), args.glob.as_deref(), args.max_files)?;
        let results = tools::read_batch(&paths, args.format).await;
        Ok(Json(BatchResults { results }))
    }

    /// Extracts specific named properties from the frontmatter of many
    /// markdown files. Dotted names walk nested objects; missing
    /// properties are reported explicitly.
    #[tool(
        description = "Extracts specific named properties from the frontmatter of multiple markdown files (by explicit paths or glob). Supports dotted paths for nested keys (e.g. metadata.owner). Missing properties are reported explicitly."
    )]
    async fn get_frontmatter_properties(
        &self,
        Parameters(args): Parameters<GetFrontmatterPropertiesArgs>,
    ) -> Result<Json<BatchResults<PropertyResult>>, ErrorData> {
        let paths = Self::resolve(args.paths.as_deref(), args.glob.as_deref(), args.max_files)?;
        let results = tools::project_batch(&paths, &args.properties).await;
        Ok(Json(BatchResults { results }))
    }
}

// `router = self.tool_router` uses the cached field rather than the macro
// default of `Self::tool_router()`, which would rebuild the whole router
// on every tool call.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for FrontmatterServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Reads only the YAML frontmatter of markdown documents, without loading the document body.",
            )
    }
}
