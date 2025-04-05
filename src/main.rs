use clap::Parser;
use colored::*;
use ignore::WalkBuilder;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// CLI Tree visualizer like the Unix `tree` command
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Root directory
    #[arg(default_value = ".")]
    directory: String,

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
    let root_path = Path::new(&args.directory);

    if !root_path.exists() || !root_path.is_dir() {
        eprintln!("Error: '{}' is not a valid directory.", root_path.display());
        std::process::exit(1);
    }

    let mut output_buffer = String::new();

    if args.output.as_deref() == Some("json") {
        let json_tree = build_json_tree(root_path, 0, args.max_depth, args.dirs_only);
        let json = serde_json::to_string_pretty(&json_tree).unwrap();
        println!("{}", json);
        output_buffer = json;
    } else {
        let heading = format!("# tree structure of directory {}", root_path.display());
        println!("{}", heading);
        println!("{}", root_path.display());

        output_buffer.push_str(&format!("{}\n{}\n", heading, root_path.display()));
        collect_tree_output(
            root_path,
            "".to_string(),
            true,
            0,
            args.max_depth,
            args.dirs_only,
            &mut output_buffer,
        );
        print!("{}", output_buffer);
    }

    if !args.no_copy {
        copy_to_clipboard(&output_buffer);
    }
}

fn collect_tree_output(
    path: &Path,
    prefix: String,
    is_last: bool,
    depth: usize,
    max_depth: Option<usize>,
    dirs_only: bool,
    output: &mut String,
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
        .filter(|entry| {
            let p = entry.path();
            if p == path {
                return false;
            }
            if dirs_only && !p.is_dir() {
                return false;
            }
            true
        })
        .collect();

    entries.sort_by_key(|e| e.path().to_path_buf());

    let count = entries.len();
    for (i, entry) in entries.into_iter().enumerate() {
        let path = entry.path();
        let is_last_entry = i == count - 1;

        let connector = if is_last_entry { "└──" } else { "├──" }.bright_black();
        let name = path.file_name().unwrap().to_string_lossy();

        let display_name = if path.is_dir() {
            format!("📁 {}", name).blue().bold()
        } else {
            format!("📄 {}", name).green()
        };

        let line = format!("{}{} {}\n", prefix, connector, display_name);
        output.push_str(&line);

        if path.is_dir() {
            let new_prefix = if is_last_entry {
                format!("{}    ", prefix)
            } else {
                format!("{}{}", prefix, "│   ".bright_black())
            };
            collect_tree_output(
                &path,
                new_prefix,
                is_last_entry,
                depth + 1,
                max_depth,
                dirs_only,
                output,
            );
        }
    }
}

fn build_json_tree(
    path: &Path,
    depth: usize,
    max_depth: Option<usize>,
    dirs_only: bool,
) -> TreeNode {
    let name = path
        .file_name()
        .unwrap_or_else(|| path.as_os_str())
        .to_string_lossy()
        .into_owned();

    let is_dir = path.is_dir();

    let children = if is_dir {
        if max_depth.map_or(true, |max| depth < max) {
            let walker = WalkBuilder::new(path)
                .max_depth(Some(1))
                .hidden(false)
                .standard_filters(true)
                .build();

            let entries = walker
                .filter_map(Result::ok)
                .filter(|e| {
                    let p = e.path();
                    if p == path {
                        return false;
                    }
                    if dirs_only && !p.is_dir() {
                        return false;
                    }
                    true
                })
                .collect::<Vec<_>>();

            Some(
                entries
                    .into_iter()
                    .map(|e| build_json_tree(e.path(), depth + 1, max_depth, dirs_only))
                    .collect(),
            )
        } else {
            Some(vec![])
        }
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

fn copy_to_clipboard(text: &str) {
    use arboard::Clipboard;
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text(text.to_owned());
    }
}
