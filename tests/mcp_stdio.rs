//! End-to-end test of the MCP wiring: an in-process `rmcp` client talks
//! to `FrontmatterServer` over an in-memory duplex transport. Everything
//! else is covered by the per-module tests; this only checks the
//! transport, tool names, schema shape, response casing, and the
//! validation error path.

mod common;

use common::fixture;
use frontmatter_mcp::mcp::FrontmatterServer;
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::{Map, Value, json};

/// Starts the server and a bare client connected over `tokio::io::duplex`,
/// returning the running client service.
async fn connect() -> rmcp::service::RunningService<rmcp::RoleClient, ()> {
    let (server_transport, client_transport) = tokio::io::duplex(4096);
    tokio::spawn(async move {
        let server = FrontmatterServer::new()
            .serve(server_transport)
            .await
            .expect("server failed to start");
        let _ = server.waiting().await;
    });
    ().serve(client_transport)
        .await
        .expect("client failed to start")
}

/// Turns a fixture path into a JSON-object arguments map with a `path` key.
fn path_args(name: &str) -> Map<String, Value> {
    json!({ "path": fixture(name).to_string_lossy() })
        .as_object()
        .unwrap()
        .clone()
}

/// The server advertises exactly the three tools, by their `snake_case`
/// names, and the `format` argument is a three-value string enum.
#[tokio::test]
async fn lists_the_three_tools_with_string_format_enum() {
    let client = connect().await;

    let tools = client.peer().list_tools(None).await.unwrap();
    let mut names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec![
            "get_frontmatter_properties",
            "read_frontmatter",
            "read_frontmatter_batch",
        ]
    );

    let read = tools
        .tools
        .iter()
        .find(|t| t.name == "read_frontmatter")
        .unwrap();
    let schema = serde_json::to_string(&read.input_schema).unwrap();
    assert!(schema.contains("\"Parsed\""));
    assert!(schema.contains("\"Raw\""));
    assert!(schema.contains("\"Both\""));

    client.cancel().await.unwrap();
}

/// Every tool's output schema is a JSON object at the top level - strict
/// MCP clients (Claude's agent modes) reject a `tools/list` where any
/// `outputSchema.type` is not `"object"`, which is why the batch tools
/// wrap their results.
#[tokio::test]
async fn every_output_schema_is_an_object() {
    let client = connect().await;

    let tools = client.peer().list_tools(None).await.unwrap();
    for tool in &tools.tools {
        let output_schema = tool
            .output_schema
            .as_ref()
            .unwrap_or_else(|| panic!("{} has no output schema", tool.name));
        assert_eq!(
            output_schema.get("type").and_then(Value::as_str),
            Some("object"),
            "{} output schema is not an object: {output_schema:?}",
            tool.name
        );
    }

    client.cancel().await.unwrap();
}

/// A successful call returns camelCase structured content.
#[tokio::test]
async fn call_returns_camel_case_structured_content() {
    let client = connect().await;

    let result = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("read_frontmatter")
                .with_arguments(path_args("valid-simple.md")),
        )
        .await
        .unwrap();

    let structured = result
        .structured_content
        .expect("expected structured content");
    let object = structured.as_object().unwrap();
    assert!(object.get("path").and_then(Value::as_str).is_some());
    assert_eq!(object.get("hasFrontmatter"), Some(&json!(true)));
    assert_eq!(
        object.get("parsed"),
        Some(&json!({"title": "Sample Document", "status": "draft"}))
    );

    client.cancel().await.unwrap();
}

/// Calling a batch tool with neither `paths` nor `glob` fails with the
/// exact validation message as a protocol error.
#[tokio::test]
async fn validation_failure_is_a_protocol_error() {
    let client = connect().await;

    let error = client
        .peer()
        .call_tool(CallToolRequestParams::new("read_frontmatter_batch"))
        .await
        .expect_err("expected an error for missing paths/glob");

    assert!(
        error
            .to_string()
            .contains("Provide exactly one of 'paths' or 'glob'."),
        "unexpected error: {error}"
    );

    client.cancel().await.unwrap();
}
