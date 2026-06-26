# xcat

A hybrid CLI that combines `tree` + `cat` + clipboard in one command. Point it at a directory and it prints a visual tree followed by every file's contents. Point it at a file and it cats it. Either way, the output lands on your clipboard ready to paste.

## Why xcat

The main use case is feeding code into an LLM. Instead of manually catting files or cobbling together tree output with file dumps, one command gives you the full picture. It's also handy for sharing project context with coworkers, auditing a directory at a glance, or capturing a snapshot of your workspace.

The output is plain text (no ANSI escape codes in the clipboard), so it pastes cleanly into chat, docs, or prompts.

## Installation

### Prebuilt binaries

Download from [GitHub Releases](https://github.com/abbazs/xcat/releases):

```bash
# Linux x86_64 (glibc)
wget https://github.com/abbazs/xcat/releases/latest/download/xcat-linux-x86_64
chmod +x xcat-linux-x86_64
sudo mv xcat-linux-x86_64 /usr/local/bin/xcat
```

Binaries are available for Linux (x86_64 glibc/musl, aarch64, armv7), macOS (Apple Silicon, Intel), and Windows (x86_64, i686).

### Cargo install

```bash
cargo install --git https://github.com/abbazs/xcat.git
```

### Build from source

```bash
git clone https://github.com/abbazs/xcat.git
cd xcat
cargo build --release
# binary at ./target/release/xcat
```

## Quick Start

```bash
# Dump the current directory: tree + all file contents, copied to clipboard
xcat

# Cat a single file
xcat src/main.rs

# Pipe content directly
echo "fix this bug" | xcat

# Dump a specific directory with depth limit
xcat src/ --max-depth 2
```

## Usage

```
Usage: xcat [OPTIONS] [PATH]...

Arguments:
  [PATH]...  One or more file or directory paths [default: .]

Options:
      --dirs-only           Show only directories in tree
      --max-depth <N>       Limit recursion depth
      --output <TYPE>       Output JSON instead of tree view (pass "json")
      --no-copy             Disable clipboard copy
      --include-locks       Include lock files (default: ignored)
      --include-files <GLOB>  Filter to files matching a glob pattern (e.g., "*.rs")
  -h, --help                Print help
  -V, --version             Print version
```

## Examples

### Viewing files

Cat a single file. The relative path is printed as a header, then the contents follow.

```bash
xcat README.md
```

```
./README.md
# xcat
A hybrid CLI that combines tree + cat + clipboard...
```

Cat multiple files at once:

```bash
xcat Cargo.toml src/main.rs
```

```
./Cargo.toml
[package]
name = "xcat"
version = "0.1.0"

################################################################################
./src/main.rs
fn main() {
    println!("Hello");
}
```

### Directory trees

Running `xcat` with no arguments defaults to `.` and shows the tree for your current directory.

```bash
xcat
```

```
# Tree structure for `my_project`
📁 ./my_project
├── 📁 src
│   ├── 📄 main.rs
│   └── 📄 utils.rs
├── 📁 tests
│   └── 📄 integration_test.rs
├── 📄 Cargo.toml
└── 📄 README.md

# File Contents

# ./src/main.rs
fn main() {
    println!("Hello");
}

# ./src/utils.rs
pub fn helper() -> bool {
    true
}

# ./tests/integration_test.rs
#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

# ./Cargo.toml
[package]
name = "my_project"
version = "0.1.0"

# ./README.md
# my_project
A sample project.
```

Show only directories, no files:

```bash
xcat --dirs-only
```

```
# Tree structure for `my_project`
📁 ./my_project
├── 📁 src
└── 📁 tests
```

Limit recursion depth:

```bash
xcat --max-depth 1
```

```
# Tree structure for `my_project`
📁 ./my_project
├── 📁 src
├── 📁 tests
├── 📄 Cargo.toml
└── 📄 README.md
```

### Tree + file contents

The default behavior. Give it a directory and you get the tree visualization followed by a "File Contents" section with every file's path and content.

```bash
xcat src/
```

```
# Tree structure for `src`
📁 ./src
├── 📄 main.rs
└── 📄 utils.rs

# File Contents

# ./src/main.rs
fn main() {
    println!("Hello");
}

# ./src/utils.rs
pub fn helper() -> bool {
    true
}
```

### Filtering files

Include only Rust files. The `.rs` pattern is auto-expanded to `*.rs`:

```bash
xcat --include-files .rs
```

```
# Tree structure for `my_project`
📁 ./my_project
├── 📁 src
│   ├── 📄 main.rs
│   └── 📄 utils.rs
└── 📁 tests
    └── 📄 integration_test.rs

# File Contents

# ./src/main.rs
fn main() {
    println!("Hello");
}
...
```

Use a full glob pattern:

```bash
xcat --include-files "*.toml"
```

Directories are shown only if they recursively contain a matching file. If no files match inside a directory, that directory is excluded from the tree entirely.

### Lock files

Lock files are ignored by default. These are skipped unless you opt in:

- `Cargo.lock`
- `package-lock.json`
- `yarn.lock`
- `pnpm-lock.yaml`
- `uv.lock`
- Any file ending in `.lock`

Include them with `--include-locks`:

```bash
xcat --include-locks
```

### JSON output

Export the directory structure as structured JSON:

```bash
xcat --output json
```

```json
{
  "name": "my_project",
  "path": "./my_project",
  "is_dir": true,
  "is_empty": false,
  "children": [
    {
      "name": "src",
      "path": "./src",
      "is_dir": true,
      "is_empty": false,
      "children": [
        {
          "name": "main.rs",
          "path": "./src/main.rs",
          "is_dir": false,
          "is_empty": false,
          "children": null
        }
      ]
    }
  ]
}
```

### Piped stdin

Pipe content directly into xcat. The piped text is printed and copied to clipboard.

```bash
echo "hello world" | xcat
```

```
stdin
hello world
```

Use `-` as an explicit stdin placeholder, useful when combining with file paths:

```bash
echo "preamble text" | xcat - README.md
```

### Multiple paths

Combine files and directories in a single invocation. Each input is separated by a divider:

```bash
xcat src/ README.md tests/
```

```
# Tree structure for `src`
📁 ./src
├── 📄 main.rs
└── 📄 utils.rs

# File Contents

# ./src/main.rs
fn main() {
    println!("Hello");
}

# ./src/utils.rs
pub fn helper() -> bool {
    true
}
################################################################################
./README.md
# xcat
A hybrid CLI...
################################################################################
# Tree structure for `tests`
📁 ./tests
└── 📄 integration_test.rs

# File Contents

# ./tests/integration_test.rs
#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}
```

### Clipboard

Output is automatically copied to your clipboard. The clipboard gets the plain-text version (no terminal colors), so it pastes cleanly anywhere.

Disable clipboard copy when you just want terminal output:

```bash
xcat --no-copy src/
```

### AI context

The primary use case. Dump an entire project directory and paste it straight into an LLM prompt:

```bash
xcat src/ --max-depth 3 --no-copy > project_context.txt
```

Or just run it and paste from clipboard:

```bash
xcat src/
# output is on clipboard, paste into your LLM of choice
```

Combine with file filtering to narrow the context:

```bash
xcat src/ --include-files "*.rs"
```

## Platform Support

Prebuilt binaries are available for:

- **Linux**: x86_64 (glibc, musl), aarch64, armv7
- **macOS**: Apple Silicon (aarch64), Intel (x86_64)
- **Windows**: x86_64, i686

## Clipboard Dependencies

On Linux, xcat talks to the clipboard through `arboard`, which auto-detects the session:

- **Wayland** (e.g. Ubuntu 22.04+/26.04, Fedora): uses `wl-copy` (from `wl-clipboard`) detached into its own session so the clipboard persists after xcat exits. Requires the `wl-clipboard` package:
  ```bash
  # Debian / Ubuntu
  sudo apt-get install wl-clipboard
  # Fedora
  sudo dnf install wl-clipboard
  ```
  Falls back to `arboard`'s native backend if `wl-copy` is absent.
- **X11**: uses `x11rb` via `arboard`. Install the X11 libraries if needed:
  ```bash
  # Debian / Ubuntu
  sudo apt-get install xorg-dev libxcb-shape0-dev libxcb-xfixes0-dev
  # Fedora
  sudo dnf install libxcb-devel
  ```

On macOS and Windows, no additional dependencies are needed.

## License

MIT
