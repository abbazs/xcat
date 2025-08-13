/*
 * xcat - Extended cat with tree visualization and clipboard integration
 *
 * A hybrid CLI tool that intelligently handles both files and directories,
 * combining the functionality of cat, tree, and xclip into a single utility.
 *
 * This version supports providing multiple file and directory paths as arguments.
 */

use clap::Parser;
use colored::*;
use globset::{Glob, GlobMatcher};
use ignore::{DirEntry, WalkBuilder};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// A hybrid cat/tree/xclip CLI tool that handles multiple files and directories.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// One or more file or directory paths to process.
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// Show only directories
    #[arg(long)]
    dirs_only: bool,

    /// Limit recursion depth for directory trees
    #[arg(long)]
    max_depth: Option<usize>,

    /// Output JSON instead of a visual tree
    #[arg(long)]
    output: Option<String>,

    /// Disable automatically copying the output to the clipboard
    #[arg(long = "no-copy", default_value_t = false)]
    no_copy: bool,

    /// Include lock files (e.g., Cargo.lock), which are ignored by default
    #[arg(long, default_value_t = false)]
    include_locks: bool,

    /// Filter the tree to only include files matching a glob pattern (e.g., "*.rs")
    #[arg(long)]
    include_files: Option<String>,
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
    let include_matcher = args.include_files.as_ref().map(|pattern| {
        let pattern = if !pattern.starts_with('*') {
            format!("*{}", pattern)
        } else {
            pattern.clone()
        };
        Glob::new(&pattern)
            .expect("Failed to compile glob pattern")
            .compile_matcher()
    });

    let mut output_buffer = String::new();
    let multiple_inputs = args.paths.len() > 1;

    for (index, path_str) in args.paths.iter().enumerate() {
        if multiple_inputs && index > 0 {
            let separator = format!("\n{}\n", "#".repeat(80));
            println!("{}", separator.bright_black());
            output_buffer.push_str(&separator);
        }

        let input_path = Path::new(path_str);
        if !input_path.exists() {
            eprintln!("Error: '{}' does not exist.", input_path.display());
            continue; // Skip to the next path
        }

        if input_path.is_file() {
            process_file(input_path, &mut output_buffer);
        } else if input_path.is_dir() {
            process_directory(input_path, &args, &mut output_buffer, &include_matcher);
        } else {
            eprintln!(
                "Error: '{}' is not a valid file or directory.",
                input_path.display()
            );
        }
    }

    if !args.no_copy {
        copy_to_clipboard(&output_buffer);
    }
}

fn process_file(file_path: &Path, output_buffer: &mut String) {
    let cwd = std::env::current_dir().unwrap();
    let rel_path = file_path.strip_prefix(&cwd).unwrap_or(file_path);
    let relative_path_str = format!("./{}", rel_path.display());

    match fs::read_to_string(file_path) {
        Ok(content) => {
            let file_header = relative_path_str.green();
            println!("{}", file_header);
            println!("{}", content);

            output_buffer.push_str(&relative_path_str);
            output_buffer.push('\n');
            output_buffer.push_str(&content);

            if !content.ends_with('\n') {
                output_buffer.push('\n');
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path.display(), e);
        }
    }
}

fn process_directory(
    root_path: &Path,
    args: &Args,
    output_buffer: &mut String,
    include_matcher: &Option<GlobMatcher>,
) {
    let comment_dir_name = get_dir_name(root_path);
    let mut file_contents = Vec::new();

    if args.output.as_deref() == Some("json") {
        let json_tree = build_json_tree(
            root_path,
            0,
            args.max_depth,
            args.dirs_only,
            args.include_locks,
            include_matcher,
        );
        let json = serde_json::to_string_pretty(&json_tree).unwrap();
        println!("{}", json);
        output_buffer.push_str(&json);
    } else {
        let heading = format!("# Tree structure for `{}`", comment_dir_name);
        let root_line = format!("📁 {}", root_path.display());

        println!("{}", heading.bold());
        println!("{}", root_line.blue().bold());

        output_buffer.push_str(&heading);
        output_buffer.push_str("\n");
        output_buffer.push_str(&root_line);
        output_buffer.push_str("\n");

        collect_tree_output(
            root_path,
            "".to_string(),
            0,
            args,
            output_buffer,
            &mut file_contents,
            include_matcher,
        );

        if !file_contents.is_empty() {
            let file_content_heading = "\n# File Contents\n";
            println!("{}", file_content_heading.bold());
            output_buffer.push_str(file_content_heading);
            for (path, content) in file_contents {
                let path_header = format!("# {}\n", path.display());
                println!("{}", path_header.green());
                println!("{}", content);

                output_buffer.push_str(&path_header);
                output_buffer.push_str(&content);
                if !content.ends_with('\n') {
                    output_buffer.push('\n');
                }
            }
        }
    }
}

fn is_directory_empty(path: &Path, args: &Args, include_matcher: &Option<GlobMatcher>) -> bool {
    let walker = WalkBuilder::new(path)
        .max_depth(Some(1))
        .hidden(false)
        .standard_filters(true)
        .build();

    !walker.filter_map(Result::ok).any(|entry| {
        filter_entry(
            &entry,
            path,
            args.dirs_only,
            args.include_locks,
            include_matcher,
        )
    })
}

fn collect_tree_output(
    path: &Path,
    prefix: String,
    depth: usize,
    args: &Args,
    output: &mut String,
    file_contents: &mut Vec<(PathBuf, String)>,
    include_matcher: &Option<GlobMatcher>,
) {
    if let Some(max) = args.max_depth {
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
        .filter(|entry| {
            filter_entry(
                entry,
                path,
                args.dirs_only,
                args.include_locks,
                include_matcher,
            )
        })
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

        let is_empty_dir =
            entry_path.is_dir() && is_directory_empty(entry_path, args, include_matcher);

        // When filtering, don't show empty directories that don't contain matches.
        if entry_path.is_dir() && is_empty_dir && include_matcher.is_some() {
            continue;
        }

        let plain_line = if is_empty_dir {
            format!("{}{} {} {} (empty)\n", prefix, connector, icon, name)
        } else {
            format!("{}{} {} {}\n", prefix, connector, icon, name)
        };
        output.push_str(&plain_line);

        let colored_name = if entry_path.is_dir() {
            if is_empty_dir {
                format!("{} {} (empty)", icon, name).bright_black()
            } else {
                format!("{} {}", icon, name).blue().bold()
            }
        } else {
            format!("{} {}", icon, name).green()
        };
        println!(
            "{}{}{}",
            prefix.bright_black(),
            connector.bright_black(),
            colored_name
        );

        if entry_path.is_file() {
            if let Ok(content) = fs::read_to_string(entry_path) {
                file_contents.push((entry_path.to_path_buf(), content));
            }
        }

        if entry_path.is_dir() {
            let new_prefix = if is_last_entry {
                format!("{}   ", prefix)
            } else {
                format!("{}│  ", prefix)
            };
            collect_tree_output(
                entry_path,
                new_prefix,
                depth + 1,
                args,
                output,
                file_contents,
                include_matcher,
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
    include_matcher: &Option<GlobMatcher>,
) -> TreeNode {
    let name = path
        .file_name()
        .unwrap_or_else(|| path.as_os_str())
        .to_string_lossy()
        .into_owned();
    let is_dir = path.is_dir();
    // This is simplified, a more complex check might be needed depending on desired JSON output for empty/filtered dirs
    let is_empty = is_dir && WalkBuilder::new(path).max_depth(Some(1)).build().count() <= 1;

    let children = if is_dir && max_depth.map_or(true, |max| depth < max) {
        let walker = WalkBuilder::new(path)
            .max_depth(Some(1))
            .hidden(false)
            .standard_filters(true)
            .build();
        let children_nodes: Vec<_> = walker
            .filter_map(Result::ok)
            .filter(|e| filter_entry(e, path, dirs_only, include_locks, include_matcher))
            .map(|e| {
                build_json_tree(
                    e.path(),
                    depth + 1,
                    max_depth,
                    dirs_only,
                    include_locks,
                    include_matcher,
                )
            })
            .collect();
        Some(children_nodes)
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

fn filter_entry(
    entry: &DirEntry,
    parent: &Path,
    dirs_only: bool,
    include_locks: bool,
    include_matcher: &Option<GlobMatcher>,
) -> bool {
    let path = entry.path();
    if path == parent {
        return false;
    }

    if let Some(matcher) = include_matcher {
        if path.is_dir() {
            // A directory is included if it recursively contains any file that matches the pattern.
            let walker = WalkBuilder::new(path)
                .hidden(false)
                .standard_filters(true)
                .build();
            return walker
                .filter_map(Result::ok)
                .any(|e| e.path().is_file() && matcher.is_match(e.path()));
        } else if !matcher.is_match(path) {
            // It's a file, but it doesn't match.
            return false;
        }
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
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Err(e) = clipboard.set_text(text.to_owned()) {
            eprintln!("Error copying to clipboard: {}", e);
        }
    } else {
        eprintln!("Error: Could not initialize clipboard.");
    }
}
