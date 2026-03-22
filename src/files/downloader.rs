use anyhow::{Context, Result};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::files::catalog::FileEntry;
use crate::security;
use crate::storage::db::Database;
use crate::telegram::client::TelegramClient;

pub async fn download_files(
    client: &TelegramClient,
    db: Arc<Database>,
    files: Vec<FileEntry>,
    output_dir: &str,
    parallel: usize,
    skip_same: bool,
    dry_run: bool,
) -> Result<()> {
    if files.is_empty() {
        println!("{}", style("No files to download.").yellow());
        return Ok(());
    }

    if dry_run {
        println!(
            "  {} Dry run — {} files would be downloaded:",
            style("ℹ").blue(),
            files.len()
        );
        for file in &files {
            let local_path = build_output_path(output_dir, file);
            println!(
                "    {} → {}",
                style(file.display_name()).cyan(),
                style(local_path.display()).dim()
            );
        }
        return Ok(());
    }

    let mp = MultiProgress::new();
    let semaphore = Arc::new(Semaphore::new(parallel));
    let mut handles = Vec::new();

    let total_bar = mp.add(ProgressBar::new(files.len() as u64));
    total_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:30.green/dim}] {pos}/{len}")
            .unwrap()
            .progress_chars("━╸─"),
    );
    total_bar.set_message("Overall progress");

    for file in files {
        let sem = semaphore.clone();
        let db = db.clone();
        let mp = mp.clone();
        let total_bar = total_bar.clone();
        let output_dir = output_dir.to_string();

        // Get file path from Telegram API
        let tg_file = client
            .get_file(&file.file_id)
            .await
            .context("Failed to get file info from Telegram")?;

        let file_path = tg_file
            .file_path
            .context("Telegram did not return a file_path — file may be too large (>20MB)")?;

        let download_url = client.file_url(&file_path);

        // Enforce HTTPS
        if !security::validate_https_url(&download_url) {
            anyhow::bail!("Refusing to download over insecure HTTP: {}", download_url);
        }

        let total_size = tg_file.file_size.or(file.file_size);

        let local_path = build_output_path(&output_dir, &file);

        // Skip if file exists with same size (--skip-same)
        if skip_same {
            if let Ok(meta) = tokio::fs::metadata(&local_path).await {
                if let Some(expected) = total_size {
                    if meta.len() == expected {
                        total_bar.inc(1);
                        continue; // Skip — already downloaded
                    }
                }
            }
        }

        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let pb = mp.insert_before(&total_bar, ProgressBar::new(total_size.unwrap_or(0)));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.cyan} {msg}\n    [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")
                .unwrap()
                .progress_chars("━╸─"),
        );
        pb.set_message(truncate_name(&file.display_name(), 40));

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            let result = download_with_curl(&download_url, &local_path, total_size, &pb).await;

            match &result {
                Ok(()) => {
                    pb.finish_with_message(format!(
                        "{} {}",
                        style("✓").green(),
                        truncate_name(&file.display_name(), 40)
                    ));
                    let _ = db.mark_downloaded(&file.file_unique_id);
                }
                Err(e) => {
                    pb.finish_with_message(format!(
                        "{} {} — {}",
                        style("✗").red(),
                        truncate_name(&file.display_name(), 30),
                        e
                    ));
                }
            }

            total_bar.inc(1);
            result
        });

        handles.push(handle);
    }

    let mut success = 0u64;
    let mut failed = 0u64;

    for handle in handles {
        match handle.await? {
            Ok(()) => success += 1,
            Err(_) => failed += 1,
        }
    }

    total_bar.finish_and_clear();

    println!();
    println!(
        "  {} {} downloaded, {} failed",
        style("Done!").green().bold(),
        success,
        failed
    );

    Ok(())
}

/// Download a file using curl with security hardening.
///
/// Security measures:
/// - HTTPS enforced (--proto =https)
/// - TLS verification enabled (default, never disabled)
/// - No shell interpolation (args passed directly, not via shell)
/// - Redirect limit (--max-redirs 5)
/// - Timeout protection (--connect-timeout, --max-time)
/// - Output path is pre-sanitized
async fn download_with_curl(
    url: &str,
    local_path: &Path,
    total_size: Option<u64>,
    pb: &ProgressBar,
) -> Result<()> {
    use tokio::process::Command;

    // Refuse symlink targets to prevent symlink attacks
    if local_path.is_symlink() {
        anyhow::bail!("Refusing to write to symlink: {}", local_path.display());
    }

    // Check for existing partial download
    let resume = if local_path.exists() {
        let meta = tokio::fs::metadata(local_path).await?;
        let existing = meta.len();
        if let Some(total) = total_size {
            if existing >= total {
                pb.set_position(total);
                return Ok(()); // Already complete
            }
        }
        pb.set_position(existing);
        true
    } else {
        false
    };

    let local_str = local_path.to_string_lossy().to_string();

    // Build hardened curl command
    let mut cmd = Command::new("curl");
    cmd.arg("-L") // follow redirects
        .arg("-f") // fail on HTTP errors
        .arg("-s") // silent
        .arg("--proto")
        .arg("=https") // HTTPS only
        .arg("--max-redirs")
        .arg("5") // limit redirect hops
        .arg("--connect-timeout")
        .arg("30") // connection timeout
        .arg("--max-time")
        .arg("3600") // max 1 hour per file
        .arg("--retry")
        .arg("3") // retry on transient failures
        .arg("--retry-delay")
        .arg("2") // 2s between retries
        .arg("-o")
        .arg(&local_str);

    if resume {
        cmd.arg("-C").arg("-");
    }

    cmd.stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null());

    let mut child = cmd
        .arg(url)
        .spawn()
        .context("Failed to run curl. Make sure curl is installed on your system.")?;

    // Monitor file size for progress
    let path_clone = local_path.to_path_buf();
    let pb_clone = pb.clone();
    let monitor = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            if let Ok(meta) = tokio::fs::metadata(&path_clone).await {
                pb_clone.set_position(meta.len());
            }
        }
    });

    let status = child.wait().await.context("curl process failed")?;
    monitor.abort();

    // Final file size update
    if let Ok(meta) = tokio::fs::metadata(local_path).await {
        pb.set_position(meta.len());
    }

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        // curl exit code 33 = range not supported, retry without resume
        if code == 33 && resume {
            let retry_status = Command::new("curl")
                .arg("-L")
                .arg("-f")
                .arg("-s")
                .arg("--proto")
                .arg("=https")
                .arg("--max-redirs")
                .arg("5")
                .arg("--connect-timeout")
                .arg("30")
                .arg("--max-time")
                .arg("3600")
                .arg("-o")
                .arg(&local_str)
                .arg(url)
                .status()
                .await?;

            if !retry_status.success() {
                anyhow::bail!(
                    "curl failed with exit code {}",
                    retry_status.code().unwrap_or(-1)
                );
            }
        } else {
            anyhow::bail!("curl failed with exit code {}", code);
        }
    }

    // Set restrictive permissions on downloaded file (non-Windows)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o644);
        let _ = std::fs::set_permissions(local_path, perms);
    }

    Ok(())
}

fn build_output_path(output_dir: &str, entry: &FileEntry) -> PathBuf {
    let subdir = entry.file_type.subdir();
    let name = entry.display_name(); // Already sanitized via security::sanitize_filename
    security::safe_output_path(output_dir, subdir, &name)
}

fn truncate_name(name: &str, max: usize) -> String {
    if name.len() <= max {
        name.to_string()
    } else {
        format!("{}…", &name[..max - 1])
    }
}
