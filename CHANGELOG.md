# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.3.0] - 2024-02-14

### Added
- `--fake-github-cli` flag to use a fake/mock GitHub Cli instead of the real one, this allows more intelligent tests without making any API calls at all

## [0.2.1] - 2024-02-13

### Added
- Compression/decompression of GitHuc CLI blob to get under the crates.io 10 MiB upload limit.

## [0.2.0] - 2024-02-13

### Changed
- No longer depend on external GitHub CLI installation. The newest GitHub CLI version is instead including in the binary and the first run will write the GitHub CLI to the host and use that binary directly.

## [0.1.1] - 2024-02-13

### Changed

Added check for GitHub CLI version

## [0.1.0] - 2024-02-13

### Added

- automatic issue creation