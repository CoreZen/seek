use clap::Parser;
use std::path::Path;

/// Seek - A fast file search tool
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Seek: Search files using glob or regex.\n\n\
Usage:\n  seek <PATH> <PATTERN>          (glob by default)\n  seek <PATH> <PATTERN> -r         (regex mode)"
)]
pub struct Args {
    /// Enable regex mode instead of glob
    #[arg(short = 'r', long = "regex")]
    pub regex: bool,

    /// Search full path instead of just filename
    #[arg(short = 'p', long = "path")]
    pub full_path: bool,

    /// Only show files (not directories)
    #[arg(short = 'f', long = "files-only")]
    pub files_only: bool,

    /// Only show directories (not files)
    #[arg(short = 'd', long = "dirs-only")]
    pub dirs_only: bool,

    /// Maximum search depth
    #[arg(short = 'D', long = "max-depth")]
    pub max_depth: Option<usize>,

    /// Show permission errors (they're automatically skipped)
    #[arg(short = 'e', long = "show-permission-errors")]
    pub show_permission_errors: bool,

    /// Maximum number of files to scan (0 = unlimited)
    #[arg(short = 'n', long = "max-files", default_value = "500000")]
    pub max_files: usize,

    /// Search timeout in seconds (0 = no timeout)
    #[arg(short = 't', long = "timeout", default_value = "600")]
    pub timeout_seconds: u64,

    /// Path to search in (default: current dir if only pattern given)
    #[arg(index = 1)]
    pub path_or_pattern: String,

    /// Pattern to search for (required if path is given)
    #[arg(index = 2)]
    pub maybe_pattern: Option<String>,
}

impl Args {
    /// Parse command line arguments and resolve the path and pattern
    pub fn parse_args() -> (Self, String, String) {
        let args = Self::parse();
        let (path, pattern) = match &args.maybe_pattern {
            Some(pat) => (args.path_or_pattern.clone(), pat.clone()),
            None => {
                let path_str = &args.path_or_pattern;
                let path = Path::new(path_str);
                if path.exists() && path.is_dir() {
                    (path_str.clone(), "*".to_string())
                } else {
                    (".".to_string(), path_str.clone())
                }
            }
        };

        (args, path, pattern)
    }
}
