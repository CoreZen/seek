use crate::matchers::EntryMatcher;
use colored::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};
use walkdir::{DirEntry, WalkDir};

/// Message types for our channels
pub enum StatusMessage {
    CurrentPath(String),
    FileCount(usize, usize), // current count, total limit
    PermissionErrors(usize),
    Timeout(u64),
    LimitReached(usize),
    Done,
}

/// Result of a search operation
pub struct SearchResult {
    pub matches: usize,
    pub files_scanned: usize,
    pub permission_errors: usize,
    pub elapsed: Duration,
    pub limit_reached: bool,
    pub timed_out: bool,
}

/// Core search functionality
pub struct Searcher {
    matcher: Arc<dyn EntryMatcher>,
    base_path: Arc<PathBuf>,
    max_depth: Option<usize>,
    max_files: usize,
    timeout: Option<Duration>,
    start_time: Instant,
    files_only: bool,
    dirs_only: bool,
    show_permission_errors: bool,
}

impl Searcher {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        matcher: Box<dyn EntryMatcher>,
        base_path: PathBuf,
        max_depth: Option<usize>,
        max_files: usize,
        timeout_seconds: u64,
        files_only: bool,
        dirs_only: bool,
        show_permission_errors: bool,
    ) -> Self {
        let timeout = if timeout_seconds > 0 {
            Some(Duration::from_secs(timeout_seconds))
        } else {
            None
        };

        Searcher {
            matcher: Arc::from(matcher),
            base_path: Arc::new(base_path),
            max_depth,
            max_files,
            timeout,
            start_time: Instant::now(), // Record start time when searcher is created
            files_only,
            dirs_only,
            show_permission_errors,
        }
    }

    /// Performs the search operation
    pub fn search(
        &self,
        full_path: bool,
    ) -> (Receiver<PathBuf>, Receiver<StatusMessage>, SearchResult) {
        let (result_tx, result_rx) = mpsc::channel::<PathBuf>();
        let (status_tx, status_rx) = mpsc::channel::<StatusMessage>();

        // Send initial status message
        let _ = status_tx.send(StatusMessage::CurrentPath("Starting search...".to_string()));

        let matcher = Arc::clone(&self.matcher);
        let base_path = Arc::clone(&self.base_path);
        let max_depth = self.max_depth;
        let max_files = self.max_files;
        let timeout = self.timeout;
        let start_time = self.start_time;
        let files_only = self.files_only;
        let dirs_only = self.dirs_only;
        let show_permission_errors = self.show_permission_errors;

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        // Store start time for elapsed time calculation
        let search_start_time = self.start_time;

        // Spawn a thread to collect and process files
        let _search_thread = std::thread::spawn(move || {
            // Setup walker with optional depth limit
            let mut walker = WalkDir::new(base_path.as_path());
            if let Some(depth) = max_depth {
                walker = walker.max_depth(depth);
            }

            // Variables to track progress
            let mut file_count = 0;
            let mut permission_errors = 0;
            let mut limit_reached = false;
            let mut timed_out = false;
            let mut match_count = 0;

            // First pass: collect entries
            let mut entries: Vec<DirEntry> = Vec::new();

            // Iterate over files with early termination
            for result in walker.into_iter() {
                // Check for timeout
                if let Some(timeout_duration) = timeout {
                    if start_time.elapsed() > timeout_duration {
                        timed_out = true;
                        let _ = status_tx.send(StatusMessage::Timeout(timeout_duration.as_secs()));
                        break;
                    }
                }

                // Check file count limit
                file_count += 1;
                if max_files > 0 && file_count >= max_files {
                    limit_reached = true;
                    let _ = status_tx.send(StatusMessage::LimitReached(max_files));
                    break;
                }

                // Update counts periodically
                if file_count % 1000 == 0 || (max_files > 0 && max_files - file_count < 1000) {
                    let _ = status_tx.send(StatusMessage::FileCount(file_count, max_files));
                }

                // Process the entry
                match result {
                    Ok(entry) => {
                        // Update current directory for spinner
                        if entry.file_type().is_dir() {
                            let rel_path = entry
                                .path()
                                .strip_prefix(base_path.as_path())
                                .unwrap_or(entry.path());
                            let path_str = rel_path.to_string_lossy().to_string();
                            if !path_str.is_empty() && file_count % 100 == 0 {
                                let _ = status_tx.send(StatusMessage::CurrentPath(path_str));
                            }
                        }

                        // Apply file type filters
                        if crate::matchers::should_process_entry(&entry, files_only, dirs_only) {
                            entries.push(entry);
                        }
                    }
                    Err(err) => {
                        // Handle permission errors
                        if let Some(path) = err.path() {
                            if let Some(io_err) = err.io_error() {
                                if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                                    permission_errors += 1;
                                    if show_permission_errors {
                                        let _ =
                                            status_tx.send(StatusMessage::CurrentPath(format!(
                                                "Permission denied: {}",
                                                path.to_string_lossy()
                                            )));
                                    }

                                    if permission_errors % 10 == 0 {
                                        let _ = status_tx.send(StatusMessage::PermissionErrors(
                                            permission_errors,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Send final collection stats
            let _ = status_tx.send(StatusMessage::FileCount(file_count, max_files));
            let _ = status_tx.send(StatusMessage::PermissionErrors(permission_errors));

            // Second phase: Process entries
            let _ = status_tx.send(StatusMessage::CurrentPath(format!(
                "Searching {} files...",
                entries.len()
            )));

            // Process entries
            for entry in &entries {
                if limit_reached || timed_out {
                    break;
                }

                // Check for timeout
                if let Some(timeout_duration) = timeout {
                    if start_time.elapsed() > timeout_duration {
                        timed_out = true;
                        break;
                    }
                }

                // Process entry
                if matcher.is_match(entry, full_path) {
                    match_count += 1;
                    counter_clone.fetch_add(1, Ordering::Relaxed);

                    // Send match immediately for display
                    let _ = result_tx.send(entry.path().to_path_buf());

                    // Update the counter periodically
                    if match_count % 10 == 0 {
                        let _ = status_tx.send(StatusMessage::CurrentPath(format!(
                            "Found {match_count} matches so far..."
                        )));
                    }
                }
            }

            // Signal completion
            let _ = status_tx.send(StatusMessage::Done);

            // Return search results with properly calculated elapsed time
            SearchResult {
                matches: match_count,
                files_scanned: file_count,
                permission_errors,
                elapsed: search_start_time.elapsed(), // Use the search start time for elapsed calculation
                limit_reached,
                timed_out,
            }
        });

        // Don't wait for the search thread to complete - we want results to stream immediately
        // But create an initial result with the correct start time for elapsed calculation
        let empty_result = SearchResult {
            matches: 0,
            files_scanned: 0,
            permission_errors: 0,
            elapsed: self.start_time.elapsed(), // Use actual elapsed time
            limit_reached: false,
            timed_out: false,
        };

        (result_rx, status_rx, empty_result)
    }

    /// Helper function to print sudo suggestion if needed
    pub fn print_permission_hint(permission_errors: usize, path: &str, pattern: &str) {
        if permission_errors > 5 {
            println!(
                "\n{}",
                "Hint: Many permission errors encountered. Try running with sudo:".yellow()
            );
            println!(
                "      {}",
                format!("sudo seek \"{path}\" \"{pattern}\"").yellow()
            );

            if cfg!(target_os = "macos") {
                println!("\n{}", "On macOS, some directories may still be restricted due to System Integrity Protection.".yellow());
                println!("{}", "For searching user data directories, you may need to grant Terminal 'Full Disk Access'".yellow());
                println!(
                    "{}",
                    "in System Preferences → Privacy & Security → Full Disk Access.".yellow()
                );
            }
        }
    }
}
