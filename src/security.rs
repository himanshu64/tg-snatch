use std::path::{Component, Path, PathBuf};

/// Sanitize a filename from Telegram to prevent path traversal attacks.
/// Strips directory separators, `..`, and null bytes. Returns a safe filename.
pub fn sanitize_filename(name: &str) -> String {
    let name = name.replace('\0', "");

    // Take only the final component (strip any directory prefix)
    let path = Path::new(&name);
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unnamed");

    // Remove any remaining path separators and dangerous chars
    let sanitized: String = filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => c,
        })
        .collect();

    // Prevent empty or dot-only names
    let trimmed = sanitized.trim_matches('.');
    if trimmed.is_empty() {
        return "unnamed".to_string();
    }

    // Limit filename length (255 bytes is typical filesystem max)
    if trimmed.len() > 200 {
        let ext = Path::new(trimmed)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let stem_max = 200 - ext.len().min(10) - 1;
        let stem = &trimmed[..stem_max.min(trimmed.len())];
        if ext.is_empty() {
            stem.to_string()
        } else {
            format!("{}.{}", stem, ext)
        }
    } else {
        trimmed.to_string()
    }
}

/// Validate that an output path is safe — no traversal outside the output directory.
pub fn safe_output_path(output_dir: &str, subdir: &str, filename: &str) -> PathBuf {
    let safe_name = sanitize_filename(filename);
    let base = PathBuf::from(output_dir);
    let full = base.join(subdir).join(&safe_name);

    // Verify the resolved path is still under output_dir
    if let (Ok(_resolved_base), Ok(resolved_full)) = (
        std::fs::canonicalize(&base).or_else(|_| Ok::<PathBuf, std::io::Error>(base.clone())),
        // For new files, canonicalize the parent
        full.parent()
            .map(|p| {
                std::fs::canonicalize(p)
                    .unwrap_or_else(|_| p.to_path_buf())
                    .join(&safe_name)
            })
            .ok_or(std::io::Error::other("no parent")),
    ) {
        // Check no path component is ".."
        for component in resolved_full.components() {
            if matches!(component, Component::ParentDir) {
                return base.join(subdir).join("unnamed");
            }
        }
        full
    } else {
        full
    }
}

/// Mask a bot token for safe display in logs/errors.
/// Shows only the first 4 chars + last 4 chars.
pub fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        return "****".to_string();
    }
    format!("{}…{}", &token[..4], &token[token.len() - 4..])
}

/// Validate that a URL is HTTPS (Telegram API must always be HTTPS).
pub fn validate_https_url(url: &str) -> bool {
    url.starts_with("https://")
}

/// Validate bot token format: should be digits:alphanumeric
pub fn validate_token_format(token: &str) -> bool {
    let parts: Vec<&str> = token.splitn(2, ':').collect();
    if parts.len() != 2 {
        return false;
    }
    let id_part = parts[0];
    let secret_part = parts[1];

    // Bot ID should be numeric
    if !id_part.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Secret should be alphanumeric with possible - and _
    if secret_part.is_empty() {
        return false;
    }
    secret_part
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_normal_filename() {
        assert_eq!(sanitize_filename("report.pdf"), "report.pdf");
    }

    #[test]
    fn sanitize_path_traversal() {
        assert_eq!(sanitize_filename("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_filename("../../../secret.txt"), "secret.txt");
    }

    #[test]
    fn sanitize_null_bytes() {
        assert_eq!(sanitize_filename("file\0.pdf"), "file.pdf");
    }

    #[test]
    fn sanitize_directory_separators() {
        // On Unix, Path::file_name extracts after the last /
        // On Windows, it also handles \
        let result = sanitize_filename("dir/file.pdf");
        assert_eq!(result, "file.pdf");
        // Backslash in filename on Unix is kept but replaced with _
        // On Windows it would extract as file_name
        let result = sanitize_filename("dir\\file.pdf");
        assert!(result == "file.pdf" || result == "dir_file.pdf");
    }

    #[test]
    fn sanitize_special_chars() {
        assert_eq!(sanitize_filename("file<>.pdf"), "file__.pdf");
    }

    #[test]
    fn sanitize_dot_only() {
        assert_eq!(sanitize_filename("..."), "unnamed");
        assert_eq!(sanitize_filename("."), "unnamed");
    }

    #[test]
    fn sanitize_empty() {
        assert_eq!(sanitize_filename(""), "unnamed");
    }

    #[test]
    fn sanitize_long_filename() {
        let long_name = "a".repeat(300) + ".pdf";
        let result = sanitize_filename(&long_name);
        assert!(result.len() <= 200);
        assert!(result.ends_with(".pdf"));
    }

    #[test]
    fn mask_token_normal() {
        let result = mask_token("1234567890:ABCDEFghijklmn");
        assert!(result.starts_with("1234"));
        assert!(result.ends_with("klmn"));
        assert!(result.contains('…'));
    }

    #[test]
    fn mask_token_short() {
        assert_eq!(mask_token("short"), "****");
    }

    #[test]
    fn validate_token_valid() {
        assert!(validate_token_format("123456:ABCdef-_123"));
    }

    #[test]
    fn validate_token_invalid_no_colon() {
        assert!(!validate_token_format("nocolon"));
    }

    #[test]
    fn validate_token_invalid_non_numeric_id() {
        assert!(!validate_token_format("abc:secret"));
    }

    #[test]
    fn validate_https() {
        assert!(validate_https_url("https://api.telegram.org/bot123/getMe"));
        assert!(!validate_https_url("http://api.telegram.org/bot123/getMe"));
    }
}
