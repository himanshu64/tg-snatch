# tg-snatch — Snatch Files from Telegram

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![Release](https://img.shields.io/badge/release-v0.1.0-blue)
![Downloads](https://img.shields.io/badge/downloads-0-lightgrey)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-informational)

A beautiful, secure CLI tool to snatch PDFs, images, videos, audio, and other files from Telegram channels and chats. Built with Rust.

```
  ╔════════════════════════════════════════╗
  ║  tg-snatch  ·  Snatch from Telegram   ║
  ╚════════════════════════════════════════╝
```

## Features

- **Interactive mode** — Run with no arguments for a guided step-by-step setup. No flags needed.
- **Beautiful CLI** — Colored output, progress bars with speed & ETA, spinners, styled file table
- **Downloads via curl** — Uses system curl for maximum compatibility, speed, and bandwidth utilization
- **Multiple file types** — PDF, image, video, audio, document, animation, voice
- **Interactive selection** — Cherry-pick files with a multi-select prompt
- **Parallel downloads** — Concurrent downloads (default: 3, configurable)
- **Resumable downloads** — Interrupted downloads resume where they left off
- **Skip duplicates** — `--skip-same` skips files already downloaded with matching size
- **Extension filters** — Include (`-I pdf,jpg`) or exclude (`-E mp4`) by file extension
- **Date range filtering** — Download files from a specific time period (`--after`, `--before`)
- **Dry run** — Preview what would be downloaded without writing files
- **Export to JSON** — Export your file index for use with external tools
- **Persistent index** — SQLite database persists across sessions
- **Single binary** — No runtime dependencies beyond curl
- **Cross-platform** — Works on macOS, Linux, and Windows

## Security

- **HTTPS enforced** — All API calls and downloads use HTTPS only (`--proto =https`)
- **TLS verification** — Certificate validation always enabled
- **Path traversal protection** — File names from Telegram are sanitized to prevent `../../` attacks
- **No shell injection** — Arguments passed directly to curl, never through a shell
- **Token masking** — Bot tokens are never logged; displayed as `1234…abcd` in errors
- **Token validation** — Format checked before making any API calls
- **Symlink protection** — Refuses to write to symlink targets
- **Safe file permissions** — Downloaded files get `644` permissions on Unix
- **Redirect limits** — curl limited to 5 redirects
- **Timeouts** — Connection timeout (30s) and max transfer time (1h) prevent hangs

## Installation

### macOS (Homebrew)

```bash
brew tap himanshusharma/tap
brew install tg-snatch
```

### macOS / Linux (quick install)

```bash
curl -fsSL https://raw.githubusercontent.com/himanshu64/tg-snatch/main/packaging/scripts/install.sh | bash
```

### Windows (Scoop)

```powershell
scoop bucket add tg-snatch https://github.com/himanshusharma/scoop-bucket
scoop install tg-snatch
```

### Windows (PowerShell installer)

```powershell
irm https://raw.githubusercontent.com/himanshu64/tg-snatch/main/packaging/scripts/install.ps1 | iex
```

### Arch Linux (AUR)

```bash
yay -S tg-snatch-bin
```

### Cargo (all platforms)

```bash
cargo install tg-snatch
```

### From source

```bash
git clone https://github.com/himanshu64/tg-snatch.git
cd tg-snatch
cargo install --path .
```

### Prerequisites

- curl (pre-installed on macOS/Linux, available on Windows)
- curl (pre-installed on macOS/Linux, available on Windows)
- A Telegram Bot token (from [@BotFather](https://t.me/BotFather))

## Quick Start (Interactive Mode)

Just run `tg-snatch` with no arguments:

```bash
$ tg-snatch

  ╔════════════════════════════════════════╗
  ║  tg-snatch  ·  Snatch from Telegram   ║
  ╚════════════════════════════════════════╝

  Welcome! Let's get you set up.

  ? Bot token (from @BotFather): 123456:ABC...

  ? What would you like to do
  > Check bot connection
    Watch a chat/channel for files
    List indexed files
    Download files
    Export files to JSON

  ? Chat ID (leave empty for all): -1001234567890

  ? File type filter
  > All types
    PDF
    Image
    Video
    ...

  ? Filter by date range? Yes
  ? After date (YYYY-MM-DD): 2025-01-01
  ? Before date (YYYY-MM-DD): 2025-06-01

  ? Parallel downloads: 3
  ? Skip already downloaded files? Yes
  ? Dry run? No
```

The interactive mode walks you through everything. No flags to memorize.

## Setup

1. **Create a bot** — Talk to [@BotFather](https://t.me/BotFather) on Telegram and create a new bot. Copy the token.

2. **Add the bot to your channel/group** — Add the bot as an admin to the channel or group.

3. **Get the chat ID** — Forward a message from the channel to [@userinfobot](https://t.me/userinfobot). Channel IDs are typically negative (e.g., `-1001234567890`).

4. **Set your token**:

```bash
export TG_BOT_TOKEN=your_bot_token_here
```

## Commands

All commands also work in interactive mode — just run `tg-snatch` or `tg-snatch setup`.

### Check bot connection

```bash
tg-snatch info
```

### Watch and index files

```bash
# Watch continuously (Ctrl+C to stop)
tg-snatch watch --chat-id -1001234567890 --continuous

# Watch for 5 minutes
tg-snatch watch --chat-id -1001234567890 --duration 300
```

### List indexed files

```bash
tg-snatch list
tg-snatch list --type pdf
tg-snatch list --chat-id -1001234567890 --after 2025-01-01 --before 2025-06-01
```

### Download files

```bash
# Interactive selection (recommended)
tg-snatch download --interactive

# Download all PDFs
tg-snatch download --type pdf --all

# Download with filters
tg-snatch download --all --include-ext pdf,docx --after 2025-03-01

# Skip already downloaded, 5 parallel threads
tg-snatch download --all --skip-same --parallel 5

# Exclude video files
tg-snatch download --all --exclude-ext mp4,avi,mkv

# Preview without downloading
tg-snatch download --all --dry-run

# Newest files first
tg-snatch download --all --desc
```

### Export to JSON

```bash
# Export to file
tg-snatch export --output files.json

# Export only PDFs to stdout
tg-snatch export --type pdf

# Pipe to jq for processing
tg-snatch export | jq '.[] | select(.file_size > 1000000)'
```

## All Options

### Global

| Option | Description |
|--------|-------------|
| `--token <TOKEN>` | Bot API token (or `TG_BOT_TOKEN` env var) |
| `--output-dir <DIR>` | Download directory (default: `./downloads`) |
| `-v, --verbose` | Debug logging |

### Download flags

| Flag | Description |
|------|-------------|
| `--all` | Download all matching files |
| `-i, --interactive` | Interactive file picker |
| `--parallel <N>` | Concurrent downloads (default: 3) |
| `--skip-same` | Skip files already downloaded with matching size |
| `-I, --include-ext <ext,..>` | Only download these extensions |
| `-E, --exclude-ext <ext,..>` | Exclude these extensions |
| `--after <YYYY-MM-DD>` | Only files after this date |
| `--before <YYYY-MM-DD>` | Only files before this date |
| `--dry-run` | Simulate without downloading |
| `--desc` | Download newest first |
| `--file-id <ID>` | Download specific files by ID |

### File types

`pdf`, `image`, `video`, `audio`, `document`, `animation`, `voice`, `all`

## Project Structure

```
src/
├── main.rs              # Entry point and command dispatch
├── lib.rs               # Public module exports
├── cli.rs               # Clap CLI definitions
├── security.rs          # Input sanitization, path safety, token masking
├── telegram/
│   ├── client.rs        # HTTPS-only Telegram Bot API client
│   ├── types.rs         # Serde types for API responses
│   └── poller.rs        # Long-polling update watcher
├── files/
│   ├── catalog.rs       # FileEntry struct + display helpers
│   ├── filter.rs        # File type classification + date range filtering
│   └── downloader.rs    # curl-based downloads with progress + resume
├── storage/
│   └── db.rs            # SQLite persistence layer
└── ui/
    ├── display.rs       # Colored tables, banners, bot info
    ├── progress.rs      # Shared progress helpers
    ├── selector.rs      # Interactive multi-select prompt
    └── setup.rs         # Interactive guided setup wizard
```

## How It Works

1. **Indexing** — `watch` polls `getUpdates` with long polling. Files are classified by type and stored in SQLite at `~/.local/share/tg-snatch/index.db`.

2. **Listing** — `list` queries the local database with filters (type, chat, date range) and displays a color-coded table.

3. **Downloading** — `download` gets file metadata via `getFile`, then streams via `curl` with progress monitoring. Files are organized into subdirectories (`pdfs/`, `images/`, `videos/`, etc.).

4. **Exporting** — `export` dumps the file index as JSON for use with external tools.

## Limitations

- **20 MB file size limit** — The Telegram Bot API limits downloads to 20 MB. For larger files, use a [local Bot API server](https://core.telegram.org/bots/api#using-a-local-bot-api-server).
- **No chat history** — Bots cannot browse past messages. `watch` only captures new messages. Forward older messages to the bot to index them.
- **Bot must be admin** — For channels, the bot must be added as an administrator.

## License

MIT
