use clap::Parser;
use std::fs;
use std::io;
use std::path::Path;

/// Rust로 만든 du 명령어
/// 디스크 사용량을 추정합니다
#[derive(Parser)]
#[command(name = "du")]
#[command(version, about = "Rust로 만든 du 명령어", disable_help_flag = true)]
struct Cli {
    /// 도움말을 출력합니다
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// 사람 친화적 포맷으로 출력합니다
    #[arg(short = 'h')]
    human_readable: bool,

    /// 각 항목의 합계만 출력합니다
    #[arg(short = 's')]
    summarize: bool,

    /// 출력할 최대 깊이
    #[arg(short = 'd')]
    max_depth: Option<usize>,

    /// 심볼릭 링크를 따라갑니다
    #[arg(short = 'L')]
    follow_links: bool,

    /// 분석할 경로
    #[arg(default_value = ".")]
    paths: Vec<String>,
}

fn format_size(bytes: u64, human_readable: bool) -> String {
    if !human_readable {
        return bytes.to_string();
    }

    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.1}T", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

fn du_recursive(
    path: &Path,
    current_depth: usize,
    max_depth: Option<usize>,
    follow_links: bool,
    summarize: bool,
    human_readable: bool,
) -> io::Result<u64> {
    let metadata = if follow_links {
        fs::metadata(path)
    } else {
        fs::symlink_metadata(path)
    }?;

    if metadata.is_file() {
        Ok(metadata.len())
    } else if metadata.is_dir() {
        let mut total = 0u64;

        let mut entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name().to_string_lossy().to_string());

        for entry in entries {
            let entry_path = entry.path();
            let size = du_recursive(
                &entry_path,
                current_depth + 1,
                max_depth,
                follow_links,
                summarize,
                human_readable,
            )?;

            if !summarize {
                let show = max_depth.map_or(true, |max| current_depth + 1 <= max);
                if show {
                    println!("{}\t{}", format_size(size, human_readable), entry_path.display());
                }
            }

            total += size;
        }

        Ok(total)
    } else {
        // 심볼릭 링크 등 (follow_links가 아니면)
        Ok(0)
    }
}

fn main() {
    let cli = Cli::parse();

    for path_str in &cli.paths {
        let path = Path::new(path_str);

        if !path.exists() {
            eprintln!("du: {}: 그런 파일이나 디렉토리가 없습니다", path_str);
            continue;
        }

        match du_recursive(
            path,
            0,
            cli.max_depth,
            cli.follow_links,
            cli.summarize,
            cli.human_readable,
        ) {
            Ok(total) => {
                println!("{}\t{}", format_size(total, cli.human_readable), path.display());
            }
            Err(e) => {
                eprintln!("du: {}: {}", path.display(), e);
            }
        }
    }
}
