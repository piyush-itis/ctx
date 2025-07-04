# prynt

A modern, multi-shell command-line productivity logger and terminal history tool.

## Features
- Logs every command you run in your terminal (except `prynt` commands)
- Stores logs in a fast, efficient SQLite database in `~/.context/prynt.sqlite`
- Powerful CLI:
  - `prynt log` — View complete history (with `--less`, `--reverse`)
  - `prynt today`, `prynt weekly` — Filter by time
  - `prynt summary <folder>` — Per-project stats
  - `prynt top` — Top N most used commands
  - `prynt projects` — List all project folders with stats
  - `prynt search <pattern>` — Search history for commands
  - `prynt stats` — Overall productivity stats
  - `prynt clear` — Clear all logs (with confirmation)
  - Export/share: `--export` and `--markdown` for summaries
- Multi-shell support: bash, zsh, fish
- Easy onboarding: `prynt init` auto-detects your shell and offers to configure it

## Installation

### From crates.io
```sh
cargo install prynt
```

### From source
```sh
git clone https://github.com/piyush-itis/prynt.git
cd prynt
cargo install --path .
```

## Shell Integration
After install, run:
```sh
prynt init
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
prynt log [--less] [--reverse]
prynt today [--export] [--markdown]
prynt weekly [--export] [--markdown]
prynt summary <folder>
prynt top [--n <number>]
prynt projects
prynt search <pattern>
prynt stats
prynt clear
```

### Command Details
- `prynt log` — Show all logged commands. Use `--less` for pager, `--reverse` for newest first.
- `prynt today` / `prynt weekly` — Show commands from the last 24 hours or 7 days. Use `--export` or `--markdown` for summaries.
- `prynt summary <folder>` — Show stats for a specific folder/project.
- `prynt top` — Show most used commands (default top 10, configurable with `--n`).
- `prynt projects` — List all project folders with command counts and time.
- `prynt search <pattern>` — Search command history for a pattern.
- `prynt stats` — Show overall stats (total commands, time, min/max/avg duration).
- `prynt clear` — Clear all logs (asks for confirmation).
- `prynt init` — Onboard and set up shell integration.

## Data Location
- Logs are stored in `~/.context/prynt.sqlite`
- State for anti-abuse is stored in `~/.context/prynt_state`

## Security & Privacy
- **Warning:** All commands, arguments, and working directories are logged. Do not run commands with secrets (e.g., API keys, passwords) if you do not want them stored in your history.
- Log/database files are stored in your home directory and are only accessible to your user by default.

## Contributing
PRs and issues welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License
MIT

[![Crates.io](https://img.shields.io/crates/v/prynt.svg)](https://crates.io/crates/prynt) 