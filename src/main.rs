/*
 * xcat - Extended cat with tree visualization and clipboard integration
 * 
 * A hybrid CLI tool that intelligently handles both files and directories,
 * combining the functionality of cat, tree, and xclip into a single utility.
 * 
 * Modes of Operation:
 * 
 * 1. File Mode (cat-like):
 *    - Reads and displays file contents with relative path formatting
 *    - Output format: ./<filename> followed by file content
 *    - Automatically copies content to clipboard for easy sharing
 * 
 * 2. Directory Mode (tree-like):
 *    - Creates hierarchical visualizations of directory structures
 *    - Enhanced version of Unix 'tree' with modern terminal styling
 *    - Supports both visual tree output and JSON representation
 *    - Can include file contents alongside the directory structure
 *    - Shows empty directories in gray color
 * 
 * 3. Clipboard Integration (xclip-like):
 *    - Automatically copies all output to system clipboard
 *    - Eliminates need for manual piping to clipboard utilities
 *    - Can be disabled with --no-copy flag when not needed
 * 
 * Key Features:
 *    - Smart path detection (auto-detects files vs directories)
 *    - Colorized terminal output with emoji icons
 *    - Empty directories shown in gray
 *    - Configurable traversal depth and filtering options
 *    - Lock file exclusion (Cargo.lock, etc.) with override option
 *    - JSON export capability for programmatic use
 *    - File content embedding in directory trees
 *    - Respects .gitignore and standard filters via ignore crate
 * 
 * Usage Examples:
 *    xcat                        # Current directory tree
 *    xcat src/main.rs            # Display and copy file content
 *    xcat --dirs-only            # Show only directories
 *    xcat --output json          # Export as JSON
 *    xcat --max-depth 2          # Limit depth
 */

use clap::Parser;
use colored::*;
use ignore::{DirEntry, WalkBuilder};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// CLI tool for visualizing file/directory structures
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Root directory or file path(s)
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// Show only directories
    #[arg(long)]
    dirs_only: bool,

    /// Limit recursion depth
    #[arg(long)]
    max_depth: Option<usize>,

    /// Output JSON instead of tree view
    #[arg(long)]
    output: Option<String>,

    /// Disable clipboard copy
    #[arg(long = "no-copy", default_value_t = false)]
    no_copy: bool,

    /// Include lock files (default: ignored)
    #[arg(long, default_value_t = false)]
    include_locks: bool,

    /// Save Output as file
    #[arg(long)]
    save: Option<String>,
}

#[derive(Debug, Serialize)]
struct TreeNode {
    name: String,
    path: String,
    is_dir: bool,
    is_empty: bool,
    children: Option<Vec<TreeNode>>,
}

fn main() {
    let args = Args::parse();
    let input_paths: Vec<_> = if args.paths.is_empty() {
        vec![".".to_string()]
    } else {
        args.paths.clone()
    };

    let mut output_buffer = String::new();
    let multiple_inputs = input_paths.len() > 1;
    
    for (index, input_path_str) in input_paths.iter().enumerate() {
        // Add separator for multiple inputs (except for the first one)
        if multiple_inputs && index > 0 {
            let separator = "#".repeat(80);
            println!("{}", separator);
            output_buffer.push_str(&separator);
            output_buffer.push('\n');
        }
        
        let input_path = Path::new(input_path_str);
        if !input_path.exists() {
            eprintln!("Error: '{}' does not exist.", input_path.display());
            continue;
        }
        if input_path.is_file() {
            process_file(input_path, &mut output_buffer);
        } else if input_path.is_dir() {
            process_directory(input_path, &args, &mut output_buffer);
        } else {
            eprintln!("Error: '{}' is neither a valid file nor directory.", input_path.display());
        }
    }
    if !args.no_copy {
        copy_to_clipboard(&output_buffer);
    }

    if let Some(file_path) = &args.save {
        write_to_file(&output_buffer, file_path);
        println!("Output saved to '{}'", file_path);
    } 
}

fn process_file(file_path: &Path, output_buffer: &mut String) {
    // Format: ./<filename>\n<file content>
    let relative_path = format!("./{}", file_path.file_name().unwrap().to_string_lossy());
    
    match fs::read_to_string(file_path) {
        Ok(content) => {
            println!("{}", relative_path);
            println!("{}", content);
            
            output_buffer.push_str(&relative_path);
            output_buffer.push('\n');
            output_buffer.push_str(&content);
            
            // Ensure content ends with newline
            if !content.ends_with('\n') {
                output_buffer.push('\n');
            }
            
            println!("File content copied to clipboard.");
        },
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path.display(), e);
            std::process::exit(1);
        }
    }
}

fn process_directory(root_path: &Path, args: &Args, output_buffer: &mut String) {
    let comment_dir_name = get_dir_name(root_path);
    let mut file_contents = Vec::new();

    if args.output.as_deref() == Some("json") {
        let json_tree = build_json_tree(
            root_path,
            0,
            args.max_depth,
            args.dirs_only,
            args.include_locks,
        );
        let json = serde_json::to_string_pretty(&json_tree).unwrap();
        println!("{}", json);
        *output_buffer = json;
    } else {
        let heading = format!("# tree structure of directory `{}`", comment_dir_name);
        let root_line = format!("📁 {}", root_path.display());

        println!("{}", heading);
        println!("{}", root_line);

        output_buffer.push_str(&heading);
        output_buffer.push('\n');
        output_buffer.push_str(&root_line);
        output_buffer.push('\n');

        collect_tree_output(
            root_path,
            "".to_string(),
            0,
            args.max_depth,
            args.dirs_only,
            args.include_locks,
            output_buffer,
            &mut file_contents,
        );

        if !file_contents.is_empty() {
            output_buffer.push_str("\n# File Contents\n");
            for (path, content) in file_contents {
                output_buffer.push_str(&format!("\n# {}\n", path.display()));
                output_buffer.push_str(&content);
                if !content.ends_with('\n') {
                    output_buffer.push('\n');
                }
            }
        }
    }
}

fn is_directory_empty(path: &Path, dirs_only: bool, include_locks: bool) -> bool {
    let walker = WalkBuilder::new(path)
        .max_depth(Some(1))
        .hidden(false)
        .standard_filters(true)
        .build();

    let entries: Vec<_> = walker
        .filter_map(Result::ok)
        .filter(|entry| filter_entry(entry, path, dirs_only, include_locks))
        .collect();

    // If no entries at all, it's empty
    if entries.is_empty() {
        return true;
    }

    // If we have entries, check if they are all empty directories
    for entry in entries {
        let entry_path = entry.path();
        if entry_path.is_file() {
            // Has at least one file, so not empty
            return false;
        } else if entry_path.is_dir() {
            // Recursively check if this subdirectory is empty
            if !is_directory_empty(entry_path, dirs_only, include_locks) {
                // Has at least one non-empty subdirectory, so not empty
                return false;
            }
        }
    }

    // All entries are empty directories, so this directory is effectively empty
    true
}

fn collect_tree_output(
    path: &Path,
    prefix: String,
    depth: usize,
    max_depth: Option<usize>,
    dirs_only: bool,
    include_locks: bool,
    output: &mut String,
    file_contents: &mut Vec<(PathBuf, String)>,
) {
    if let Some(max) = max_depth {
        if depth >= max {
            return;
        }
    }

    let walker = WalkBuilder::new(path)
        .max_depth(Some(1))
        .hidden(false)
        .standard_filters(true)
        .build();

    let mut entries: Vec<_> = walker
        .filter_map(Result::ok)
        .filter(|entry| filter_entry(entry, path, dirs_only, include_locks))
        .collect();

    entries.sort_by_key(|e| e.path().to_path_buf());

    let count = entries.len();
    for (i, entry) in entries.into_iter().enumerate() {
        let entry_path = entry.path();
        let is_last_entry = i == count - 1;
        let connector = if is_last_entry {
            "└──"
        } else {
            "├──"
        };
        let icon = if entry_path.is_dir() { "📁" } else { "📄" };
        let name = entry_path.file_name().unwrap().to_string_lossy();

        // Check if directory is empty
        let is_empty_dir = entry_path.is_dir() && is_directory_empty(entry_path, dirs_only, include_locks);

        // Plain text for clipboard (include empty indicator)
        let plain_line = if is_empty_dir {
            format!("{}{} {} {} (empty)", prefix, connector, icon, name)
        } else {
            format!("{}{} {} {}", prefix, connector, icon, name)
        };
        output.push_str(&plain_line);
        output.push('\n');

        // Colored output for terminal
        let colored_connector = connector.bright_black();
        let colored_name = if entry_path.is_dir() {
            if is_empty_dir {
                format!("{} {} (empty)", icon, name).bright_black()
            } else {
                format!("{} {}", icon, name).blue().bold()
            }
        } else {
            format!("{} {}", icon, name).green()
        };
        let colored_prefix = if depth > 0 && !prefix.is_empty() {
            prefix.bright_black()
        } else {
            prefix.normal()
        };
        println!("{}{} {}", colored_prefix, colored_connector, colored_name);

        if entry_path.is_file() {
            if let Ok(content) = fs::read_to_string(entry_path) {
                file_contents.push((entry_path.to_path_buf(), content));
            }
        }

        if entry_path.is_dir() {
            let new_prefix = if is_last_entry {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            collect_tree_output(
                entry_path,
                new_prefix,
                depth + 1,
                max_depth,
                dirs_only,
                include_locks,
                output,
                file_contents,
            );
        }
    }
}

fn build_json_tree(
    path: &Path,
    depth: usize,
    max_depth: Option<usize>,
    dirs_only: bool,
    include_locks: bool,
) -> TreeNode {
    let name = path
        .file_name()
        .unwrap_or_else(|| path.as_os_str())
        .to_string_lossy()
        .into_owned();
    let is_dir = path.is_dir();
    let is_empty = is_dir && is_directory_empty(path, dirs_only, include_locks);

    let children = if is_dir && max_depth.map_or(true, |max| depth < max) {
        let walker = WalkBuilder::new(path)
            .max_depth(Some(1))
            .hidden(false)
            .standard_filters(true)
            .build();

        let entries = walker
            .filter_map(Result::ok)
            .filter(|e| filter_entry(e, path, dirs_only, include_locks))
            .collect::<Vec<_>>();

        Some(
            entries
                .into_iter()
                .map(|e| build_json_tree(e.path(), depth + 1, max_depth, dirs_only, include_locks))
                .collect(),
        )
    } else {
        None
    };

    TreeNode {
        name,
        path: path.to_string_lossy().into_owned(),
        is_dir,
        is_empty,
        children,
    }
}

fn filter_entry(entry: &DirEntry, parent: &Path, dirs_only: bool, include_locks: bool) -> bool {
    let path = entry.path();
    if path == parent {
        return false;
    }
    if dirs_only && !path.is_dir() {
        return false;
    }
    if !include_locks && path.file_name().map_or(false, |n| n == "Cargo.lock") {
        return false;
    }
    true
}

fn get_dir_name(path: &Path) -> String {
    if path == Path::new(".") {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| ".".to_string())
    } else {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string())
    }
}

fn copy_to_clipboard(text: &str) {
    use arboard::Clipboard;
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(text.to_owned());
    }
}

fn write_to_file(text: &str, file_path: &str) {
    use std::fs::File;
    use std::io::Write;
    
    if let Ok(mut file) = File::create(file_path) {
        let _ = file.write_all(text.as_bytes());
    }
}