# Prompt: Create a Rust CLI Tool for Directory and File Visualization

I need a command-line tool in Rust called "sdir" that can visualize directory structures and display file contents. The tool should have the following features:

## Core Functionality

1. Directory mode: Visualize directory structures like the Unix 'tree' command
2. File mode: When a file is provided, display the file content and copy it to clipboard in format `./<filename>\n<file content>`
3. Automatic clipboard integration for easy sharing

## Basic Requirements

- Use the Clap crate for command-line argument parsing
- Support colorized output in the terminal using the 'colored' crate
- Include appropriate error handling for invalid paths or unreadable files

## Directory Mode Features

- Tree-like visualization with branch connectors (├── and └──)
- Icons for directories (📁) and files (📄)
- Ability to filter to show only directories
- Option to limit recursion depth
- Option to include/exclude lock files (default: exclude Cargo.lock files)
- Option to output as JSON instead of tree view
- Include file contents in the output if desired
- Copy the tree structure to clipboard automatically (unless disabled)

## File Mode Features

- Read and display file content
- Format output as `./<filename>` followed by the file content
- Copy the output to clipboard

## Command-Line Arguments

- `[PATH]`: Root directory or file path (default: ".")
- `--dirs-only`: Show only directories
- `--max-depth`: Limit recursion depth
- `--output`: Output format (e.g., "json")
- `--no-copy`: Disable clipboard copy
- `--include-locks`: Include lock files

## Dependencies

- clap (for command-line argument parsing)
- colored (for terminal coloring)
- ignore (for directory walking)
- serde and serde_json (for JSON serialization)
- arboard (for clipboard operations)

## Implementation Notes

- Use a clear separation of concerns between file and directory handling
- Provide good error messages for invalid paths or permissions issues
- Ensure code is well-structured and commented
- Add a comprehensive README.md file explaining installation and usage
