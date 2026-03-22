use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, Select};

use crate::cli::FileTypeFilter;

pub struct SetupResult {
    pub token: String,
    pub action: SetupAction,
}

pub enum SetupAction {
    Info,
    Watch {
        chat_id: i64,
        continuous: bool,
    },
    List {
        chat_id: Option<i64>,
        file_type: Option<FileTypeFilter>,
        after: Option<String>,
        before: Option<String>,
    },
    Download {
        chat_id: Option<i64>,
        file_type: Option<FileTypeFilter>,
        interactive: bool,
        parallel: usize,
        skip_same: bool,
        dry_run: bool,
        after: Option<String>,
        before: Option<String>,
    },
    Export {
        chat_id: Option<i64>,
        file_type: Option<FileTypeFilter>,
        output: Option<String>,
    },
}

pub fn run_setup(existing_token: Option<&str>) -> Result<SetupResult> {
    println!(
        "  {}",
        style("Welcome! Let's get you set up.").cyan().bold()
    );
    println!();

    // Token
    let token: String = if let Some(t) = existing_token {
        println!(
            "  {} Using token from {}",
            style("✓").green(),
            style("$TG_BOT_TOKEN").cyan()
        );
        t.to_string()
    } else {
        Input::new()
            .with_prompt(format!(
                "  {} Bot token (from @BotFather)",
                style("?").cyan()
            ))
            .interact_text()?
    };

    println!();

    // Action
    let actions = &[
        "Check bot connection",
        "Watch a chat/channel for files",
        "List indexed files",
        "Download files",
        "Export files to JSON",
    ];

    let action_idx = Select::new()
        .with_prompt(format!(
            "  {} What would you like to do",
            style("?").cyan()
        ))
        .items(actions)
        .default(0)
        .interact()?;

    let action = match action_idx {
        0 => SetupAction::Info,
        1 => {
            println!();
            let chat_id = ask_chat_id()?;

            let mode_items = &["Watch continuously (Ctrl+C to stop)", "Watch once"];
            let mode = Select::new()
                .with_prompt(format!("  {} Polling mode", style("?").cyan()))
                .items(mode_items)
                .default(0)
                .interact()?;

            SetupAction::Watch {
                chat_id,
                continuous: mode == 0,
            }
        }
        2 => {
            println!();
            let chat_id = ask_optional_chat_id()?;
            let file_type = ask_file_type()?;
            let (after, before) = ask_date_range()?;

            SetupAction::List {
                chat_id,
                file_type,
                after,
                before,
            }
        }
        3 => {
            println!();
            let chat_id = ask_optional_chat_id()?;
            let file_type = ask_file_type()?;
            let (after, before) = ask_date_range()?;

            let parallel: usize = Input::new()
                .with_prompt(format!(
                    "  {} Parallel downloads",
                    style("?").cyan()
                ))
                .default(3)
                .interact_text()?;

            let skip_same = Confirm::new()
                .with_prompt(format!(
                    "  {} Skip already downloaded files",
                    style("?").cyan()
                ))
                .default(true)
                .interact()?;

            let dry_run = Confirm::new()
                .with_prompt(format!(
                    "  {} Dry run (simulate without downloading)",
                    style("?").cyan()
                ))
                .default(false)
                .interact()?;

            SetupAction::Download {
                chat_id,
                file_type,
                interactive: true,
                parallel,
                skip_same,
                dry_run,
                after,
                before,
            }
        }
        4 => {
            println!();
            let chat_id = ask_optional_chat_id()?;
            let file_type = ask_file_type()?;

            let output: String = Input::new()
                .with_prompt(format!(
                    "  {} Output file (leave empty for stdout)",
                    style("?").cyan()
                ))
                .allow_empty(true)
                .interact_text()?;

            SetupAction::Export {
                chat_id,
                file_type,
                output: if output.is_empty() {
                    None
                } else {
                    Some(output)
                },
            }
        }
        _ => unreachable!(),
    };

    println!();

    Ok(SetupResult { token, action })
}

fn ask_chat_id() -> Result<i64> {
    let input: i64 = Input::new()
        .with_prompt(format!(
            "  {} Chat/Channel ID (e.g. -1001234567890)",
            style("?").cyan()
        ))
        .interact_text()?;
    Ok(input)
}

fn ask_optional_chat_id() -> Result<Option<i64>> {
    let input: String = Input::new()
        .with_prompt(format!(
            "  {} Chat ID (leave empty for all)",
            style("?").cyan()
        ))
        .allow_empty(true)
        .interact_text()?;

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input.parse()?))
    }
}

fn ask_file_type() -> Result<Option<FileTypeFilter>> {
    let types = &[
        "All types",
        "PDF",
        "Image",
        "Video",
        "Audio",
        "Document",
        "Animation",
        "Voice",
    ];
    let idx = Select::new()
        .with_prompt(format!("  {} File type filter", style("?").cyan()))
        .items(types)
        .default(0)
        .interact()?;

    Ok(match idx {
        0 => None,
        1 => Some(FileTypeFilter::Pdf),
        2 => Some(FileTypeFilter::Image),
        3 => Some(FileTypeFilter::Video),
        4 => Some(FileTypeFilter::Audio),
        5 => Some(FileTypeFilter::Document),
        6 => Some(FileTypeFilter::Animation),
        7 => Some(FileTypeFilter::Voice),
        _ => None,
    })
}

fn ask_date_range() -> Result<(Option<String>, Option<String>)> {
    let use_date = Confirm::new()
        .with_prompt(format!(
            "  {} Filter by date range",
            style("?").cyan()
        ))
        .default(false)
        .interact()?;

    if !use_date {
        return Ok((None, None));
    }

    let after: String = Input::new()
        .with_prompt(format!(
            "  {} After date (YYYY-MM-DD, empty to skip)",
            style("?").cyan()
        ))
        .allow_empty(true)
        .interact_text()?;

    let before: String = Input::new()
        .with_prompt(format!(
            "  {} Before date (YYYY-MM-DD, empty to skip)",
            style("?").cyan()
        ))
        .allow_empty(true)
        .interact_text()?;

    Ok((
        if after.is_empty() { None } else { Some(after) },
        if before.is_empty() {
            None
        } else {
            Some(before)
        },
    ))
}
