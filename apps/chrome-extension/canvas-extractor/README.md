# Canvas Payload Parser

A Rust CLI tool that reads a Canvas LMS course payload JSON file, downloads course materials, extracts grades, identifies video links, and provides detailed logging.

## Features

- Parse Canvas LMS course payload JSON files
- Download course materials to organized directories
- Extract student grades
- Identify and extract video links
- Comprehensive logging of all operations
- Structured JSON output

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run -- [OPTIONS] [INPUT_FILE]
```

## Project Structure

- `src/main.rs` - Entry point and CLI argument handling
- `src/models.rs` - Data structures for Canvas payload
- `src/parser.rs` - JSON payload parsing
- `src/downloader.rs` - File download functionality
- `src/logger.rs` - Logging configuration
- `src/grades.rs` - Grade extraction
- `src/videos.rs` - Video link extraction

## Dependencies

- `serde`/`serde_json` - JSON serialization/deserialization
- `reqwest` - HTTP client for downloads
- `tokio` - Async runtime
- `clap` - CLI argument parsing
- `log`/`env_logger` - Logging
