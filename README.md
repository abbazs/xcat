# xcat

**Extended cat with tree visualization and clipboard integration**

A hybrid CLI tool that intelligently handles both files and directories, combining the functionality of `cat`, `tree`, and `xclip` into a single utility.

## 🚀 Features

- **Smart path detection** - Auto-detects files vs directories
- **Colorized terminal output** with emoji icons 🎨
- **Automatic clipboard integration** - No more manual piping
- **Multiple output formats** - Visual tree or JSON export
- **Configurable filtering** - Control depth, file types, and exclusions
- **File content embedding** - Include file contents in directory trees
- **Gitignore respect** - Uses standard filters via ignore crate

## 📋 Modes of Operation

### 1. File Mode (cat-like)
- Reads and displays file contents with relative path formatting
- Output format: `./<filename>` followed by file content
- Automatically copies content to clipboard for easy sharing

### 2. Directory Mode (tree-like)
- Creates hierarchical visualizations of directory structures
- Enhanced version of Unix `tree` with modern terminal styling
- Supports both visual tree output and JSON representation
- Can include file contents alongside the directory structure

### 3. Clipboard Integration (xclip-like)
- Automatically copies all output to system clipboard
- Eliminates need for manual piping to clipboard utilities
- Can be disabled with `--no-copy` flag when not needed

## 📖 Usage Examples

```bash
# Display current directory tree
xcat

# Display and copy file content
xcat src/main.rs

# Show only directories
xcat --dirs-only

# Export as JSON
xcat --output json

# Limit traversal depth
xcat --max-depth 2

# Include lock files (normally excluded)
xcat --include-locks

# Disable clipboard copying
xcat --no-copy
```

## 🛠️ Installation

### From Releases (Recommended)

Download the appropriate binary for your system from the [releases page](https://github.com/abbazs/xcat/releases):

#### Linux
```bash
# x86_64 (glibc) - Most common
wget https://github.com/abbazs/xcat/releases/latest/download/xcat-linux-x86_64.tar.gz
tar -xzf xcat-linux-x86_64.tar.gz
sudo mv xcat /usr/local/bin/

# x86_64 (musl) - Static binary
wget https://github.com/abbazs/xcat/releases/latest/download/xcat-linux-x86_64-musl.tar.gz
tar -xzf xcat-linux-x86_64-musl.tar.gz
sudo mv xcat /usr/local/bin/

# ARM64
wget https://github.com/abbazs/xcat/releases/latest/download/xcat-linux-aarch64.tar.gz
tar -xzf xcat-linux-aarch64.tar.gz
sudo mv xcat /usr/local/bin/
```

#### macOS
```bash
# Intel Macs
wget https://github.com/abbazs/xcat/releases/latest/download/xcat-macos-x86_64.tar.gz
tar -xzf xcat-macos-x86_64.tar.gz
sudo mv xcat /usr/local/bin/

# Apple Silicon Macs
wget https://github.com/abbazs/xcat/releases/latest/download/xcat-macos-aarch64.tar.gz
tar -xzf xcat-macos-aarch64.tar.gz
sudo mv xcat /usr/local/bin/
```

#### Windows
1. Download `xcat-windows-x86_64.exe.zip` from releases
2. Extract the zip file
3. Move `xcat.exe` to a directory in your PATH

### From Source

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/abbazs/xcat.git
cd xcat
cargo build --release

# Install
sudo cp target/release/xcat /usr/local/bin/
```

## 🔧 Command Line Options

```
Usage: xcat [OPTIONS] [PATH]

Arguments:
  [PATH]  Root directory or file path [default: .]

Options:
      --dirs-only        Show only directories
      --max-depth <MAX_DEPTH>  Limit recursion depth
      --output <OUTPUT>  Output JSON instead of tree view
      --no-copy          Disable clipboard copy
      --include-locks    Include lock files (default: ignored)
  -h, --help             Print help
  -V, --version          Print version
```

## 📝 Output Examples

### File Mode
```bash
$ xcat README.md
./README.md
# xcat
Extended cat with tree visualization...
```

### Directory Mode
```bash
$ xcat --max-depth 2
# tree structure of directory `my-project`
📁 .
├── 📁 src
│   ├── 📄 main.rs
│   └── 📄 lib.rs
├── 📄 Cargo.toml
├── 📄 Cargo.lock
└── 📄 README.md
```

### JSON Output
```bash
$ xcat --output json
{
  "name": "my-project",
  "path": ".",
  "is_dir": true,
  "children": [
    {
      "name": "src",
      "path": "./src",
      "is_dir": true,
      "children": [...]
    }
  ]
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) 🦀
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) for gitignore support
- Uses [colored](https://github.com/mackwic/colored) for terminal colors
- Uses [arboard](https://github.com/1Password/arboard) for clipboard integration