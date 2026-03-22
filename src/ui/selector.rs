use console::style;
use dialoguer::MultiSelect;

use crate::files::catalog::FileEntry;

pub fn select_files(files: &[FileEntry]) -> anyhow::Result<Vec<FileEntry>> {
    if files.is_empty() {
        println!("  {}", style("No files available for selection.").yellow());
        return Ok(Vec::new());
    }

    let items: Vec<String> = files
        .iter()
        .map(|f| {
            format!(
                "[{}] {} ({})",
                f.file_type,
                f.display_name(),
                f.display_size()
            )
        })
        .collect();

    println!();
    println!(
        "  {} Use space to select, enter to confirm:",
        style("Select files to download").cyan().bold()
    );
    println!();

    let selected = MultiSelect::new().items(&items).interact()?;

    let result = selected.into_iter().map(|i| files[i].clone()).collect();

    Ok(result)
}
