# AGENTS.md

Guide for AI agents working in the `xcat` codebase.

## What This Is

`xcat` is a single-binary Rust CLI that combines `tree` + `cat` + clipboard. Given a directory it prints a visual tree followed by every file's contents; given a file it cats it; given piped stdin it echoes it. The assembled output is copied to the clipboard. Primary use case: dumping a project into an LLM prompt.

The entire application lives in **one file**: `src/main.rs` (~500 lines). There is no `lib.rs` and no internal module structure.

## Commands

```bash
cargo build --release                              # build (binary at target/release/xcat)
cargo test --workspace                             # run tests
cargo fmt --all -- --check                         # format check (CI fails on diff)
cargo clippy --all-targets --all-features -- -D warnings   # lint (warnings are errors in CI)
```

CI (`.github/workflows/ci.yml`) runs fmt + clippy + test, then cross-compiles for 8 targets. **Clippy warnings are hard errors** (`-D warnings`) — fix all of them before considering work done.

## Toolchain

- **Rust edition 2024** (`Cargo.toml`). This requires rustc 1.85+. The code relies on edition-2024 features that agents may not expect: **let-chains** (`if cond && let Ok(x) = ... { }`) and the `is_none_or`/`is_some_and` Option methods appear throughout. Preserve this style; do not "fix" them into nested `if let`.
- No `rust-toolchain.toml` and no `rust-version` MSRV pin. CI uses `dtolnay/rust-toolchain@stable`.

## Architecture & Control Flow

Everything is in `src/main.rs`. Entry point `main()`:

1. `Args::parse()` (clap derive) → `include_files` glob is compiled once into a `GlobMatcher` (see gotcha below).
2. Detect piped stdin via `atty::is(Stream::Stdin)`. If stdin is not a TTY, it is read once into `stdin_content: Option<String>`.
3. Loop over `args.paths` (default `["."]`). For each path: dispatch to `process_stdin` (`-` placeholder or piped-with-default-path), `process_file`, or `process_directory`. Multiple inputs are separated by an 80-char `#` divider.
4. After the loop, copy the assembled `output_buffer` to the clipboard unless `--no-copy`.

Key functions:

- `build_walker` — configures `ignore::WalkBuilder` (the crate is `ignore`, not `walkdir`). Central walker config; all traversal goes through it.
- `filter_entry` — **the single source of truth for filtering**. Enforces `dirs_only`, `include_locks`, and the `include_files` glob matcher. Used by both the tree path and JSON path — any new filter must be added here to apply everywhere.
- `collect_tree_output` — recursive tree printer (text mode). Also collects file contents into a `Vec` for the trailing "File Contents" section.
- `build_json_tree` — recursive builder for `--output json`, produces `TreeNode`.
- `process_file` / `process_stdin` / `process_directory` — the three input handlers.

## Critical Conventions (Gotchas)

### Dual-write to stdout AND buffer
Every output function writes to **two** places simultaneously: `println!` (colored, via the `colored` crate) and `output_buffer.push_str(...)` (plain text, no ANSI codes). The buffer is what gets copied to the clipboard; stdout is what the user sees. **The clipboard must always get plain text.** When adding any output, mirror both writes or the clipboard will be wrong. Never push colored strings into `output_buffer`.

### `--include-files` auto-prepends `*`
In `main()` (around the `include_matcher` construction), if the user's pattern does not start with `*`, a `*` is prepended. So `--include-files .rs` becomes `*.rs`. This is intentional UX — preserve it when touching that code.

### Lock files are hardcoded
`LOCK_FILE_NAMES` (`Cargo.lock`, `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `uv.lock`) plus **any filename ending in `.lock`** are excluded unless `--include-locks`. To add a new lock file, append to the constant.

### Hidden files are NOT ignored, but .gitignore IS respected
`build_walker` sets `.hidden(false)` — hidden/dotfiles are included. But `.ignore`, `.gitignore`, global gitignore, and `.git/info/exclude` are all honored (`ignore(true)`, `git_ignore(true)`, etc.). This combination is non-obvious and intentional.

### Directory filtering with a glob walks the whole subtree
When an `include_matcher` is set, `filter_entry` checks whether a directory *recursively contains* a matching file by walking the entire subtree (`build_walker(path, None)`). This is O(n) per directory and can be slow on huge trees. Empty directories (no matches) are pruned from the tree output entirely. Directories are never matched against the glob themselves.

### `process_file` panics on missing cwd
It calls `std::env::current_dir().unwrap()`. Don't "fix" this unless asked — it's pre-existing.

### arboard needs the `wayland-data-control` feature (do NOT remove it)
`Cargo.toml` declares `arboard` with `default-features = false, features = ["wayland-data-control"]`. arboard's **default features only compile the X11 backend** on Linux; the `wayland-data-control` feature is what pulls in `wl-clipboard-rs` for native Wayland clipboard support. Without it, clipboard silently fails on Wayland sessions (Ubuntu 22.04+/26.04, Fedora default). The feature is safe on macOS/Windows (arboard gates the wayland code to Linux internally). `default-features = false` drops the unused `image-data` feature — xcat is text-only. Don't "simplify" this line back to `arboard = "3.3"`. Backend selection is automatic: arboard prefers Wayland when `WAYLAND_DISPLAY` is set, else falls back to X11.

### Wayland clipboard persistence: xcat uses `setsid wl-copy` (do NOT revert to arboard-only)
Enabling the `wayland-data-control` feature is necessary but **not sufficient**. The Wayland clipboard only persists while a process actively serves it. arboard (via `wl-clipboard-rs`) forks a daemon via plain `fork()`, but on **systemd-based Wayland sessions** (Ubuntu 22.04+/26.04, Fedora) that daemon is killed when the process tree/session exits — the clipboard ends up empty. xcat works around this in `copy_to_clipboard`: when `WAYLAND_DISPLAY` is set, it spawns `wl-copy` detached into its **own session** via the `setsid()` syscall (`pre_exec`), so the daemon survives the parent's exit. It falls back to arboard on X11/macOS/Windows. Do not "simplify" this back to bare `arboard::set_text()` — it silently fails on Wayland. If `wl-copy` is not installed, it falls back to arboard automatically.

### `is_empty` in JSON is a rough heuristic
`build_json_tree` computes `is_empty` as `build_walker(path, Some(1)).count() <= 1`, which does not account for filtering. Treat it as approximate.

## Testing

- Integration tests live in `tests/filters.rs` using `assert_cmd` (spawns the binary via `cargo_bin("xcat")`), `predicates`, `tempfile`, and `serde_json`.
- **Tests require `git init` in the tempdir.** Because the walker honors `.gitignore`, the gitignore-related tests must initialize a real git repo, otherwise `.gitignore` is not consulted. If you add tests involving ignore behavior, run `git init` in the temp dir first.
- Always pass `--no-copy` in tests to avoid touching the clipboard / requiring a display server.
- Use `--output json` + parse with `serde_json::Value` to assert on structure rather than scraping tree text.
- There is no `#[cfg(test)]` unit-test module in `main.rs`.

## Releases

Two workflows:
- `.github/workflows/ci.yml` — on push to `main`, after lint+test+build, **auto-creates a date-based tag** (`vYYYY.MM.DD.N`, incrementing N for same-day pushes) and publishes a release. Merging to `main` publishes automatically — do not be surprised by auto-generated tags.
- `.github/workflows/release.yml` — manually triggered (`workflow_dispatch`) or tag-pushed (`v*.*.*`).

Binaries are cross-compiled for: linux x86_64 (gnu/musl), aarch64, armv7; macOS aarch64 + x86_64; windows x86_64 + i686.

## Project Files

- `PROMPT.md` — the original spec prompt used to generate this tool (historical; the tool was originally named "sdir"). Read only for context, not authoritative for current behavior.
- `README.md` — user-facing docs, kept in sync with CLI flags. Update it when changing `Args`.
- `.gitignore` contains only `/target`.
- `.ruff_cache/` is a stray Python-linter cache directory (no Python in this project); ignore it.

## Working In This Codebase

- When changing CLI behavior, update three places: the `Args` struct (clap), the relevant `process_*`/`filter`/`build_*` logic, and `README.md`'s usage/examples.
- Keep all new filtering logic inside `filter_entry` so it applies to both tree and JSON output uniformly.
- Maintain the dual stdout/buffer write pattern in any new output code.
- Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before finishing — CI enforces all three.
