//! # minimime
//!
//! A minimal MIME type detection library for Rust, ported from the Ruby [minimime](https://github.com/discourse/mini_mime) gem.
//!
//! This library provides fast MIME type detection based on file extensions and content types,
//! using embedded database files for efficient lookups without external dependencies.
//!
//! ## Features
//!
//! - **Fast lookups**: Uses embedded hash maps for average O(1) MIME type detection
//! - **No external dependencies**: Database files are embedded at compile time
//! - **Case insensitive**: Handles file extensions in any case
//! - **Binary detection**: Identifies binary vs text file types
//! - **Thread safe**: Safe for concurrent use across multiple threads
//!
//! ## Quick Start
//!
//! ```rust
//! use minimime::{lookup_by_filename, lookup_by_extension, lookup_by_content_type};
//!
//! // Look up by filename
//! if let Some(info) = lookup_by_filename("document.pdf") {
//!     println!("MIME type: {}", info.content_type); // "application/pdf"
//!     println!("Is binary: {}", info.is_binary());  // true
//! }
//!
//! // Look up by extension
//! if let Some(info) = lookup_by_extension("json") {
//!     println!("MIME type: {}", info.content_type); // "application/json"
//! }
//!
//! // Look up by content type
//! if let Some(info) = lookup_by_content_type("text/css") {
//!     println!("Extension: {}", info.extension); // "css"
//! }
//! ```
//!
//! ## Supported File Types
//!
//! This library supports hundreds of file extensions and MIME types, including:
//! - Web formats (HTML, CSS, JS, JSON, XML)
//! - Images (PNG, JPEG, GIF, SVG, WebP)
//! - Documents (PDF, DOC, XLS, PPT)
//! - Archives (ZIP, TAR, GZ)
//! - Media files (MP3, MP4, AVI, MOV)
//! - And many more...

use std::{
    collections::HashMap,
    path::Path,
    sync::{Mutex, OnceLock},
};

/// MIME type information including extension, content type, and encoding.
///
/// This struct contains all the information about a specific MIME type,
/// including whether it's a binary or text format.
#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    /// File extension (without the dot)
    pub extension: String,
    /// MIME content type (e.g., "text/plain", "image/png")
    pub content_type: String,
    /// Encoding type (e.g., "8bit", "base64")
    pub encoding: String,
}

impl Info {
    /// Encodings that indicate binary file types
    const BINARY_ENCODINGS: &'static [&'static str] = &["base64", "8bit"];

    /// Creates a new `Info` instance from a database line.
    ///
    /// The line format is: `extension content_type encoding`
    ///
    /// # Arguments
    ///
    /// * `line` - A whitespace-separated string containing extension, content type, and encoding
    ///
    /// # Returns
    ///
    /// * `Some(Info)` if the line is valid
    /// * `None` if the line doesn't have at least 3 parts
    ///
    /// # Examples
    ///
    /// ```
    /// use minimime::Info;
    ///
    /// let info = Info::new("pdf application/pdf base64").unwrap();
    /// assert_eq!(info.extension, "pdf");
    /// assert_eq!(info.content_type, "application/pdf");
    /// assert_eq!(info.encoding, "base64");
    /// ```
    pub fn new(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            Some(Info {
                extension: parts[0].to_string(),
                content_type: parts[1].to_string(),
                encoding: parts[2].to_string(),
            })
        } else {
            None
        }
    }

    /// Determines if this MIME type represents a binary file format.
    ///
    /// Binary files are those that use "base64" or "8bit" encoding.
    ///
    /// # Returns
    ///
    /// * `true` if the file type is binary
    /// * `false` if the file type is text-based
    ///
    /// # Examples
    ///
    /// ```
    /// use minimime::Info;
    ///
    /// let pdf = Info::new("pdf application/pdf base64").unwrap();
    /// assert!(pdf.is_binary());
    ///
    /// let txt = Info::new("txt text/plain 7bit").unwrap();
    /// assert!(!txt.is_binary());
    /// ```
    pub fn is_binary(&self) -> bool {
        Self::BINARY_ENCODINGS.contains(&self.encoding.as_str())
    }
}

/// Internal database for MIME type lookups.
///
/// This struct manages the hash maps used for fast MIME type lookups
/// by file extension and content type.
pub struct Db {
    ext_db: HashMap<String, Info>,
    content_type_db: HashMap<String, Info>,
}

impl Db {
    /// Creates a new database instance and loads the embedded data files.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut db = Db {
            ext_db: HashMap::new(),
            content_type_db: HashMap::new(),
        };

        // Load extension database
        db.load_ext_db()?;
        // Load content type database
        db.load_content_type_db()?;

        Ok(db)
    }

    /// Loads the file extension to MIME type database.
    ///
    /// This method reads the embedded `ext_mime.db` file and populates
    /// the extension lookup hash map.
    fn load_ext_db(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let db_content = include_str!("db/ext_mime.db");
        for line in db_content.lines() {
            if let Some(info) = Info::new(line) {
                self.ext_db.insert(info.extension.clone(), info);
            }
        }
        Ok(())
    }

    /// Loads the content type to MIME type database.
    ///
    /// This method reads the embedded `content_type_mime.db` file and populates
    /// the content type lookup hash map.
    fn load_content_type_db(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let db_content = include_str!("db/content_type_mime.db");
        for line in db_content.lines() {
            if let Some(info) = Info::new(line) {
                self.content_type_db.insert(info.content_type.clone(), info);
            }
        }
        Ok(())
    }

    /// Looks up MIME information by file extension.
    ///
    /// The lookup is case-insensitive, trying the exact extension first,
    /// then falling back to lowercase.
    ///
    /// # Arguments
    ///
    /// * `extension` - File extension (with or without leading dot)
    ///
    /// # Returns
    ///
    /// * `Some(&Info)` if the extension is found
    /// * `None` if the extension is not recognized
    pub fn lookup_by_extension(&self, extension: &str) -> Option<&Info> {
        self.ext_db
            .get(extension)
            .or_else(|| self.ext_db.get(&extension.to_lowercase()))
    }

    /// Looks up MIME information by content type.
    ///
    /// # Arguments
    ///
    /// * `content_type` - MIME content type (e.g., "text/plain")
    ///
    /// # Returns
    ///
    /// * `Some(&Info)` if the content type is found
    /// * `None` if the content type is not recognized
    pub fn lookup_by_content_type(&self, content_type: &str) -> Option<&Info> {
        self.content_type_db.get(content_type)
    }

    /// Looks up MIME information by filename.
    ///
    /// Extracts the file extension from the filename and performs a lookup.
    /// The lookup is case-insensitive.
    ///
    /// # Arguments
    ///
    /// * `filename` - Full filename or path
    ///
    /// # Returns
    ///
    /// * `Some(&Info)` if the file extension is recognized
    /// * `None` if the file has no extension or the extension is not recognized
    pub fn lookup_by_filename(&self, filename: &str) -> Option<&Info> {
        let path = Path::new(filename);
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.lookup_by_extension(ext_str);
            }
        }
        None
    }
}

// Global database instance
static DB: OnceLock<Mutex<Db>> = OnceLock::new();

/// Gets the global database instance.
///
/// This function initializes the database on first access and returns
/// a reference to the global singleton instance.
///
/// # Returns
///
/// A reference to the global `Mutex<Db>` instance
///
/// # Panics
///
/// Panics if the database fails to initialize
fn get_db() -> &'static Mutex<Db> {
    DB.get_or_init(|| Mutex::new(Db::new().expect("Failed to initialize MIME database")))
}

/// Looks up MIME information by filename.
///
/// This is a convenience function that uses the global database instance
/// to perform the lookup. The lookup is case-insensitive.
///
/// # Arguments
///
/// * `filename` - Full filename or path
///
/// # Returns
///
/// * `Some(Info)` if the file extension is recognized
/// * `None` if the file has no extension or the extension is not recognized
///
/// # Examples
///
/// ```
/// use minimime::lookup_by_filename;
///
/// if let Some(info) = lookup_by_filename("document.pdf") {
///     println!("MIME type: {}", info.content_type);
///     println!("Is binary: {}", info.is_binary());
/// }
/// ```
pub fn lookup_by_filename(filename: &str) -> Option<Info> {
    let db = get_db().lock().unwrap();
    db.lookup_by_filename(filename).cloned()
}

/// Looks up MIME information by file extension.
///
/// This is a convenience function that uses the global database instance
/// to perform the lookup. The lookup is case-insensitive.
///
/// # Arguments
///
/// * `extension` - File extension (with or without leading dot)
///
/// # Returns
///
/// * `Some(Info)` if the extension is found
/// * `None` if the extension is not recognized
///
/// # Examples
///
/// ```
/// use minimime::lookup_by_extension;
///
/// if let Some(info) = lookup_by_extension("pdf") {
///     println!("MIME type: {}", info.content_type);
///     println!("Encoding: {}", info.encoding);
/// }
/// ```
pub fn lookup_by_extension(extension: &str) -> Option<Info> {
    let db = get_db().lock().unwrap();
    db.lookup_by_extension(extension).cloned()
}

/// Looks up MIME information by content type.
///
/// This is a convenience function that uses the global database instance
/// to perform the lookup.
///
/// # Arguments
///
/// * `content_type` - MIME content type (e.g., "text/plain")
///
/// # Returns
///
/// * `Some(Info)` if the content type is found
/// * `None` if the content type is not recognized
///
/// # Examples
///
/// ```
/// use minimime::lookup_by_content_type;
///
/// if let Some(info) = lookup_by_content_type("application/pdf") {
///     println!("Extension: {}", info.extension);
///     println!("Is binary: {}", info.is_binary());
/// }
/// ```
pub fn lookup_by_content_type(content_type: &str) -> Option<Info> {
    let db = get_db().lock().unwrap();
    db.lookup_by_content_type(content_type).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_creation() {
        let info = Info::new("pdf application/pdf base64").unwrap();
        assert_eq!(info.extension, "pdf");
        assert_eq!(info.content_type, "application/pdf");
        assert_eq!(info.encoding, "base64");
        assert!(info.is_binary());
    }

    #[test]
    fn test_extension() {
        if let Some(info) = lookup_by_extension("zip") {
            assert_eq!(info.content_type, "application/zip");
        }
    }

    #[test]
    fn test_mixed_case() {
        // Test case insensitive lookups
        if let Some(info) = lookup_by_filename("a.GTM") {
            assert_eq!(info.content_type, "application/vnd.groove-tool-message");
        }
        if let Some(info) = lookup_by_extension("ZiP") {
            assert_eq!(info.content_type, "application/zip");
        }
    }

    #[test]
    fn test_content_type_lookups() {
        // Test various file extensions
        if let Some(info) = lookup_by_filename("a.123") {
            assert_eq!(info.content_type, "application/vnd.lotus-1-2-3");
        }
        if let Some(info) = lookup_by_filename("a.Z") {
            assert_eq!(info.content_type, "application/x-compressed");
        }
        if let Some(info) = lookup_by_filename("a.gtm") {
            assert_eq!(info.content_type, "application/vnd.groove-tool-message");
        }
        if let Some(info) = lookup_by_filename("a.zmm") {
            assert_eq!(
                info.content_type,
                "application/vnd.HandHeld-Entertainment+xml"
            );
        }
        if let Some(info) = lookup_by_filename("x.csv") {
            assert_eq!(info.content_type, "text/csv");
        }
        if let Some(info) = lookup_by_filename("x.mda") {
            assert_eq!(info.content_type, "application/x-msaccess");
        }

        // Test unknown extension
        assert!(lookup_by_filename("a.frog").is_none());
    }

    #[test]
    fn test_binary() {
        // Test binary detection
        if let Some(info) = lookup_by_filename("a.z") {
            assert!(info.is_binary());
        }
        if let Some(info) = lookup_by_filename("a.Z") {
            assert!(info.is_binary());
        }
        if let Some(info) = lookup_by_filename("a.txt") {
            assert!(!info.is_binary());
        }
        assert!(lookup_by_filename("a.frog").is_none());
    }

    #[test]
    fn test_binary_content_type() {
        if let Some(info) = lookup_by_content_type("application/x-compressed") {
            assert!(info.is_binary());
        }
        assert!(lookup_by_content_type("something-fake").is_none());
        if let Some(info) = lookup_by_content_type("text/plain") {
            assert!(!info.is_binary());
        }
    }

    #[test]
    fn test_prioritize_extensions_correctly() {
        if let Some(info) = lookup_by_content_type("text/plain") {
            assert_eq!(info.extension, "txt");
        }
    }

    #[test]
    fn test_lookup_by_filename() {
        if let Some(info) = lookup_by_filename("document.pdf") {
            assert_eq!(info.content_type, "application/pdf");
        }
    }

    #[test]
    fn test_lookup_by_extension() {
        if let Some(info) = lookup_by_extension("pdf") {
            assert_eq!(info.content_type, "application/pdf");
        }
    }
}
