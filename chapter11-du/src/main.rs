// clap CLI, 디렉터리 재귀 합산과 깊이·`-s`에 따른 출력 제어
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

/// `-h`일 때 K~T 단위 문자열, 아니면 바이트 숫자.
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

/// 하위까지 크기를 합산해 반환합니다. 디렉터리 자식마다 재귀 호출 후, `-s`가 아니면 자식별 줄을 출력합니다.
fn du_recursive(
    path: &Path,
    current_depth: usize,
    max_depth: Option<usize>,
    follow_links: bool,
    summarize: bool,
    human_readable: bool,
) -> io::Result<u64> {
    // `-L`: 링크를 따라 실제 파일·디렉터리 크기; 아니면 링크 자체 메타데이터(디렉터리가 아니면 0에 가깝게 처리)
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

        // 각 자식 서브트리 크기를 더함; `-d` 범위 안이면 자식 경로별 한 줄 출력
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

            // `-s`면 중간 경로는 출력하지 않고 합만 위로 전달
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
        // 디렉터리·일반 파일이 아니면(예: `-L` 없이 심볼릭 링크) 사용량에 포함하지 않음
        Ok(0)
    }
}

fn main() {
    let cli = Cli::parse();

    // 인자마다 루트 경로 합계 한 줄(재귀 안에서 자식 줄은 옵션에 따라 추가)
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
