[private]
@default:
    just --list

# Needs the rust toolchain
env:
    rustc --version
    cargo --version

# Run the tests
test:
    cargo test

# Build the application
build *ARGS:
    cargo build {{ ARGS }}


# Run the application (use `--` to pass arguments to the application)
run *ARGS:
    cargo run {{ ARGS }}

# Clean the `target` directory
clean:
    cargo clean

# Build the documentation (use `--open` to open in the browser)
doc *ARGS:
    cargo doc {{ ARGS }}