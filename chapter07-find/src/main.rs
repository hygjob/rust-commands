use clap::Parser;
use walkdir::WalkDir;
use std::fs;

/// Rust로 만든 find 명령어
/// 디렉토리에서 파일을 검색합니다
#[derive(Parser)]
#[command(name = "find")]
#[command(version, about = "Rust로 만든 find 명령어")]
struct Cli {
    /// 파일 이름 패턴 (와일드카드 지원, 예: *.rs)
    #[arg(short = 'n', long = "name")]
    name: Option<String>,

    /// 파일 타입 (f=파일, d=디렉토리, l=심볼릭 링크)
    #[arg(short = 't', long = "type")]
    file_type: Option<String>,

    /// 최소 파일 크기 (바이트)
    #[arg(short = 's', long = "size")]
    min_size: Option<u64>,

    /// 검색 시작 경로 (기본값: 현재 디렉토리)
    #[arg(default_value = ".")]
    path: String,
}

fn glob_match(text: &str, pattern: &str) -> bool {
    let mut ti = 0;
    let mut pi = 0;
    let mut star_idx = usize::MAX;
    let mut match_idx = 0;
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    while ti < text_chars.len() {
        if pi < pattern_chars.len()
            && (pattern_chars[pi] == text_chars[ti] || pattern_chars[pi] == '?')
        {
            ti += 1;
            pi += 1;
        } else if pi < pattern_chars.len() && pattern_chars[pi] == '*' {
            star_idx = pi;
            match_idx = ti;
            pi += 1;
        } else if star_idx != usize::MAX {
            pi = star_idx + 1;
            match_idx += 1;
            ti = match_idx;
        } else {
            return false;
        }
    }

    while pi < pattern_chars.len() && pattern_chars[pi] == '*' {
        pi += 1;
    }

    pi == pattern_chars.len()
}

fn match_name(file_name: &str, pattern: &str) -> bool {
    if pattern.contains('*') || pattern.contains('?') {
        glob_match(file_name, pattern)
    } else {
        file_name == pattern
    }
}

fn match_type(entry: &walkdir::DirEntry, type_str: &str) -> bool {
    let ft = entry.file_type();
    match type_str {
        "f" => ft.is_file(),
        "d" => ft.is_dir(),
        "l" => ft.is_symlink(),
        _ => true,
    }
}

fn get_file_size(entry: &walkdir::DirEntry) -> Option<u64> {
    if entry.file_type().is_file() {
        fs::metadata(entry.path()).ok().map(|m| m.len())
    } else {
        None
    }
}

/// GNU find는 `-type`, `-name` 등 한 덩어리로 쓰지만, clap은 `-type`을 `-t`+값 `ype`로 해석한다.
/// 흔한 find 스타일 인자를 긴 옵션으로 바꿔서 파싱을 맞춘다.
fn normalize_find_style_argv(mut args: Vec<String>) -> Vec<String> {
    for arg in args.iter_mut().skip(1) {
        match arg.as_str() {
            "-type" => *arg = "--type".into(),
            "-name" => *arg = "--name".into(),
            "-size" => *arg = "--size".into(),
            _ => {}
        }
    }
    args
}

fn main() {
    let args = normalize_find_style_argv(std::env::args().collect());
    let cli = Cli::parse_from(args);

    for entry in WalkDir::new(&cli.path).into_iter().filter_map(|e| e.ok()) {
        // 파일 타입 필터
        if let Some(ref type_str) = cli.file_type {
            if !match_type(&entry, type_str) {
                continue;
            }
        }

        // 파일 이름 필터
        if let Some(ref pattern) = cli.name {
            let file_name = entry.file_name().to_string_lossy();
            if !match_name(&file_name, pattern) {
                continue;
            }
        }

        // 파일 크기 필터
        if let Some(min_size) = cli.min_size {
            if let Some(size) = get_file_size(&entry) {
                if size < min_size {
                    continue;
                }
            } else {
                continue;
            }
        }

        println!("{}", entry.path().display());
    }
}
