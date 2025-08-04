# minimime

[![Crates.io](https://img.shields.io/crates/v/minimime.svg)](https://crates.io/crates/minimime)
[![Documentation](https://docs.rs/minimime/badge.svg)](https://docs.rs/minimime)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A minimal MIME type detection library for Rust, ported from the Ruby [minimime](https://github.com/discourse/mini_mime) gem.

This library provides fast MIME type detection based on file extensions and content types, using embedded database files for efficient lookups without external dependencies.

## Features

- **Fast lookups**: Uses embedded hash maps for O(1) MIME type detection
- **No external dependencies**: Database files are embedded at compile time
- **Case insensitive**: Handles file extensions in any case
- **Binary detection**: Identifies binary vs text file types
- **Thread safe**: Safe for concurrent use across multiple threads

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
minimime = "1.0.0"
```

## Quick Start

```rust
use minimime::{lookup_by_filename, lookup_by_extension, lookup_by_content_type};

// Look up by filename
if let Some(info) = lookup_by_filename("document.pdf") {
    println!("MIME type: {}", info.content_type); // "application/pdf"
    println!("Is binary: {}", info.is_binary());  // true
}

// Look up by extension
if let Some(info) = lookup_by_extension("json") {
    println!("MIME type: {}", info.content_type); // "application/json"
}

// Look up by content type
if let Some(info) = lookup_by_content_type("text/css") {
    println!("Extension: {}", info.extension); // "css"
}
```

## API Reference

- `lookup_by_filename(filename: &str) -> Option<Info>` - Look up MIME type by filename
- `lookup_by_extension(extension: &str) -> Option<Info>` - Look up MIME type by file extension  
- `lookup_by_content_type(content_type: &str) -> Option<Info>` - Look up by MIME content type

Each function returns an `Info` struct containing:
- `extension` - File extension (without dot)
- `content_type` - MIME content type
- `encoding` - Encoding type
- `is_binary()` - Whether the file type is binary

## Supported File Types

This library supports hundreds of file extensions and MIME types, including:

- **Web formats**: HTML, CSS, JS, JSON, XML
- **Images**: PNG, JPEG, GIF, SVG, WebP, BMP, ICO
- **Documents**: PDF, DOC, DOCX, XLS, XLSX, PPT, PPTX
- **Archives**: ZIP, TAR, GZ, RAR, 7Z
- **Media files**: MP3, MP4, AVI, MOV, WAV, FLAC
- **Programming**: RS, PY, JS, TS, GO, C, CPP, H
- And many more...

## Examples

### Basic Usage

```rust
use minimime::lookup_by_filename;

fn main() {
    // Check different file types
    let files = vec![
        "document.pdf",
        "image.png", 
        "script.js",
        "data.json",
        "archive.zip"
    ];
    
    for filename in files {
        if let Some(info) = lookup_by_filename(filename) {
            println!(
                "{}: {} ({})", 
                filename, 
                info.content_type,
                if info.is_binary() { "binary" } else { "text" }
            );
        }
    }
}
```

### Web Server Integration

```rust
use minimime::lookup_by_filename;

fn serve_file(filename: &str) -> String {
    let mime_type = lookup_by_filename(filename)
        .map(|info| info.content_type)
        .unwrap_or("application/octet-stream".to_string());
    
    format!("Content-Type: {}", mime_type)
}

fn main() {
    println!("{}", serve_file("index.html"));  // Content-Type: text/html
    println!("{}", serve_file("style.css"));   // Content-Type: text/css
    println!("{}", serve_file("app.js"));      // Content-Type: application/javascript
}
```

## Performance

The library uses embedded hash maps for fast lookups, making it extremely efficient:

- Average case O(1) lookups with amortized performance
- No file system access required
- Thread-safe for concurrent usage
- Minimal memory footprint

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Ported from the Ruby [minimime](https://github.com/discourse/mini_mime) gem
- Inspired by the need for a fast, dependency-free MIME detection library in Rust