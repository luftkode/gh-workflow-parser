# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.4] - 2024-03-05
### Fix
- [#9 - Large amount of timestamps bypasses duplicate check](https://github.com/luftkode/gh-workflow-parser/issues/9#issue-2158319812) fixed in [!12](https://github.com/luftkode/gh-workflow-parser/pull/12)

## [0.5.3] - 2024-02-18

### Changed
- Included logs in a created issue are now in a markdown code block
- Canonicalize repo URL to guarantee valids HTTP links in created issue markdown

## [0.5.2] - 2024-02-16

### Changed
- Fixed issue with internal function for parsing yocto error summaries

## [0.5.1] - 2024-02-15

### Changed
- Locating the more specific "failure log" from a yocto build failure now uses the same algorithm as `locate-failure-log` when searching for the log to attach to an issue


## [0.5.0] - 2024-02-15

### Added
- the release profile `release-ci` to speed up compilation times in use-cases where re-compilation occurs frequently (use `cargo install gh-workflow-prase --profile release-ci`)

## [0.4.0] - 2024-02-15

### Added
- `locate-failure-log` subcommand to locate specific logs that adds details about why a test/build/deployment failed (currently only support `--kind=yocto`)

### Changed
- The `--repo` flag is no longer a global flag, it is required depending on the used subcommand

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
