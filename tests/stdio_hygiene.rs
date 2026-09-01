//! Spawns the real built binary and checks that stdout carries only MCP
//! protocol frames (parseable JSON, one per line) while every log line
//! goes to stderr - the invariant that lets the server share a process's
//! stdio with a client.

use std::io::Write as _;
use std::process::{Command, Stdio};

/// A minimal `initialize` + `initialized` + `tools/list` exchange.
const REQUESTS: &str = concat!(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"hygiene","version":"0"}}}"#,
    "\n",
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    "\n",
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    "\n",
);

/// stdout is only newline-delimited JSON; the startup log lands on stderr.
#[test]
fn stdout_is_only_json_and_logs_go_to_stderr() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_frontmatter-mcp"))
        .env("RUST_LOG", "info")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn the server binary");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(REQUESTS.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(
        lines.len(),
        2,
        "expected 2 responses on stdout, got: {stdout:?}"
    );
    for line in &lines {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|error| panic!("non-JSON line on stdout: {line:?} ({error})"));
    }

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("frontmatter-mcp") && stderr.contains("starting"),
        "expected startup log on stderr, got: {stderr:?}"
    );
}

/// `--version` prints a line and exits 0 without starting the server.
#[test]
fn version_flag_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_frontmatter-mcp"))
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("frontmatter-mcp "), "got: {stdout:?}");
}
