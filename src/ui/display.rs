use chrono::DateTime;
use console::style;

use crate::files::catalog::FileEntry;
use crate::telegram::types::User;

pub fn print_banner() {
    println!();
    println!(
        "  {}",
        style("╔════════════════════════════════════════╗").cyan()
    );
    println!(
        "  {}",
        style("║  tg-snatch  ·  Snatch from Telegram   ║")
            .cyan()
            .bold()
    );
    println!(
        "  {}",
        style("╚════════════════════════════════════════╝").cyan()
    );
    println!();
}

pub fn print_bot_info(user: &User) {
    println!(
        "  {} {}",
        style("Bot:").bold(),
        style(&user.first_name).green()
    );
    if let Some(username) = &user.username {
        println!(
            "  {} @{}",
            style("Username:").bold(),
            style(username).cyan()
        );
    }
    println!("  {} {}", style("ID:").bold(), style(user.id).dim());
    println!(
        "  {} {}",
        style("Status:").bold(),
        style("Connected").green().bold()
    );
    println!();
}

pub fn print_file_table(files: &[FileEntry]) {
    if files.is_empty() {
        println!("  {}", style("No files found.").yellow());
        return;
    }

    // Header
    println!(
        "  {:<4} {:<10} {:<35} {:<10} {:<20} {}",
        style("#").bold().dim(),
        style("TYPE").bold().dim(),
        style("NAME").bold().dim(),
        style("SIZE").bold().dim(),
        style("DATE").bold().dim(),
        style("STATUS").bold().dim(),
    );
    println!("  {}", style("─".repeat(90)).dim());

    for (i, file) in files.iter().enumerate() {
        let type_styled = match file.file_type {
            crate::files::filter::FileType::Pdf => style(format!("{:<10}", "PDF")).red(),
            crate::files::filter::FileType::Image => style(format!("{:<10}", "IMAGE")).green(),
            crate::files::filter::FileType::Video => style(format!("{:<10}", "VIDEO")).magenta(),
            crate::files::filter::FileType::Audio => style(format!("{:<10}", "AUDIO")).blue(),
            crate::files::filter::FileType::Document => style(format!("{:<10}", "DOC")).white(),
            crate::files::filter::FileType::Animation => style(format!("{:<10}", "GIF")).yellow(),
            crate::files::filter::FileType::Voice => style(format!("{:<10}", "VOICE")).cyan(),
        };

        let name = truncate(&file.display_name(), 35);
        let size = format!("{:<10}", file.display_size());

        let date = DateTime::from_timestamp(file.date, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let status = if file.downloaded {
            style("downloaded").green()
        } else {
            style("pending").yellow()
        };

        println!(
            "  {:<4} {} {:<35} {} {:<20} {}",
            style(i + 1).dim(),
            type_styled,
            style(&name).white(),
            style(&size).dim(),
            style(&date).dim(),
            status,
        );
    }

    println!();
    println!("  {} files total", style(files.len()).cyan().bold());
    println!();
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
