use anyhow::{bail, Result};
use clap::Parser;
use console::style;
use std::sync::Arc;

use tg_snatch::cli::{self, Cli, Command};
use tg_snatch::files;
use tg_snatch::security;
use tg_snatch::storage::db::Database;
use tg_snatch::telegram;
use tg_snatch::telegram::client::TelegramClient;
use tg_snatch::ui;
use tg_snatch::ui::display::{print_banner, print_bot_info, print_file_table};
use tg_snatch::ui::setup::{self, SetupAction};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("tg_snatch=debug")
            .init();
    }

    print_banner();

    match &cli.command {
        None | Some(Command::Setup) => run_interactive(&cli).await,
        Some(Command::Info) => cmd_info(&cli).await,
        Some(Command::Watch {
            chat_id,
            continuous,
            duration,
        }) => cmd_watch(&cli, *chat_id, *continuous, *duration).await,
        Some(Command::List {
            chat_id,
            r#type,
            limit,
            after,
            before,
        }) => cmd_list(
            *chat_id,
            r#type.clone(),
            *limit,
            after.clone(),
            before.clone(),
        ),
        Some(Command::Download {
            chat_id,
            r#type,
            all,
            interactive,
            parallel,
            file_id,
            skip_same,
            include_ext,
            exclude_ext,
            dry_run,
            desc,
            after,
            before,
        }) => {
            cmd_download(
                &cli,
                *chat_id,
                r#type.clone(),
                *all,
                *interactive,
                *parallel,
                file_id.clone(),
                *skip_same,
                include_ext.clone(),
                exclude_ext.clone(),
                *dry_run,
                *desc,
                after.clone(),
                before.clone(),
            )
            .await
        }
        Some(Command::Export {
            chat_id,
            r#type,
            output,
        }) => cmd_export(*chat_id, r#type.clone(), output.clone()),
    }
}

async fn run_interactive(cli: &Cli) -> Result<()> {
    let result = setup::run_setup(cli.token.as_deref())?;

    match result.action {
        SetupAction::Info => {
            let client = TelegramClient::new(result.token);
            println!("  {} Connecting to Telegram...", style("●").cyan());
            let user = client.get_me().await?;
            print_bot_info(&user);
        }
        SetupAction::Watch {
            chat_id,
            continuous,
        } => {
            let client = TelegramClient::new(result.token);
            let user = client.get_me().await?;
            println!(
                "  {} Connected as {}",
                style("●").green(),
                style(&user.first_name).cyan()
            );
            println!(
                "  {} Watching chat {}",
                style("●").green(),
                style(chat_id).cyan()
            );
            println!();
            let db = Arc::new(Database::open()?);
            telegram::poller::watch(&client, db, chat_id, continuous, None).await?;
        }
        SetupAction::List {
            chat_id,
            file_type,
            after,
            before,
        } => {
            let db = Database::open()?;
            let files = db.query_files(chat_id, file_type.as_ref(), 50, false)?;
            let files =
                files::filter::filter_by_date_range(files, after.as_deref(), before.as_deref())?;
            if files.is_empty() {
                println!(
                    "  {} No files indexed yet. Run {} first.",
                    style("ℹ").blue(),
                    style("tg-snatch watch").cyan()
                );
            } else {
                print_file_table(&files);
            }
        }
        SetupAction::Download {
            chat_id,
            file_type,
            interactive,
            parallel,
            skip_same,
            dry_run,
            after,
            before,
        } => {
            let client = TelegramClient::new(result.token);
            let db = Arc::new(Database::open()?);
            let all_files = db.query_files(chat_id, file_type.as_ref(), 10000, true)?;
            let all_files = files::filter::filter_by_date_range(
                all_files,
                after.as_deref(),
                before.as_deref(),
            )?;

            if all_files.is_empty() {
                println!(
                    "  {} No files to download. Run {} first.",
                    style("ℹ").blue(),
                    style("tg-snatch watch").cyan()
                );
                return Ok(());
            }

            let to_download = if interactive {
                ui::selector::select_files(&all_files)?
            } else {
                all_files
            };

            if to_download.is_empty() {
                println!("  {}", style("No files selected.").yellow());
                return Ok(());
            }

            if !dry_run {
                println!(
                    "  {} Downloading {} files...",
                    style("●").green(),
                    style(to_download.len()).cyan().bold()
                );
            }
            println!();

            files::downloader::download_files(
                &client,
                db,
                to_download,
                &cli.output_dir,
                parallel,
                skip_same,
                dry_run,
            )
            .await?;
        }
        SetupAction::Export {
            chat_id,
            file_type,
            output,
        } => {
            cmd_export(chat_id, file_type, output)?;
        }
    }

    Ok(())
}

fn get_token(cli: &Cli) -> Result<String> {
    let token = cli.token.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "Bot token required. Use {} or set {}",
            style("--token <TOKEN>").cyan(),
            style("TG_BOT_TOKEN").cyan()
        )
    })?;

    // Validate token format
    if !security::validate_token_format(&token) {
        bail!(
            "Invalid bot token format. Expected format: {}",
            style("123456:ABCdefGHIjklMNO").cyan()
        );
    }

    Ok(token)
}

async fn cmd_info(cli: &Cli) -> Result<()> {
    let token = get_token(cli)?;
    let client = TelegramClient::new(token);

    println!("  {} Connecting to Telegram...", style("●").cyan());
    let user = client.get_me().await?;
    print_bot_info(&user);
    Ok(())
}

async fn cmd_watch(cli: &Cli, chat_id: i64, continuous: bool, duration: Option<u64>) -> Result<()> {
    let token = get_token(cli)?;
    let client = TelegramClient::new(token);

    let user = client.get_me().await?;
    println!(
        "  {} Connected as {}",
        style("●").green(),
        style(&user.first_name).cyan()
    );
    println!(
        "  {} Watching chat {}",
        style("●").green(),
        style(chat_id).cyan()
    );

    if !continuous && duration.is_none() {
        println!(
            "  {} Tip: use {} for continuous polling or {} to set a duration",
            style("ℹ").blue(),
            style("--continuous").cyan(),
            style("--duration <SECS>").cyan()
        );
    }
    println!();

    let db = Arc::new(Database::open()?);
    telegram::poller::watch(&client, db, chat_id, continuous, duration).await
}

fn cmd_list(
    chat_id: Option<i64>,
    file_type: Option<cli::FileTypeFilter>,
    limit: usize,
    after: Option<String>,
    before: Option<String>,
) -> Result<()> {
    let db = Database::open()?;
    let files = db.query_files(chat_id, file_type.as_ref(), limit, false)?;

    // Apply date range filter
    let files = files::filter::filter_by_date_range(files, after.as_deref(), before.as_deref())?;

    if files.is_empty() {
        println!(
            "  {} No files indexed yet. Run {} first to discover files.",
            style("ℹ").blue(),
            style("tg-snatch watch --chat-id <ID> --continuous").cyan()
        );
        return Ok(());
    }

    print_file_table(&files);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn cmd_download(
    cli: &Cli,
    chat_id: Option<i64>,
    file_type: Option<cli::FileTypeFilter>,
    all: bool,
    interactive: bool,
    parallel: usize,
    file_ids: Vec<String>,
    skip_same: bool,
    include_ext: Vec<String>,
    exclude_ext: Vec<String>,
    dry_run: bool,
    desc: bool,
    after: Option<String>,
    before: Option<String>,
) -> Result<()> {
    // Validate: cannot combine --include-ext and --exclude-ext
    if !include_ext.is_empty() && !exclude_ext.is_empty() {
        bail!(
            "Cannot combine {} and {} — use one or the other",
            style("--include-ext").cyan(),
            style("--exclude-ext").cyan()
        );
    }

    let token = get_token(cli)?;
    let client = TelegramClient::new(token);
    let db = Arc::new(Database::open()?);

    let mut files = if !file_ids.is_empty() {
        let all_files = db.query_files(None, None, 10000, false)?;
        all_files
            .into_iter()
            .filter(|f| file_ids.contains(&f.file_unique_id) || file_ids.contains(&f.file_id))
            .collect::<Vec<_>>()
    } else {
        db.query_files(chat_id, file_type.as_ref(), 10000, !all)?
    };

    // Apply extension filters
    if !include_ext.is_empty() {
        let exts: Vec<String> = include_ext.iter().map(|e| e.to_lowercase()).collect();
        files.retain(|f| {
            let name = f.display_name().to_lowercase();
            exts.iter().any(|ext| name.ends_with(&format!(".{}", ext)))
        });
    }
    if !exclude_ext.is_empty() {
        let exts: Vec<String> = exclude_ext.iter().map(|e| e.to_lowercase()).collect();
        files.retain(|f| {
            let name = f.display_name().to_lowercase();
            !exts.iter().any(|ext| name.ends_with(&format!(".{}", ext)))
        });
    }

    // Date range filter
    files = files::filter::filter_by_date_range(files, after.as_deref(), before.as_deref())?;

    // Sort order
    if desc {
        files.sort_by(|a, b| b.date.cmp(&a.date));
    }

    if files.is_empty() {
        println!(
            "  {} No matching files found. Run {} to index files first.",
            style("ℹ").blue(),
            style("tg-snatch watch").cyan()
        );
        return Ok(());
    }

    let to_download = if interactive {
        ui::selector::select_files(&files)?
    } else if all || !file_ids.is_empty() {
        files
    } else {
        bail!(
            "Specify {} for interactive selection, {} to download all, or {} for specific files",
            style("--interactive").cyan(),
            style("--all").cyan(),
            style("--file-id").cyan()
        );
    };

    if to_download.is_empty() {
        println!("  {}", style("No files selected.").yellow());
        return Ok(());
    }

    if dry_run {
        println!(
            "  {} Dry run — would download {} files",
            style("ℹ").blue(),
            style(to_download.len()).cyan().bold()
        );
    } else {
        println!(
            "  {} Downloading {} files...",
            style("●").green(),
            style(to_download.len()).cyan().bold()
        );
    }
    println!();

    files::downloader::download_files(
        &client,
        db,
        to_download,
        &cli.output_dir,
        parallel,
        skip_same,
        dry_run,
    )
    .await
}

fn cmd_export(
    chat_id: Option<i64>,
    file_type: Option<cli::FileTypeFilter>,
    output: Option<String>,
) -> Result<()> {
    let db = Database::open()?;
    let files = db.query_files(chat_id, file_type.as_ref(), 100000, false)?;

    if files.is_empty() {
        println!(
            "  {} No files to export. Run {} first.",
            style("ℹ").blue(),
            style("tg-snatch watch").cyan()
        );
        return Ok(());
    }

    // Build JSON output
    let export_data: Vec<serde_json::Value> = files
        .iter()
        .map(|f| {
            serde_json::json!({
                "file_id": f.file_id,
                "file_unique_id": f.file_unique_id,
                "file_type": f.file_type.as_str(),
                "file_name": f.file_name,
                "file_size": f.file_size,
                "mime_type": f.mime_type,
                "chat_id": f.chat_id,
                "message_id": f.message_id,
                "date": f.date,
                "caption": f.caption,
                "downloaded": f.downloaded,
            })
        })
        .collect();

    let json = serde_json::to_string_pretty(&export_data)?;

    match output {
        Some(path) => {
            std::fs::write(&path, &json)?;
            println!(
                "  {} Exported {} files to {}",
                style("✓").green(),
                style(files.len()).cyan().bold(),
                style(&path).cyan()
            );
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}
