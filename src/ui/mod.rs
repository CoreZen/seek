use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use crate::search::{SearchResult, StatusMessage};

/// Display manager for search progress and results
pub struct DisplayManager {
    spinner: ProgressBar,
    current_path: String,
    file_count: usize,
    found_count: usize,
    permission_errors: usize,
    max_files: usize,
}

impl DisplayManager {
    /// Create a new display manager with a spinner
    pub fn new() -> Self {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋ ", "⠙ ", "⠹ ", "⠸ ", "⠼ ", "⠴ ", "⠦ ", "⠧ ", "⠇ ", "⠏ "]),
        );
        spinner.enable_steady_tick(Duration::from_millis(80));
        spinner.set_message("Starting search...");

        DisplayManager {
            spinner,
            current_path: String::from("..."),
            file_count: 0,
            found_count: 0,
            permission_errors: 0,
            max_files: 0,
        }
    }

    /// Process and display search results in real-time
    pub fn process_results(
        &mut self,
        result_rx: Receiver<PathBuf>,
        status_rx: Receiver<StatusMessage>,
    ) -> (usize, usize, usize, bool, bool, u64) {
        let start_time = std::time::Instant::now();
        let mut limit_reached = false;
        let mut timed_out = false;
        let mut _timeout_seconds = 0;
        let mut last_update = std::time::Instant::now();
        let mut received_initial_message = false;
        let update_interval = Duration::from_millis(100);

        loop {
            let now = std::time::Instant::now();
            let mut updated = false;

            // Process status messages first to get context
            match status_rx.try_recv() {
                Ok(msg) => {
                    received_initial_message = true;
                    updated = true;
                    match msg {
                        StatusMessage::CurrentPath(path) => {
                            self.current_path = path;
                            self.update_spinner_message();
                        }
                        StatusMessage::FileCount(count, max) => {
                            self.file_count = count;
                            self.max_files = max;
                            self.update_spinner_message();
                        }
                        StatusMessage::PermissionErrors(count) => {
                            self.permission_errors = count;
                            self.update_spinner_message();
                        }
                        StatusMessage::Timeout(seconds) => {
                            timed_out = true;
                            _timeout_seconds = seconds;
                            self.spinner.set_message(format!(
                                "Search timed out after {} seconds! ({} scanned, {} found)",
                                seconds, self.file_count, self.found_count
                            ));
                        }
                        StatusMessage::LimitReached(limit) => {
                            limit_reached = true;
                            self.spinner.set_message(format!(
                                "File limit reached ({})! Finishing search...",
                                limit
                            ));
                        }
                        StatusMessage::Done => {
                            break;
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    // No status messages, continue to results
                }
                Err(TryRecvError::Disconnected) => {
                    if received_initial_message {
                        // Status channel closed, exit loop
                        break;
                    }
                }
            }

            // Check for new results to print (non-blocking)
            let mut result_count = 0;
            while result_count < 10 {
                // Process up to 10 results at once
                match result_rx.try_recv() {
                    Ok(path) => {
                        self.found_count += 1;
                        result_count += 1;
                        updated = true;

                        // Pause spinner to print result
                        self.spinner.suspend(|| {
                            println!("{}", path.display().to_string().green());
                        });

                        // Update spinner after the first match or periodically
                        if self.found_count == 1 || self.found_count % 5 == 0 {
                            self.update_spinner_message();
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        // No more results right now
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        // Result channel closed - will be handled below
                        if received_initial_message && self.found_count > 0 {
                            break;
                        }
                    }
                }
            }

            // If we haven't received any updates for a while, show a spinner update
            if !updated && now.duration_since(last_update) > update_interval {
                if received_initial_message {
                    self.update_spinner_message();
                } else if self.current_path == "..." {
                    self.spinner.set_message("Preparing search...");
                }
                last_update = now;
            }

            // Brief sleep to avoid high CPU usage
            std::thread::sleep(Duration::from_millis(10));
        }

        (
            self.found_count,
            self.file_count,
            self.permission_errors,
            limit_reached,
            timed_out,
            start_time.elapsed().as_secs(),
        )
    }

    /// Update the spinner message with current stats
    fn update_spinner_message(&self) {
        // Special case for when we've found something to make it immediately visible
        if self.found_count > 0 && self.found_count < 5 {
            self.spinner.set_message(format!(
                "Found {} match{}! Continuing search...",
                self.found_count,
                if self.found_count == 1 { "" } else { "es" }
            ));
            return;
        }

        let remaining = if self.max_files > 0 && self.max_files > self.file_count {
            format!(", {} remaining", self.max_files - self.file_count)
        } else {
            String::new()
        };

        let permission_msg = if self.permission_errors > 0 {
            format!(", {} permission errors", self.permission_errors)
        } else {
            String::new()
        };

        let count_msg = if self.file_count > 0 {
            format!("{} scanned", self.file_count)
        } else {
            "searching".to_string()
        };

        let found_msg = if self.found_count > 0 {
            format!(", {} found", self.found_count)
        } else {
            "".to_string()
        };

        self.spinner.set_message(format!(
            "Searching in: {} ({}{}{}{})",
            self.current_path, count_msg, found_msg, permission_msg, remaining
        ));
    }

    /// Complete the search and show final results
    pub fn finish(&self, result: &SearchResult, base_path: &PathBuf) {
        // Calculate elapsed time (ensure it's never zero to avoid confusion)
        let elapsed = if result.elapsed.as_secs_f64() < 0.1 {
            std::time::Duration::from_millis(100) // Minimum display time of 0.1s
        } else {
            result.elapsed
        };
        let match_text = if result.matches == 0 {
            "No matches found".to_string()
        } else if result.matches == 1 {
            "Found 1 match".to_string()
        } else {
            format!("Found {} matches", result.matches)
        };

        if result.timed_out {
            self.spinner.finish_with_message(format!(
                "Search timed out after {:.1}s! {} in {} (scanned {} files{})",
                elapsed.as_secs_f64(),
                match_text,
                base_path.display(),
                result.files_scanned,
                if result.permission_errors > 0 {
                    format!(", {} permission errors", result.permission_errors)
                } else {
                    String::new()
                }
            ));
        } else if result.limit_reached {
            self.spinner.finish_with_message(format!(
                "Search stopped at file limit! {} in {} ({:.1}s{})",
                match_text,
                base_path.display(),
                elapsed.as_secs_f64(),
                if result.permission_errors > 0 {
                    format!(", {} permission errors", result.permission_errors)
                } else {
                    String::new()
                }
            ));
        } else {
            self.spinner.finish_with_message(format!(
                "Search complete! {} in {} ({:.1}s, {} files{})",
                match_text,
                base_path.display(),
                elapsed.as_secs_f64(),
                result.files_scanned,
                if result.permission_errors > 0 {
                    format!(", {} permission errors", result.permission_errors)
                } else {
                    String::new()
                }
            ));
        }
    }

    /// Get the spinner for advanced operations
    pub fn spinner(&self) -> &ProgressBar {
        &self.spinner
    }
}
