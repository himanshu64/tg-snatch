use clap::Parser;
use tg_snatch::cli::{Cli, Command, FileTypeFilter};

fn parse(args: &[&str]) -> Cli {
    let mut full_args = vec!["tg-snatch"];
    full_args.extend_from_slice(args);
    Cli::parse_from(full_args)
}

#[test]
fn no_args_gives_none_command() {
    let cli = parse(&[]);
    assert!(cli.command.is_none());
}

#[test]
fn parse_setup_command() {
    let cli = parse(&["setup"]);
    assert!(matches!(cli.command, Some(Command::Setup)));
}

#[test]
fn parse_info_command() {
    let cli = parse(&["--token", "abc123", "info"]);
    assert_eq!(cli.token.as_deref(), Some("abc123"));
    assert!(matches!(cli.command, Some(Command::Info)));
}

#[test]
fn parse_watch_command() {
    let cli = parse(&["watch", "--chat-id", "-1001234567890", "--continuous"]);
    match &cli.command {
        Some(Command::Watch {
            chat_id,
            continuous,
            duration,
        }) => {
            assert_eq!(*chat_id, -1001234567890);
            assert!(*continuous);
            assert_eq!(*duration, None);
        }
        _ => panic!("Expected Watch command"),
    }
}

#[test]
fn parse_watch_with_duration() {
    let cli = parse(&["watch", "--chat-id", "100", "--duration", "300"]);
    match &cli.command {
        Some(Command::Watch { duration, .. }) => {
            assert_eq!(*duration, Some(300));
        }
        _ => panic!("Expected Watch command"),
    }
}

#[test]
fn parse_list_with_filters() {
    let cli = parse(&[
        "list",
        "--type",
        "pdf",
        "--chat-id",
        "-100",
        "--limit",
        "25",
    ]);
    match &cli.command {
        Some(Command::List {
            chat_id,
            r#type,
            limit,
            ..
        }) => {
            assert_eq!(*chat_id, Some(-100));
            assert!(matches!(r#type, Some(FileTypeFilter::Pdf)));
            assert_eq!(*limit, 25);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn parse_list_defaults() {
    let cli = parse(&["list"]);
    match &cli.command {
        Some(Command::List {
            chat_id,
            r#type,
            limit,
            ..
        }) => {
            assert_eq!(*chat_id, None);
            assert!(r#type.is_none());
            assert_eq!(*limit, 50);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn parse_download_interactive() {
    let cli = parse(&["download", "--interactive"]);
    match &cli.command {
        Some(Command::Download {
            interactive,
            all,
            parallel,
            ..
        }) => {
            assert!(*interactive);
            assert!(!*all);
            assert_eq!(*parallel, 3);
        }
        _ => panic!("Expected Download command"),
    }
}

#[test]
fn parse_download_all_with_type() {
    let cli = parse(&["download", "--all", "--type", "image", "--parallel", "5"]);
    match &cli.command {
        Some(Command::Download {
            all,
            r#type,
            parallel,
            ..
        }) => {
            assert!(*all);
            assert!(matches!(r#type, Some(FileTypeFilter::Image)));
            assert_eq!(*parallel, 5);
        }
        _ => panic!("Expected Download command"),
    }
}

#[test]
fn parse_download_specific_files() {
    let cli = parse(&["download", "--file-id", "abc", "--file-id", "def"]);
    match &cli.command {
        Some(Command::Download { file_id, .. }) => {
            assert_eq!(file_id, &vec!["abc".to_string(), "def".to_string()]);
        }
        _ => panic!("Expected Download command"),
    }
}

#[test]
fn parse_all_file_types() {
    let types = [
        "pdf",
        "image",
        "video",
        "audio",
        "document",
        "animation",
        "voice",
        "all",
    ];
    for t in types {
        let cli = parse(&["list", "--type", t]);
        match &cli.command {
            Some(Command::List { r#type, .. }) => {
                assert!(r#type.is_some(), "Failed to parse type: {}", t);
            }
            _ => panic!("Expected List command"),
        }
    }
}

#[test]
fn default_output_dir() {
    let cli = parse(&["info"]);
    assert_eq!(cli.output_dir, "./downloads");
}

#[test]
fn custom_output_dir() {
    let cli = parse(&["--output-dir", "/tmp/my-downloads", "info"]);
    assert_eq!(cli.output_dir, "/tmp/my-downloads");
}

#[test]
fn verbose_flag() {
    let cli = parse(&["-v", "info"]);
    assert!(cli.verbose);
}
