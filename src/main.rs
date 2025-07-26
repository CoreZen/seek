use std::path::PathBuf;
use std::process;

use seek::cli::Args;
use seek::matchers;
use seek::search::Searcher;
use seek::ui::DisplayManager;

fn main() {
    // Parse command line arguments
    let (args, path, pattern) = Args::parse_args();

    // Create the base path
    let base_path = PathBuf::from(&path);

    // Create the appropriate matcher
    let matcher = match matchers::create_matcher(&pattern, args.regex) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Record search start time
    let start_time = std::time::Instant::now();

    // Create the searcher
    let searcher = Searcher::new(
        matcher,
        base_path.clone(),
        args.max_depth,
        args.max_files,
        args.timeout_seconds,
        args.files_only,
        args.dirs_only,
        args.show_permission_errors,
    );

    // Create the display manager
    let mut display = DisplayManager::new();

    // Start the search
    let (result_rx, status_rx, _) = searcher.search(args.full_path);

    // Process and display results in real-time
    let (found_count, file_count, permission_errors, limit_reached, timed_out, _) =
        display.process_results(result_rx, status_rx);

    // Create the final search result with proper elapsed time
    let result = seek::search::SearchResult {
        matches: found_count,
        files_scanned: file_count,
        permission_errors,
        elapsed: start_time.elapsed(), // Use actual elapsed time since search started
        limit_reached,
        timed_out,
    };

    // Show final results
    display.finish(&result, &base_path);

    // Show permission hints if needed
    Searcher::print_permission_hint(permission_errors, &path, &pattern);
}
