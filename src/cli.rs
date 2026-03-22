use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "tg-snatch",
    about = "Snatch files from Telegram — fast, secure, beautiful.",
    version,
    author,
    after_help = "Run with no arguments for interactive setup mode."
)]
pub struct Cli {
    /// Telegram Bot API token
    #[arg(long, env = "TG_BOT_TOKEN", global = true)]
    pub token: Option<String>,

    /// Output directory for downloads
    #[arg(long, default_value = "./downloads", global = true)]
    pub output_dir: String,

    /// Increase verbosity
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Interactive guided setup — asks for token, chat ID, etc.
    Setup,

    /// Show bot info and connection status
    Info,

    /// Poll for new files from a chat/channel and index them
    #[command(allow_negative_numbers = true)]
    Watch {
        /// Chat or channel ID to watch
        #[arg(long)]
        chat_id: i64,

        /// Run continuously (Ctrl+C to stop)
        #[arg(long)]
        continuous: bool,

        /// Duration to poll in seconds (ignored if --continuous)
        #[arg(long)]
        duration: Option<u64>,
    },

    /// List indexed files
    #[command(allow_negative_numbers = true)]
    List {
        /// Filter by chat ID
        #[arg(long)]
        chat_id: Option<i64>,

        /// Filter by file type
        #[arg(long, value_enum)]
        r#type: Option<FileTypeFilter>,

        /// Maximum number of files to show
        #[arg(long, default_value = "50")]
        limit: usize,

        /// Only files after this date (YYYY-MM-DD)
        #[arg(long)]
        after: Option<String>,

        /// Only files before this date (YYYY-MM-DD)
        #[arg(long)]
        before: Option<String>,
    },

    /// Download files
    #[command(allow_negative_numbers = true)]
    Download {
        /// Filter by chat ID
        #[arg(long)]
        chat_id: Option<i64>,

        /// Filter by file type
        #[arg(long, value_enum)]
        r#type: Option<FileTypeFilter>,

        /// Download all matching files
        #[arg(long)]
        all: bool,

        /// Interactive file selection
        #[arg(short, long)]
        interactive: bool,

        /// Number of parallel downloads
        #[arg(long, default_value = "3")]
        parallel: usize,

        /// Specific file IDs to download
        #[arg(long)]
        file_id: Vec<String>,

        /// Skip files that already exist with matching size
        #[arg(long)]
        skip_same: bool,

        /// Include only these extensions (comma-separated, e.g. "pdf,jpg")
        #[arg(short = 'I', long, value_delimiter = ',')]
        include_ext: Vec<String>,

        /// Exclude these extensions (comma-separated, e.g. "mp4,avi")
        #[arg(short = 'E', long, value_delimiter = ',')]
        exclude_ext: Vec<String>,

        /// Simulate download without writing files
        #[arg(long)]
        dry_run: bool,

        /// Download newest files first
        #[arg(long)]
        desc: bool,

        /// Only files after this date (YYYY-MM-DD)
        #[arg(long)]
        after: Option<String>,

        /// Only files before this date (YYYY-MM-DD)
        #[arg(long)]
        before: Option<String>,
    },

    /// Export indexed files to JSON
    #[command(allow_negative_numbers = true)]
    Export {
        /// Filter by chat ID
        #[arg(long)]
        chat_id: Option<i64>,

        /// Filter by file type
        #[arg(long, value_enum)]
        r#type: Option<FileTypeFilter>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum FileTypeFilter {
    Pdf,
    Image,
    Video,
    Audio,
    Document,
    Animation,
    Voice,
    All,
}
