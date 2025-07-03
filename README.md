# ctx

A modern, multi-shell command-line productivity logger and terminal history tool.

## Features
- Logs every command you run in your terminal (except `ctx` commands)
- Stores logs in a fast, efficient SQLite database in `~/.context/ctx.sqlite`
- Anti-abuse: ignores rapid commands (<0.5s), blacklisted commands (`ls`, `clear`, `pwd`, `history`, `exit`), consecutive duplicates, and only logs from interactive shells (TTY)
- Powerful CLI:
  - `ctx log` — View complete history (with `--less`, `--reverse`)
  - `ctx today`, `ctx weekly` — Filter by time
  - `ctx summary <folder>` — Per-project stats
  - `ctx top` — Top N most used commands
  - `ctx projects` — List all project folders with stats
  - `ctx search <pattern>` — Search history for commands
  - `ctx stats` — Overall productivity stats
  - `ctx clear` — Clear all logs (with confirmation)
  - Export/share: `--export` and `--markdown` for summaries
- Multi-shell support: bash, zsh, fish
- Easy onboarding: `ctx init` auto-detects your shell and offers to configure it

## Installation

### From crates.io
```sh
cargo install ctx
```

### From source
```sh
git clone https://github.com/piyush-itis/ctx.git
cd ctx
cargo install --path .
```

## Shell Integration
After install, run:
```sh
ctx init
```
Follow the prompt to automatically configure your shell, or copy the snippet to your shell config file.

- For bash: `~/.bashrc`
- For zsh: `~/.zshrc`
- For fish: `~/.config/fish/config.fish`

Then reload your shell:
```sh
source ~/.bashrc  # or ~/.zshrc or ~/.config/fish/config.fish
```

## Usage
```sh
ctx log [--less] [--reverse]
ctx today [--export] [--markdown]
ctx weekly [--export] [--markdown]
ctx summary <folder>
ctx top [--n <number>]
ctx projects
ctx search <pattern>
ctx stats
ctx clear
```

### Command Details
- `ctx log` — Show all logged commands. Use `--less` for pager, `--reverse` for newest first.
- `ctx today` / `ctx weekly` — Show commands from the last 24 hours or 7 days. Use `--export` or `--markdown` for summaries.
- `ctx summary <folder>` — Show stats for a specific folder/project.
- `ctx top` — Show most used commands (default top 10, configurable with `--n`).
- `ctx projects` — List all project folders with command counts and time.
- `ctx search <pattern>` — Search command history for a pattern.
- `ctx stats` — Show overall stats (total commands, time, min/max/avg duration).
- `ctx clear` — Clear all logs (asks for confirmation).
- `ctx init` — Onboard and set up shell integration.

## Anti-abuse & Credibility
- Ignores rapid-fire commands (<0.5s apart)
- Ignores blacklisted commands: `ls`, `clear`, `pwd`, `history`, `exit`
- Ignores consecutive duplicate commands
- Only logs from interactive shells (TTY)
- Excludes all `ctx` commands from logs and stats

## Data Location
- Logs are stored in `~/.context/ctx.sqlite`
- State for anti-abuse is stored in `~/.context/ctx_state`

## Security & Privacy
- **Warning:** All commands, arguments, and working directories are logged. Do not run commands with secrets (e.g., API keys, passwords) if you do not want them stored in your history.
- Log/database files are stored in your home directory and are only accessible to your user by default.

## Contributing
PRs and issues welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License
MIT 