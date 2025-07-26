use globset::{Glob, GlobMatcher};
use regex::Regex;

use walkdir::DirEntry;

/// A trait for matching file entries against patterns
pub trait EntryMatcher: Send + Sync {
    fn is_match(&self, entry: &DirEntry, full_path: bool) -> bool;
}

/// Glob-based matcher
pub struct GlobEntryMatcher {
    matcher: GlobMatcher,
}

impl GlobEntryMatcher {
    pub fn new(pattern: &str) -> Result<Self, globset::Error> {
        Ok(GlobEntryMatcher {
            matcher: Glob::new(pattern)?.compile_matcher(),
        })
    }
}

impl EntryMatcher for GlobEntryMatcher {
    fn is_match(&self, entry: &DirEntry, full_path: bool) -> bool {
        if full_path {
            self.matcher.is_match(entry.path())
        } else {
            self.matcher.is_match(entry.file_name())
        }
    }
}

/// Regex-based matcher
pub struct RegexEntryMatcher {
    regex: Regex,
}

impl RegexEntryMatcher {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(RegexEntryMatcher {
            regex: Regex::new(pattern)?,
        })
    }
}

impl EntryMatcher for RegexEntryMatcher {
    fn is_match(&self, entry: &DirEntry, full_path: bool) -> bool {
        let text = if full_path {
            entry.path().to_string_lossy()
        } else {
            entry.file_name().to_string_lossy()
        };
        self.regex.is_match(&text)
    }
}

/// Create a matcher based on the pattern type
pub fn create_matcher(pattern: &str, use_regex: bool) -> Result<Box<dyn EntryMatcher>, String> {
    if use_regex {
        match RegexEntryMatcher::new(pattern) {
            Ok(m) => Ok(Box::new(m)),
            Err(e) => Err(format!("Invalid regex pattern: {}", e)),
        }
    } else {
        match GlobEntryMatcher::new(pattern) {
            Ok(m) => Ok(Box::new(m)),
            Err(e) => Err(format!("Invalid glob pattern: {}", e)),
        }
    }
}

/// Helper function to determine if an entry should be processed based on file type filters
pub fn should_process_entry(entry: &DirEntry, files_only: bool, dirs_only: bool) -> bool {
    if files_only && !entry.file_type().is_file() {
        return false;
    }
    if dirs_only && !entry.file_type().is_dir() {
        return false;
    }
    true
}
