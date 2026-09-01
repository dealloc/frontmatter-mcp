//! `frontmatter-mcp` is an MCP server that reads only the YAML frontmatter
//! block of markdown documents - skills, ADRs, PIRs, docs, anything with a
//! `---`-delimited header - without loading the rest of the document.
//!
//! Modules are added incrementally as the Rust rewrite proceeds; see the
//! project's migration plan for the full module map.

pub mod frontmatter;
pub mod glob;
pub mod projection;
pub mod tools;
pub mod yaml;
