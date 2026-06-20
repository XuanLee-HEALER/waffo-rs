# waffo-rs task runner. Run `just` (or `just --list`) to see all recipes.

# On Windows, run recipes with PowerShell 7 (other OSes keep the default `sh`).
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]

# The proc-macro crate runs at *compile* time, so runtime coverage can't measure
# it — exclude it from coverage so it doesn't skew the denominator.
cov_ignore := "waffo-rs-derive"
# Minimum line coverage the `cov` gate enforces.
cov_min := "80"
# Where HTML coverage reports are written (under target/, already gitignored).
cov_dir := "target/llvm-cov"

# List available recipes.
default:
    @just --list

# ---- format / lint / test --------------------------------------------------

# Format all code in place.
fmt:
    cargo fmt --all

# Check formatting without modifying files.
fmt-check:
    cargo fmt --all --check

# Lint with the strict clippy gate (deny warnings on top of the workspace lints).
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run the test suite (all features). Excludes the ignored sandbox e2e tests.
test:
    cargo test --all-features

# Run the sandbox end-to-end tests. Needs WAFFO_* credentials in the env
# (WAFFO_API_KEY / WAFFO_PRIVATE_KEY / WAFFO_PUBLIC_KEY [+ WAFFO_MERCHANT_ID,
# WAFFO_ENVIRONMENT=SANDBOX]).
e2e:
    cargo test --test e2e -- --ignored --nocapture

# Build the whole workspace (all features).
build:
    cargo build --all-features

# Fast quality gate: formatting + lint + tests. (Coverage is gated separately.)
check: fmt-check lint test

# ---- coverage (cargo-llvm-cov) ---------------------------------------------

# Coverage gate: generate the HTML report AND enforce the {{cov_min}}% line floor.
cov:
    cargo llvm-cov --all-features --ignore-filename-regex "{{cov_ignore}}" --html --output-dir {{cov_dir}} --fail-under-lines {{cov_min}}

# Coverage HTML report only (no threshold), written to {{cov_dir}}/html/index.html.
cov-report:
    cargo llvm-cov --all-features --ignore-filename-regex "{{cov_ignore}}" --html --output-dir {{cov_dir}}

# Print a coverage summary table to the terminal.
cov-summary:
    cargo llvm-cov --all-features --ignore-filename-regex "{{cov_ignore}}" --summary-only

# Generate the HTML report and open it in a browser.
cov-open:
    cargo llvm-cov --all-features --ignore-filename-regex "{{cov_ignore}}" --open --output-dir {{cov_dir}}

# Export coverage as lcov (for later CI / Codecov use).
cov-lcov:
    cargo llvm-cov --all-features --ignore-filename-regex "{{cov_ignore}}" --lcov --output-path {{cov_dir}}/lcov.info

# ---- misc ------------------------------------------------------------------

# Build the API docs.
doc:
    cargo doc --all-features --no-deps

# Remove build and coverage artifacts.
clean:
    cargo clean
