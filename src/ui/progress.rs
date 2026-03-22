use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

/// Spinner style presets
pub enum SpinnerType {
    /// Green dots — used for polling/watching
    Watch,
    /// Cyan dots — used for connecting
    Connect,
    /// Blue dots — used for indexing/processing
    Process,
}

/// Create a styled spinner with a message.
pub fn spinner(spin_type: SpinnerType, msg: &str) -> ProgressBar {
    let (template, tick_ms) = match spin_type {
        SpinnerType::Watch => ("{spinner:.green} {msg}", 100),
        SpinnerType::Connect => ("{spinner:.cyan} {msg}", 80),
        SpinnerType::Process => ("{spinner:.blue} {msg}", 120),
    };

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template(template).unwrap());
    pb.enable_steady_tick(Duration::from_millis(tick_ms));
    pb.set_message(msg.to_string());
    pb
}

/// Create a download progress bar for a single file.
pub fn download_bar(total_size: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "  {spinner:.cyan} {msg}\n    [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})",
            )
            .unwrap()
            .progress_chars("━╸─"),
    );
    pb
}

/// Create an overall progress bar for tracking multiple downloads.
pub fn overall_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:30.green/dim}] {pos}/{len}")
            .unwrap()
            .progress_chars("━╸─"),
    );
    pb.set_message("Overall progress");
    pb
}

/// Create a multi-progress container and attach an overall bar.
/// Returns (MultiProgress, overall_bar).
pub fn multi_download(total_files: u64) -> (MultiProgress, ProgressBar) {
    let mp = MultiProgress::new();
    let total = mp.add(overall_bar(total_files));
    (mp, total)
}

/// Insert a file download bar above the overall bar in a MultiProgress.
pub fn add_file_bar(mp: &MultiProgress, total_bar: &ProgressBar, total_size: u64) -> ProgressBar {
    let pb = mp.insert_before(total_bar, ProgressBar::new(total_size));
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "  {spinner:.cyan} {msg}\n    [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})",
            )
            .unwrap()
            .progress_chars("━╸─"),
    );
    pb
}

/// Create a counting progress bar (e.g., for export/indexing operations).
pub fn count_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {spinner:.green} {msg} [{bar:30.green/dim}] {pos}/{len}")
            .unwrap()
            .progress_chars("━╸─"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}
