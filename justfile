# https://just.systems

# Run every check CI runs, in the same order, so a green `just precommit`
# locally means a green CI run.
precommit: format lint dependencies commits test

format:
    cargo fmt --check
    taplo check

lint:
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings

dependencies:
    cargo machete
    cargo deny check
    cargo audit

commits:
    committed origin/master..HEAD

test profile='fast':
    cargo nextest run --no-fail-fast --cargo-profile {{ profile }}

# Apply every automatic fix available, then re-check.
fix:
    cargo clippy --fix --allow-dirty
    cargo fmt
    taplo format

changelog:
    git-cliff -o CHANGELOG.md --latest --strip all

# One-time local setup: install every tool `precommit` needs.
setup:
    cargo install --locked just committed cargo-deny cargo-nextest cargo-audit
    cargo install --locked taplo-cli cargo-machete
    cargo fetch --locked
