[private]
@default:
    just --list

# Needs the rust toolchain
env:
    rustc --version
    cargo --version

# Lint the code
lint:
    cargo clippy

# Format the code
format:
    cargo fmt

# Check if it compiles without compiling
check:
    cargo check

# Run the tests
test:
    cargo test

# Build the application
build *ARGS:
    cargo build {{ ARGS }}


# Run the application (use `--` to pass arguments to the application)
run ARGS:
    cargo run {{ ARGS }}

# Clean the `target` directory
clean:
    cargo clean

# Build the documentation (use `--open` to open in the browser)
doc *ARGS:
    cargo doc {{ ARGS }}

# Publish the crate
publish:
    cargo publish

# List the dependencies
deps:
    cargo tree

# Update the dependencies
update:
    cargo update

# Run Full checks and format
full-check: lint format check test