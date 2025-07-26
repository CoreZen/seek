pub mod cli;
pub mod matchers;
pub mod search;
pub mod ui;

// Re-export common types
pub use matchers::EntryMatcher;
pub use search::{SearchResult, StatusMessage};
