# Contributing

## AI usage

AI assistance is welcome under the conditions in [AI-POLICY.md](AI-POLICY.md):
you remain fully responsible for what you submit, and disclosure of AI
involvement is expected.

## Layout

Single-project .NET solution:

- `src/FrontmatterMCP/` — the MCP server itself.
  - `Tools/` — `[McpServerTool]`-decorated methods, kept as thin adapters over `Core/`.
  - `Core/` — the actual logic (streaming extraction, YAML parsing, glob
    expansion, property projection), independently unit-testable with no MCP
    server bootstrapping required. Types here are `internal`.
  - `FrontmatterJsonContext.cs` — source-generated JSON metadata for every
    custom type that crosses the tool boundary (parameters and return
    values). Required for Native AOT: without an entry here, the published
    binary crashes at startup trying to resolve that type's schema. If you
    add a new tool parameter or return type, add it here too.
- `tests/FrontmatterMCP.Tests/` — TUnit tests, mirroring the `Core`/`Tools`
  structure, with fixture markdown files under `Fixtures/`.

The SDK version is pinned in `global.json`. Build-wide settings
(`TreatWarningsAsErrors`, analyzers, code style enforcement) live in
`Directory.Build.props` and apply to every project.

## Setup

```
dotnet restore
```

## Before opening a PR

These are the same checks CI runs (`.github/workflows/ci.yml`), so running
them locally avoids CI failures:

```
dotnet build frontmatter-mcp.slnx --no-restore -warnaserror
dotnet test --project tests/FrontmatterMCP.Tests
dotnet format frontmatter-mcp.slnx --verify-no-changes
```

If you touched anything under `src/FrontmatterMCP/`, also confirm it still
publishes clean under Native AOT — this is the check most likely to catch a
reflection-based regression (e.g. accidentally using a reflection-based
JSON/YAML API) before it reaches CI or a release:

```
dotnet publish src/FrontmatterMCP -c Release -r <your-RID> --self-contained -p:PublishAot=true
```

## Test-driven development

This project is built test-first: write a failing TUnit test before writing
the implementation it exercises. Add new fixture files under
`tests/FrontmatterMCP.Tests/Fixtures/` for new edge cases (malformed input,
encoding quirks, etc.) rather than constructing content inline, unless the
test is specifically about a large or generated payload — see
`FrontmatterReaderTests` for examples of generating large content in-memory
instead of committing large fixture files.

## Commit messages

Commit messages must follow [Conventional Commits](https://www.conventionalcommits.org/).
This is enforced in CI (the `commits` job in `ci.yml`, on pull requests) via
[`committed`](https://github.com/crate-ci/committed), configured in
`committed.toml`. To check locally before pushing:

```
cargo install committed   # one-time, if you have a Rust toolchain
committed origin/master..HEAD
```

It's fine to skip this locally and let CI catch it — `committed` isn't part
of the .NET build and adds no toolchain coupling.

## Changelogs and releases

Release notes are generated from conventional commit history via
[git-cliff](https://git-cliff.org/) (`cliff.toml`) when a GitHub release is
published — see `.github/workflows/release.yml`. You don't need to do
anything for this beyond writing a conventional commit message.

## License

AGPL-3.0 — see [LICENSE.md](LICENSE.md).
