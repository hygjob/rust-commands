// clap CLI, 디렉터리 엔트리 읽기·정렬·`-l` 상세 출력 (유닉스 퍼미션 비트 사용)
use clap::Parser;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Rust로 만든 ls 명령어
/// 디렉토리 내용을 나열합니다
#[derive(Parser)]
#[command(name = "ls")]
#[command(version, about = "Rust로 만든 ls 명령어", disable_help_flag = true)]
struct Cli {
    /// 도움말을 출력합니다
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// 긴 형식으로 출력합니다
    #[arg(short = 'l')]
    long: bool,

    /// 숨김 파일도 표시합니다
    #[arg(short = 'a')]
    all: bool,

    /// 파일 크기를 사람 친화적 형태로 표시합니다
    #[arg(short = 'h')]
    human_readable: bool,

    /// 나열할 디렉토리
    #[arg(default_value = ".")]
    dir: String,
}

/// `st_mode` 하위 9비트로 소유자/그룹/기타 rwx와 타입 접두 `d`/`-`를 만듭니다.
fn format_permissions(mode: u32, is_dir: bool) -> String {
    let file_type = if is_dir { 'd' } else { '-' };
    format!("{}{}{}{}{}{}{}{}{}{}",
        file_type,
        if mode & 0o400 != 0 { 'r' } else { '-' },
        if mode & 0o200 != 0 { 'w' } else { '-' },
        if mode & 0o100 != 0 { 'x' } else { '-' },
        if mode & 0o040 != 0 { 'r' } else { '-' },
        if mode & 0o020 != 0 { 'w' } else { '-' },
        if mode & 0o010 != 0 { 'x' } else { '-' },
        if mode & 0o004 != 0 { 'r' } else { '-' },
        if mode & 0o002 != 0 { 'w' } else { '-' },
        if mode & 0o001 != 0 { 'x' } else { '-' })
}

/// `-h`일 때 K/M/G 단위로, 아니면 바이트 숫자 문자열.
fn format_size(size: u64, human_readable: bool) -> String {
    if !human_readable {
        return size.to_string();
    }

    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        format!("{:.1}G", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        size.to_string()
    }
}

/// 수정 시각을 단순 근사(일·월 환산)로 `"Mon DD HH:MM"` 형태 문자열로 만듭니다.
fn format_time(metadata: &fs::Metadata) -> String {
    let modified = metadata.modified().unwrap_or(
        std::time::UNIX_EPOCH
    );
    let duration = modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let month_day = days % 365;
    let month = (month_day / 30) + 1;
    let day = (month_day % 30) + 1;

    format!("{} {:02} {:02}:{:02}", month_name(month), day, hours, minutes)
}

fn month_name(m: u64) -> &'static str {
    match m {
        1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
        5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
        9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
        _ => "???",
    }
}

/// 정렬·출력용으로 경로와 파일명을 함께 둡니다.
struct EntryRow {
    path: PathBuf,
    file_name: OsString,
}

/// 이름 순(대소문자 무시)으로 정렬합니다. `-a`이면 `.`·`..`를 앞에 두고 나머지와 합칩니다.
fn prepare_rows(entries: Vec<fs::DirEntry>, list_dir: &Path, all: bool) -> Vec<EntryRow> {
    let mut rows: Vec<EntryRow> = entries
        .into_iter()
        .map(|e| EntryRow {
            path: e.path(),
            file_name: e.file_name(),
        })
        .collect();
    // -a 일 때 .과 .. 이 보이도록
    if all {
        let mut dot = None;
        let mut dotdot = None;
        let mut rest = Vec::new();
        for row in rows {
            if row.file_name.as_os_str() == OsStr::new(".") {
                dot = Some(row);
            } else if row.file_name.as_os_str() == OsStr::new("..") {
                dotdot = Some(row);
            } else {
                rest.push(row);
            }
        }
        rest.sort_by_key(|r| r.file_name.to_string_lossy().to_lowercase());
        let dot = dot.unwrap_or_else(|| EntryRow {
            path: list_dir.join("."),
            file_name: OsString::from("."),
        });
        let dotdot = dotdot.unwrap_or_else(|| EntryRow {
            path: list_dir.join(".."),
            file_name: OsString::from(".."),
        });
        let mut out = vec![dot, dotdot];
        out.extend(rest);
        out
    } else {
        rows.sort_by_key(|r| r.file_name.to_string_lossy().to_lowercase());
        rows
    }
}

fn main() {
    let cli = Cli::parse();

    let mut entries: Vec<_> = match fs::read_dir(&cli.dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            eprintln!("ls: {}: {}", cli.dir, e);
            std::process::exit(1);
        }
    };

    // 기본: `.`로 시작하는 이름 제외 (`-a`가 아닐 때)
    if !cli.all {
        entries.retain(|e| {
            !e.file_name().to_string_lossy().starts_with('.')
        });
    }

    let list_path = Path::new(&cli.dir);
    let rows = prepare_rows(entries, list_path, cli.all);

    // `-l`: 퍼미션·크기·시간·이름(디렉터리는 끝에 `/`)
    if cli.long {
        for row in &rows {
            let file_name = &row.file_name;
            let name = file_name.to_string_lossy();
            let metadata =
                fs::metadata(&row.path).expect("metadata for directory entry");

            let mode = metadata.permissions().mode();
            let perms = format_permissions(mode, metadata.is_dir());
            let size = format_size(metadata.len(), cli.human_readable);
            let time = format_time(&metadata);

            println!("{} {} {} {}  {}", perms, size, time, name,
                if metadata.is_dir() { "/" } else { "" });
        }
    } else {
        // 짧은 형식: 이름만 한 줄에 나열
        for row in &rows {
            let file_name = &row.file_name;
            let name = file_name.to_string_lossy();
            print!("{}  ", name);
        }
        println!();
    }
}

