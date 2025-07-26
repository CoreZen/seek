# Seek

A blazingly fast file search utility written in Rust with a friendly, animated interface.

## Features

- **Fast Parallel Searches**: Uses multi-threading to speed up file searches in large directories
- **Live Search Animation**: Shows real-time progress with an animated spinner while searching
- **Multiple Search Patterns**:
  - **Glob Patterns** (default): Use wildcards like `*.rs` or `**/*.json`
  - **Regex Patterns**: Use powerful regular expressions with the `-r` flag
- **Flexible Search Options**:
  - Search in filenames or full paths (`-p/--path` flag)
  - Filter by file type (files-only with `-f` or directories-only with `-d`)
  - Control search depth with `-D/--max-depth`
- **Safety Features**:
  - File count limits to prevent excessive searches
  - Timeout mechanism to prevent hanging on large directories
  - Automatic permission error skipping with optional reporting
- **User Experience**:
  - Real-time directory tracking to see what's being searched
  - Live results printing as matches are found
  - Smart progress tracking with remaining file count display
  - Helpful suggestions when permission issues are encountered
- **Clean, Colored Output**: Results are displayed in a user-friendly format with live updates

## Installation

### Using Cargo (All Platforms)

If you have Rust installed, you can install Seek via Cargo directly from GitHub:

```bash
cargo install --git https://github.com/CoreZen/seek.git
```

Note: This project is not yet published on crates.io.

### Platform-Specific Installation

#### macOS

**Option 1: Using Homebrew**
```bash
# Add my tap
brew tap CoreZen/tap

# Install seek
brew install CoreZen/tap/seek
```

**Option 2: Manual Installation**
```bash
git clone https://github.com/CoreZen/seek.git
cd seek
cargo build --release
sudo cp target/release/seek /usr/local/bin/
```

#### Linux

**Ubuntu/Debian**
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build Seek
git clone https://github.com/CoreZen/seek.git
cd seek
cargo build --release
sudo cp target/release/seek /usr/local/bin/
```

**Fedora/RHEL/CentOS**
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build Seek
git clone https://github.com/CoreZen/seek.git
cd seek
cargo build --release
sudo cp target/release/seek /usr/local/bin/
```

**Arch Linux**
```bash
# Install Rust if needed
sudo pacman -S rust

# Clone and build Seek
git clone https://github.com/CoreZen/seek.git
cd seek
cargo build --release
sudo cp target/release/seek /usr/local/bin/
```

#### Windows

**Option 1: Using Cargo**
```powershell
cargo install --git https://github.com/CoreZen/seek.git
```

**Option 2: Manual Installation**
```powershell
# Clone the repository
git clone https://github.com/CoreZen/seek.git
cd seek

# Build the release binary
cargo build --release

# Copy to a location in your PATH (optional)
# For example, to your user's bin directory:
copy target\release\seek.exe C:\Users\YourUsername\bin\

# Or add to system PATH
# - Right-click "This PC" > Properties > Advanced system settings > Environment Variables
# - Add the path to the directory containing seek.exe
```

## Usage

### Basic Examples

```bash
# Search for all Rust files in current directory
seek "*.rs"

# Search in a specific directory
seek /path/to/dir "*.txt"

# Use regex mode
seek . "^README.*\.md$" -r

# Search full paths instead of just filenames
seek /src "test" -p

# Only search for directories
seek . "*config*" -d

# Only search for files with max depth of 3
seek /project "*.json" -f -D 3

# Show permission errors and set file limit
seek /usr "*.conf" -e -n 200000

# Set custom timeout for large directory searches
seek / "important.txt" -t 60
```

### Platform-Specific Examples

#### macOS

```bash
# Search for system configuration files
seek /etc "*.conf"

# Search your Documents folder for PDFs
seek ~/Documents "*.pdf"

# Search for application property lists
seek /Applications "*.plist"

# Search system directories (requires sudo)
sudo seek /System "*.plist"

# Search Time Machine backups (requires permissions)
seek /Volumes/TimeMachine "*.jpg"
```

#### Linux

```bash
# Search for configuration files
seek /etc "*.conf"

# Find all shell scripts in your home directory
seek ~ "*.sh"

# Search system logs (may require sudo)
sudo seek /var/log "*.log"

# Find executable files
seek /usr/bin -f

# Search mounted filesystems
seek /mnt "*.csv"
```

#### Windows

```powershell
# Search for text files in Documents folder
seek C:\Users\YourUsername\Documents "*.txt"

# Find all executable files in Program Files
seek "C:\Program Files" "*.exe"

# Search the Windows directory
seek C:\Windows "*.dll"

# Find configuration files
seek C:\ "*.config" -D 4

# Search for user data
seek "C:\Users" "*.dat" -p
```

## Command-Line Options

```
USAGE:
  seek [OPTIONS] [PATH] [PATTERN]

ARGS:
  <PATH>      Path to search in (default: current directory)
  <PATTERN>   Pattern to search for (glob by default)

OPTIONS:
  -r, --regex        Enable regex mode instead of glob
  -p, --path         Search full path instead of just filename
  -f, --files-only   Only show files (not directories)
  -d, --dirs-only    Only show directories (not files)
  -D, --max-depth <DEPTH>   Maximum search depth
  -e, --show-permission-errors   Show permission errors (skipped automatically)
  -n, --max-files <COUNT>   Maximum number of files to scan (default: 500000)
  -t, --timeout <SECONDS>   Search timeout in seconds (default: 600)
  -h, --help         Print help
  -V, --version      Print version
```

## Performance

Seek is designed to be fast and efficient:

- Uses Rayon for parallel file traversal
- Shows live progress with animated spinner during search
- Displays current directory being searched in real-time
- Prints results immediately as they're found (no waiting until search completes)
- Avoids unnecessary string allocations
- Efficiently filters results during traversal
- Safe handling of large directories with limits, timeouts, and automatic permission error skipping

## Permission Issues

When searching system directories or protected files, you may encounter permission errors:

```
Search complete! Found 24 matches in /etc (2.3s, 1542 files, 8 permission errors)
```

### Solutions for Permission Issues:

#### All Platforms
- Show permission errors with the `-e` flag to see what's being skipped:
  ```bash
  seek /Library "*.plist" -e
  ```

#### macOS
- **Use sudo** for system directories:
  ```bash
  sudo seek /System "*.plist"
  ```
- **System Integrity Protection (SIP)** may prevent access to some directories even with sudo
- For searching user data directories (Mail, Messages, etc.), grant Terminal "Full Disk Access" in:
  - System Preferences → Privacy & Security → Full Disk Access
- Time Machine backups might have special permissions that require sudo

#### Linux
- **Use sudo** for system directories:
  ```bash
  sudo seek /var/log "*.log"
  ```
- Some mounted filesystems may have special permissions or ACLs
- Consider using root permissions cautiously and only when necessary

#### Windows
- **Run as Administrator** for system directories:
  - Right-click on Command Prompt or PowerShell and select "Run as Administrator"
  - Then run your seek command
- Use proper escaping for Windows paths with spaces:
  ```powershell
  seek "C:\Program Files" "*.dll"
  ```
- Some directories may be protected by Windows security features and remain inaccessible

## Troubleshooting

### Search is Too Slow
- Limit the search depth: `seek ~/Documents "*.pdf" -D 3`
- Add a shorter timeout: `seek /usr "*.conf" -t 30`
- Limit the file count: `seek / "important.txt" -n 100000`

### Too Many Permission Errors
- Skip permission error reporting: remove the `-e` flag
- Run with elevated privileges (see platform-specific instructions above)
- Narrow your search to directories you have access to

### No Results Found
- Check your pattern syntax (glob vs regex mode)
- Verify path exists and is accessible
- Try searching with broader patterns first, then narrow down
- Use `-p` flag to search in full paths, not just filenames

## Development

### GitHub Actions Workflows

This project uses GitHub Actions for automation:

1. **CI Workflow**: Runs on every push and pull request to main
   - Tests the code on Ubuntu, macOS, and Windows
   - Runs clippy and formatting checks
   - Ensures the project builds correctly

2. **Release Workflow**: Triggered when a new tag is pushed
   - Builds binaries for:
     - Linux (x86_64 and ARM64)
     - macOS (Intel, Apple Silicon, and Universal)
     - Windows (x86_64)
   - Creates a GitHub release with all the binaries
   - Generates SHA256 checksums for verification

3. **Homebrew Update Workflow**: Runs when a new release is published
   - Automatically updates the Homebrew formula in CoreZen/homebrew-tap
   - Updates the version, URL, and SHA256 checksum

### Making a New Release

To create a new release:

1. Update the version in `Cargo.toml`
2. Commit your changes: `git commit -am "Bump version to x.y.z"`
3. Tag the commit: `git tag -a vx.y.z -m "Version x.y.z"`
4. Push the changes and tags: `git push && git push --tags`
5. The GitHub Actions workflows will handle the rest!

## License

MIT
