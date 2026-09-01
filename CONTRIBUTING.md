# Contributing

## Prerequisites

- The Rust toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml)
  (installed automatically by `rustup` when you run any `cargo` command in
  this repo).
- [`just`](https://just.systems) as the task runner.
- The rest of the tooling `just precommit` needs — install it in one step:

  ```sh
  just setup
  ```

## Workflow

- **Test-first.** For a new module or behavior, write the test(s) first, see
  them fail, then implement until they pass. No module lands without tests
  written before its implementation.
- **Warnings are errors, always.** Never leave a `cargo check` / `cargo
  clippy` warning standing while you work — fix it immediately. `#[allow(...)]`
  is not an escape hatch here; if a lint fires, either the code or the lint
  configuration changes, not an `allow`.
- **Let the tools format the code.** Don't hand-align or hand-wrap code —
  run `cargo fmt` (and `taplo format` for TOML) and let them decide.
- **Prefer `cargo` commands over hand-edited `Cargo.toml`.** Add a
  dependency with `cargo add <crate>` (or `cargo add --dev <crate>` for a
  dev-dependency) rather than typing a version in by hand.

Before committing, run:

```sh
just precommit
```

This runs the exact checks CI runs: formatting (`cargo fmt`, `taplo`),
linting (`cargo check`, `cargo clippy -- -D warnings`), dependency hygiene
(`cargo machete`, `cargo deny check`, `cargo audit`), commit-message linting
(`committed`), and the test suite (`cargo nextest`).

## Commit messages

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
and are linted by [`committed`](https://github.com/crate-ci/committed) (see
[`committed.toml`](committed.toml)). Release notes are generated from these
messages by [`git-cliff`](https://git-cliff.org/) (see [`cliff.toml`](cliff.toml)).

## Releasing

Releases are cut by running the `Release` GitHub Actions workflow with a
`version` input (e.g. `v0.2.0`); it creates the tag, drafts the GitHub
release, and builds the per-platform archives and `.mcpb` bundles. See
[`.github/workflows/release.yml`](.github/workflows/release.yml).
