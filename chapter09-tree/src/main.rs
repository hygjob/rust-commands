// clap CLI, 디렉터리 재귀 순회 후 트리 문자로 출력
use clap::Parser;
use std::fs;
use std::path::Path;

/// Rust로 만든 tree 명령어
/// 디렉토리 구조를 트리 형태로 출력합니다
#[derive(Parser)]
#[command(name = "tree")]
#[command(version, about = "Rust로 만든 tree 명령어")]
struct Cli {
    /// 표시할 최대 깊이
    #[arg(short = 'L')]
    max_depth: Option<usize>,

    /// 디렉토리만 표시합니다
    #[arg(short = 'd')]
    dirs_only: bool,

    /// 숨김 파일도 표시합니다
    #[arg(short = 'a')]
    show_hidden: bool,

    /// 시작 디렉토리
    #[arg(default_value = ".")]
    directory: String,
}

/// `dir` 아래를 재귀적으로 출력합니다. 반환값은 (디렉터리 개수, 파일 개수) 누적입니다.
fn print_tree(
    dir: &Path,
    prefix: &str,
    current_depth: usize,
    max_depth: Option<usize>,
    dirs_only: bool,
    show_hidden: bool,
) -> std::io::Result<(usize, usize)> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();

    // `-a`가 아니면 `.`로 시작하는 항목 제외
    if !show_hidden {
        entries.retain(|e| {
            !e.file_name().to_string_lossy().starts_with('.')
        });
    }

    entries.sort_by_key(|e| e.file_name().to_string_lossy().to_string());

    let mut total_dirs = 0;
    let mut total_files = 0;
    let count = entries.len();

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let is_dir = path.is_dir();

        // `-d`: 디렉터리만 출력
        if dirs_only && !is_dir {
            continue;
        }

        // 형제 중 마지막이면 `└`, 아니면 `├`
        let connector = if is_last { "└── " } else { "├── " };
        println!("{}{}{}", prefix, connector, name);

        if is_dir {
            total_dirs += 1;

            // `-L`: 다음 깊이가 한계 이상이면 내용은 들여다보지 않음
            if let Some(max) = max_depth {
                if current_depth + 1 >= max {
                    continue;
                }
            }

            // 자식 줄 앞에 붙일 세로 막대·공백 (마지막 형제면 막대 생략)
            let new_prefix = if is_last { "    " } else { "│   " };
            let (d, f) = print_tree(
                &path,
                &format!("{}{}", prefix, new_prefix),
                current_depth + 1,
                max_depth,
                dirs_only,
                show_hidden,
            )?;
            total_dirs += d;
            total_files += f;
        } else {
            total_files += 1;
        }
    }

    Ok((total_dirs, total_files))
}

fn main() {
    let cli = Cli::parse();
    let path = Path::new(&cli.directory);

    if !path.is_dir() {
        eprintln!("tree: {}: 디렉토리가 아닙니다", cli.directory);
        std::process::exit(1);
    }

    // 루트 경로 한 줄 후, 깊이 0부터 트리 본문
    println!("{}", path.display());

    match print_tree(path, "", 0, cli.max_depth, cli.dirs_only, cli.show_hidden) {
        Ok((dirs, files)) => {
            println!();
            // 하단 요약 (복수형 영어 접미사 단순 처리)
            println!("{} directory{}, {} file{}",
                dirs, if dirs != 1 { "ies" } else { "y" },
                files, if files != 1 { "s" } else { "" });
        }
        Err(e) => {
            eprintln!("tree: {}", e);
            std::process::exit(1);
        }
    }
}
