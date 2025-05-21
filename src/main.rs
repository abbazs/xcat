/*
 * sdir - A CLI tool for visualizing file/directory structures
 * 
 * This tool provides two main functionalities:
 * 
 * 1. File Mode:
 *    - When provided with a file path, it reads the file's content
 *    - Outputs the file path as ./<filename>
 *    - Displays and copies the file content to the clipboard
 * 
 * 2. Directory Mode:
 *    - When provided with a directory path, it creates a tree visualization
 *    - Similar to the Unix 'tree' command but with additional features
 *    - Supports colored output in the terminal
 *    - Can generate JSON representation of the directory structure
 *    - Copies the tree structure to clipboard automatically
 *    - Includes file contents in the output if desired
 * 
 * Features:
 *    - Directory traversal with customizable depth
 *    - Directory-only filtering option
 *    - Lock file filtering (can be disabled)
 *    - JSON output option
 *    - Automatic clipboard copying
 *    - Colorized terminal output
 */

use clap::Parser;
use colored::*;
use ignore::{DirEntry, WalkBuilder};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// CLI Tree visualizer like the Unix `tree` command
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Root directory or file path
    #[arg(default_value = ".")]
    path: String,

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
}

#[derive(Debug, Serialize)]
struct TreeNode {
    name: String,
    path: String,
    is_dir: bool,
    children: Option<Vec<TreeNode>>,
}

fn main() {
    let args = Args::parse();
    let input_path = Path::new(&args.path);

    if !input_path.exists() {
        eprintln!("Error: '{}' does not exist.", input_path.display());
        std::process::exit(1);
    }

    let mut output_buffer = String::new();

    if input_path.is_file() {
        // Handle file input
        process_file(input_path, &mut output_buffer);
    } else if input_path.is_dir() {
        // Handle directory input (original functionality)
        process_directory(input_path, &args, &mut output_buffer);
    } else {
        eprintln!("Error: '{}' is neither a valid file nor directory.", input_path.display());
        std::process::exit(1);
    }

    if !args.no_copy {
        copy_to_clipboard(&output_buffer);
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
    let comment_dir_name = get_dir_name(&args.path);
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

        // Plain text for clipboard
        let plain_line = format!("{}{} {} {}", prefix, connector, icon, name);
        output.push_str(&plain_line);
        output.push('\n');

        // Colored output for terminal
        let colored_connector = connector.bright_black();
        let colored_name = if entry_path.is_dir() {
            format!("{} {}", icon, name).blue().bold()
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

fn get_dir_name(directory: &str) -> String {
    if directory == "." {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| ".".to_string())
    } else {
        PathBuf::from(directory)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| directory.to_string())
    }
}

fn copy_to_clipboard(text: &str) {
    use arboard::Clipboard;
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(text.to_owned());
    }
}