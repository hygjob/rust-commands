// clap·walkdir로 트리 순회, 파일 메타데이터로 필터
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

/// 와일드카드 `*`(임의 길이)·`?`(한 글자)를 지원하는 단순 글롭 매칭입니다.
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
            // `*`가 텍스트의 어디까지 먹을지 나중에 되돌리며 시도
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

    // 남은 패턴이 `*`뿐이면 매칭 성공
    while pi < pattern_chars.len() && pattern_chars[pi] == '*' {
        pi += 1;
    }

    pi == pattern_chars.len()
}

/// 패턴에 `*`/`?`가 있으면 글롭, 없으면 파일명 전체 일치로 비교합니다.
fn match_name(file_name: &str, pattern: &str) -> bool {
    if pattern.contains('*') || pattern.contains('?') {
        glob_match(file_name, pattern)
    } else {
        file_name == pattern
    }
}

/// `-t`: `f` 파일, `d` 디렉터리, `l` 심볼릭 링크; 그 외 값은 필터하지 않음.
fn match_type(entry: &walkdir::DirEntry, type_str: &str) -> bool {
    let ft = entry.file_type();
    match type_str {
        "f" => ft.is_file(),
        "d" => ft.is_dir(),
        "l" => ft.is_symlink(),
        _ => true,
    }
}

/// 일반 파일일 때만 `metadata` 길이를 돌려줍니다 (`-s` 최소 크기 비교용).
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

    // 깊이 우선으로 모든 항목 순회; 권한 오류 등은 건너뜀
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
