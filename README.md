# sdir

A CLI utility that combines file viewing, directory traversal, and clipboard operations. sdir functions as a multi-purpose command integrating the capabilities of tree, cat, and clipboard management tools.

## Features

- **File Mode**: View and copy file contents with a single command
- **Directory Mode**: Generate tree-like visualizations of directory structures
- **Colorized Output**: Better terminal visualization with colorized output
- **Flexible Options**: Control depth, filtering, and output format
- **Clipboard Integration**: Automatically copies output to your clipboard
- **JSON Export**: Option to export the structure as JSON

## Installation

### Prerequisites

You need to have Rust and Cargo installed on your system. If you don't have them installed, you can get them from [rustup.rs](https://rustup.rs/).

```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building from Source

1. Clone this repository:

    ```bash
    git clone https://github.com/abbazs/sdir.git
    cd sdir
    ```

2. Build the project:

    ```bash
    cargo build --release
    ```

3. The compiled binary will be available at `./target/release/sdir`

4. (Optional) Install the binary to your system:

    ```bash
    cargo install --path .
    ```

## Usage

### Basic Usage

```bash
# View directory structure (current directory by default)
sdir

# View specific directory structure
sdir /path/to/directory

# View file content
sdir /path/to/file.txt
```

### Command Line Options

```text
Usage: sdir [OPTIONS] [PATH]

Arguments:
  [PATH]  Root directory or file path [default: .]

Options:
      --dirs-only       Show only directories
      --max-depth <N>   Limit recursion depth
      --output <TYPE>   Output JSON instead of tree view
      --no-copy         Disable clipboard copy
      --include-locks   Include lock files (default: ignored)
  -h, --help            Print help
  -V, --version         Print version
```

### Examples

```bash
# Show only directories
sdir --dirs-only

# Limit depth to 2 levels
sdir --max-depth 2

# Generate JSON output
sdir --output json > structure.json

# View file content
sdir config.json

# Include lock files
sdir --include-locks
```

## Output Examples

### Directory Mode

```text
# tree structure of directory `my_project`
📁 ./my_project
├── 📁 src
│   ├── 📄 main.rs
│   └── 📄 utils.rs
├── 📄 Cargo.toml
└── 📄 README.md
```

### File Mode

```json
./config.json
{
  "version": "1.0",
  "settings": {
    "debug": false,
    "theme": "dark"
  }
}
```

## Clipboard Support

The tool automatically copies the output to your clipboard. If you want to disable this functionality, use the `--no-copy` flag.

## Building for Different Platforms

### Windows

```bash
cargo build --release --target x86_64-pc-windows-msvc
```

### macOS

```bash
cargo build --release --target x86_64-apple-darwin
```

### Linux

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

## Troubleshooting

### Missing Clipboard Dependencies

On Linux, you might need additional dependencies for clipboard functionality:

```bash
# For Debian/Ubuntu
sudo apt-get install xorg-dev libxcb-shape0-dev libxcb-xfixes0-dev

# For Fedora
sudo dnf install libxcb-devel
```

### Permission Issues

If you encounter permission issues when running the tool:

```bash
# Make the binary executable
chmod +x ./target/release/sdir
```

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests if you have ideas for improvements or new features.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
