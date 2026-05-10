// 여러 유닉스 스타일 도구를 하나의 바이너리 서브커맨드로 묶습니다. `human_size`는 라이브러리 크레이트에서.
use clap::{Parser, Subcommand};
use std::io;
use std::fs::{self, File};
use std::io::{ BufRead, BufReader, Read, Seek, SeekFrom, stdin, Write};
use std::collections::VecDeque;
use std::path::{PathBuf};
use rustbox_lib::format::human_size;

/// 최상위 CLI; 실제 처리는 `Commands` 분기와 아래 `cmd_*` 함수가 담당합니다.
#[derive(Parser)]
#[command(name = "rustbox")]
#[command(version, about = "Rust로 만든 유닉스 커맨드 모음")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "텍스트를 표준 출력에 표시합니다")]
    Echo {
        /// 마지막에 줄바꿈을 하지 않습니다
        #[arg(short = 'n')]
        no_newline: bool,
        /// 백슬래시 이스케이프 해석을 활성화합니다
        #[arg(short = 'e')]
        escape: bool,
        /// 출력할 텍스트
        text: Vec<String>,
    },

    #[command(about = "파일 내용을 표준 출력에 표시합니다")]
    Cat {
        /// 모든 줄에 번호를 매깁니다
        #[arg(short = 'n')]
        number: bool,
        /// 빈 줄이 아닌 줄에만 번호를 매깁니다
        #[arg(short = 'b')]
        number_nonblank: bool,
        /// 연속된 빈 줄을 하나로 압축합니다
        #[arg(short = 's')]
        squeeze_blank: bool,
        /// 출력할 파일 목록
        files: Vec<String>,
    },

    #[command(about = "파일의 앞부분을 출력합니다")]
    Head {
        /// 출력할 줄 수
        #[arg(short = 'n', default_value = "10")]
        lines: usize,
        /// 출력할 바이트 수
        #[arg(short = 'c')]
        bytes: Option<usize>,
        /// 입력 파일
        files: Vec<std::path::PathBuf>,
    },

    #[command(about = "파일의 뒷부분을 출력합니다")]
    Tail {
        /// 출력할 줄 수
        #[arg(short = 'n', default_value = "10")]
        lines: usize,
        /// 출력할 바이트 수
        #[arg(short = 'c')]
        bytes: Option<usize>,
        /// 입력 파일
        files: Vec<std::path::PathBuf>,
    },

    #[command(about = "패턴과 일치하는 줄을 검색합니다")]
    Grep {
        /// 대소문자 구분 없이 검색합니다
        #[arg(short = 'i')]
        ignore_case: bool,
        /// 줄 번호를 함께 출력합니다
        #[arg(short = 'n')]
        line_number: bool,
        /// 일치하는 줄 수만 출력합니다
        #[arg(short = 'c')]
        count: bool,
        /// 검색할 정규표현식 패턴
        pattern: String,
        /// 검색할 파일 또는 디렉토리
        paths: Vec<String>,
    },

    #[command(about = "텍스트 줄을 정렬합니다")]
    Sort {
        /// 역순으로 정렬합니다
        #[arg(short = 'r')]
        reverse: bool,
        /// 숫자 기준으로 정렬합니다
        #[arg(short = 'n')]
        numeric: bool,
        /// 대소문자를 구분하지 않습니다
        #[arg(short = 'f')]
        ignore_case: bool,
        /// 중복 줄을 제거합니다
        #[arg(short = 'u')]
        unique: bool,
        /// 입력 파일
        files: Vec<std::path::PathBuf>,
    },

    #[command(about = "디렉토리에서 파일을 검색합니다")]
    Find {
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
    },

    #[command(about = "디렉토리 내용을 나열합니다")]
    Ls {
        /// 긴 형식으로 출력합니다
        #[arg(short = 'l')]
        long: bool,
        /// 숨김 파일도 포함하여 출력합니다
        #[arg(short = 'a')]
        all: bool,
        /// 파일 크기를 사람이 읽기 쉬운 형식으로 출력합니다
        #[arg(short = 'r')]  // h는 help와 겹치므로 r로 변경
        human_readable: bool,
        /// 나열할 디렉토리
        #[arg(default_value = ".")]
        dir: String,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // `find`·`ls`는 내부에서 직접 출력하며, 나머지는 `io::Result`로 오류를 전달합니다.
    match cli.command {
        Commands::Echo { no_newline, escape, text } => {
            cmd_echo(no_newline, escape, text)?;
        }
        Commands::Cat { number, number_nonblank, squeeze_blank, files } => {
            cmd_cat(number, number_nonblank, squeeze_blank, files)?;
        }
        Commands::Head { lines, bytes, files } => {
            cmd_head(lines, bytes, files)?;
        }
        Commands::Tail { lines, bytes, files } => {
            cmd_tail(lines, bytes, files)?;
        }
        Commands::Grep { ignore_case, line_number, count, pattern, paths } => {
            cmd_grep(ignore_case, line_number, count, pattern, paths)?;
        }
        Commands::Sort { reverse, numeric, ignore_case, unique, files } => {
            cmd_sort(reverse, numeric, ignore_case, unique, files)?;
        }
        Commands::Find { name, file_type, min_size, path } => {
            cmd_find(name, file_type, min_size, path);
        }
        Commands::Ls { long, all, human_readable, dir } => {
            cmd_ls(long, all, human_readable, dir);
        }
    }

    Ok(())
}

// ── echo ──────────────────────────────────────────

/// `-e`: `\n` `\t` 등 백슬래시 시퀀스를 실제 제어 문자로 바꿉니다.
fn process_escapes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('n') => { result.push('\n'); chars.next(); }
                Some('t') => { result.push('\t'); chars.next(); }
                Some('r') => { result.push('\r'); chars.next(); }
                Some('\\') => { result.push('\\'); chars.next(); }
                Some('a') => { result.push('\x07'); chars.next(); }
                Some('b') => { result.push('\x08'); chars.next(); }
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 인자 공백 결합 후 `-e`/`-n` 적용. 마지막에 줄바꿈 생략 가능.
fn cmd_echo(no_newline: bool, escape: bool, text: Vec<String>) -> io::Result<()> {
    let text = text.join(" ");
    let output = if escape { process_escapes(&text) } else { text };
    if no_newline { print!("{}", output); } else { println!("{}", output); }
    io::stdout().flush()?;
    Ok(())
}

// ── cat ──────────────────────────────────────────

/// `-n`/`-b`/`-s` 조합으로 한 줄씩 표준 출력에 보냅니다.
fn cat_lines<R: BufRead>(
    reader: &mut R, number: bool, number_nonblank: bool, squeeze_blank: bool,
) -> io::Result<()> {
    let mut line_number = 1;
    let mut prev_blank = false;
    for line in reader.lines() {
        let line = line?;
        if squeeze_blank && line.is_empty() {
            if prev_blank { continue; }
            prev_blank = true;
        } else {
            prev_blank = false;
        }
        if number_nonblank {
            if line.is_empty() { println!(); }
            else { println!("{:6}\t{}", line_number, line); line_number += 1; }
        } else if number {
            println!("{:6}\t{}", line_number, line); line_number += 1;
        } else {
            println!("{}", line);
        }
    }
    Ok(())
}

/// 파일 목록이 비면 stdin, 아니면 순서대로 열어 `cat_lines`에 넘깁니다.
fn cmd_cat(number: bool, number_nonblank: bool, squeeze_blank: bool, files: Vec<String>) -> io::Result<()> {
    if files.is_empty() {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin.lock());
        cat_lines(&mut reader, number, number_nonblank, squeeze_blank)?;
    } else {
        for filename in &files {
            match File::open(filename) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    cat_lines(&mut reader, number, number_nonblank, squeeze_blank)?;
                }
                Err(e) => eprintln!("cat: {}: {}", filename, e),
            }
        }
    }
    Ok(())
}

// ── head ──────────────────────────────────────────

/// 앞에서부터 `n`줄만 출력합니다.
fn head_lines<R: BufRead>(reader: &mut R, n: usize) -> io::Result<()> {
    for (i, line) in reader.lines().enumerate() {
        if i >= n { break; }
        let line = line?;
        println!("{}", line);
    }
    Ok(())
}

/// `-c`: 앞에서부터 최대 `n`바이트만 stdout으로 복사합니다.
fn head_bytes<R: Read>(reader: &mut R, n: usize) -> io::Result<()> {
    let mut handle = reader.take(n as u64);
    let mut stdout = io::stdout();
    io::copy(&mut handle, &mut stdout)?;
    Ok(())
}

/// stdin 또는 여러 파일; 복수 파일이면 `==> path <==` 헤더. `-c`면 바이트, 아니면 줄.
fn cmd_head(lines: usize, bytes: Option<usize>, files: Vec<PathBuf>) -> io::Result<()> {
    if files.is_empty() {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin.lock());
        match bytes {
            Some(n) => head_bytes(&mut reader, n)?,
            None => head_lines(&mut reader, lines)?,
        }
    } else {
        for (i, path) in files.iter().enumerate() {
            if files.len() > 1 {
                if i > 0 { println!(); }
                println!("==> {} <==", path.display());
            }
            let file = File::open(path)?;
            let mut reader = BufReader::new(file);
            match bytes {
                Some(n) => head_bytes(&mut reader, n)?,
                None => head_lines(&mut reader, lines)?,
            }
        }
    }
    Ok(())
}

// ── tail ──────────────────────────────────────────

/// 슬라이딩 `VecDeque`로 마지막 `n`줄만 유지합니다.
fn tail_lines<R: BufRead>(reader: &mut R, n: usize) -> io::Result<()> {
    let mut buf: VecDeque<String> = VecDeque::with_capacity(n);
    for line in reader.lines() {
        let line = line?;
        if buf.len() == n { buf.pop_front(); }
        buf.push_back(line);
    }
    for line in &buf { println!("{}", line); }
    Ok(())
}

/// stdin은 줄 단위만. 파일+`-c`는 끝에서 `seek`해 마지막 n바이트, 아니면 `tail_lines`.
fn cmd_tail(lines: usize, bytes: Option<usize>, files: Vec<PathBuf>) -> io::Result<()> {
    if files.is_empty() {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin.lock());
        tail_lines(&mut reader, lines)?;
    } else {
        for (i, path) in files.iter().enumerate() {
            if files.len() > 1 {
                if i > 0 { println!(); }
                println!("==> {} <==", path.display());
            }
            if let Some(n) = bytes {
                let mut file = File::open(path)?;
                let file_size = file.seek(SeekFrom::End(0))?;
                let start = if file_size > n as u64 { file_size - n as u64 } else { 0 };
                file.seek(SeekFrom::Start(start))?;
                let mut stdout = io::stdout();
                io::copy(&mut file, &mut stdout)?;
            } else {
                let file = File::open(path)?;
                let mut reader = BufReader::new(file);
                tail_lines(&mut reader, lines)?;
            }
        }
    }
    Ok(())
}

// ── grep ──────────────────────────────────────────

// 이 섹션만 색·정귀식 크레이트 사용 (파일 상단 import 그룹과 분리)
use colored::*;
use regex::{Regex, RegexBuilder};

/// 일치 구간을 터미널에서 빨간 굵게 표시합니다.
fn highlight_matches(line: &str, re: &Regex) -> String {
    let mut result = String::new();
    let mut last_end = 0;
    for mat in re.find_iter(line) {
        result.push_str(&line[last_end..mat.start()]);
        result.push_str(&line[mat.start()..mat.end()].red().bold().to_string());
        last_end = mat.end();
    }
    result.push_str(&line[last_end..]);
    result
}

/// 각 경로를 파일 하나로만 처리(디렉터리 재귀 없음). `-c`면 파일별 건수, 아니면 하이라이트 줄.
fn cmd_grep(ignore_case: bool, line_number: bool, count: bool, pattern: String, paths: Vec<String>) -> io::Result<()> {
    let re = RegexBuilder::new(&pattern)
        .case_insensitive(ignore_case)
        .build()
        .unwrap();

    let mut total_count = 0;
    for path in &paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut match_count = 0;

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            if re.is_match(&line) {
                match_count += 1;
                if !count {
                    let highlighted = highlight_matches(&line, &re);
                    if line_number {
                        println!("{}:{}:{}", path, i + 1, highlighted);
                    } else {
                        println!("{}", highlighted);
                    }
                }
            }
        }
        if count {
            println!("{}:{}", path, match_count);
        }
        total_count += match_count;
    }
    let _ = total_count;
    Ok(())
}

// ── sort ──────────────────────────────────────────

/// 숫자로 파싱되면 비교, 아니면 문자열 비교로 대체합니다.
fn numeric_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let num_a = a.trim().parse::<f64>();
    let num_b = b.trim().parse::<f64>();
    match (num_a, num_b) {
        (Ok(a), Ok(b)) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

/// stdin 또는 파일들에서 줄을 모은 뒤 정렬; `-u`면 정렬 후 `dedup`.
fn cmd_sort(reverse: bool, numeric: bool, ignore_case: bool, unique: bool, files: Vec<PathBuf>) -> io::Result<()> {
    let mut lines = Vec::new();
    if files.is_empty() {
        let stdin = stdin();
        let reader = BufReader::new(stdin.lock());
        for line in reader.lines() { lines.push(line?); }
    } else {
        for path in &files {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                for line in reader.lines() { lines.push(line?); }
            }
        }
    }

    lines.sort_by(|a, b| {
        let ord = if numeric { numeric_compare(a, b) }
        else if ignore_case { a.to_lowercase().cmp(&b.to_lowercase()) }
        else { a.cmp(b) };
        if reverse { ord.reverse() } else { ord }
    });

    if unique { lines.dedup(); }
    for line in &lines { println!("{}", line); }
    Ok(())
}

// ── find ──────────────────────────────────────────

use walkdir::WalkDir;

/// `*`·`?` 와일드카드 글롭(단순 백트래킹).
fn glob_match(text: &str, pattern: &str) -> bool {
    let mut ti = 0; let mut pi = 0;
    let mut star_idx = usize::MAX; let mut match_idx = 0;
    let tc: Vec<char> = text.chars().collect();
    let pc: Vec<char> = pattern.chars().collect();
    while ti < tc.len() {
        if pi < pc.len() && (pc[pi] == tc[ti] || pc[pi] == '?') { ti += 1; pi += 1; }
        else if pi < pc.len() && pc[pi] == '*' { star_idx = pi; match_idx = ti; pi += 1; }
        else if star_idx != usize::MAX { pi = star_idx + 1; match_idx += 1; ti = match_idx; }
        else { return false; }
    }
    while pi < pc.len() && pc[pi] == '*' { pi += 1; }
    pi == pc.len()
}

/// 트리 순회하며 타입·이름·최소 크기 필터를 통과한 경로만 한 줄씩 출력합니다.
fn cmd_find(name: Option<String>, file_type: Option<String>, min_size: Option<u64>, path: String) {
    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        if let Some(ref ft) = file_type {
            let is_match = match ft.as_str() {
                "f" => entry.file_type().is_file(),
                "d" => entry.file_type().is_dir(),
                _ => true,
            };
            if !is_match { continue; }
        }
        if let Some(ref pattern) = name {
            let fname = entry.file_name().to_string_lossy();
            if !glob_match(&fname, pattern) { continue; }
        }
        if let Some(min) = min_size {
            if let Ok(meta) = fs::metadata(entry.path()) {
                if meta.len() < min { continue; }
            } else { continue; }
        }
        println!("{}", entry.path().display());
    }
}

// ── ls ──────────────────────────────────────────

/// `-l`일 때 고정 퍼미션 문자열+크기+이름, 아니면 이름만 나열. 숨김 필터는 `-a`로 해제.
fn cmd_ls(long: bool, all: bool, human_readable: bool, dir: String) {
    let mut entries: Vec<_> = match fs::read_dir(&dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(e) => { eprintln!("ls: {}: {}", dir, e); return; }
    };
    if !all {
        entries.retain(|e| !e.file_name().to_string_lossy().starts_with('.'));
    }
    entries.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());

    if long {
        for entry in &entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().unwrap();
            let size = if human_readable { human_size(meta.len()) } else { meta.len().to_string() };
            //let type_marker = if meta.is_dir() { "/" } else { "" };
            println!("{}{} {}  {}", if meta.is_dir() { "d" } else { "-" },
                "rwxr-xr-x", size, name);
        }
    } else {
        for entry in &entries {
            print!("{}  ", entry.file_name().to_string_lossy());
        }
        println!();
    }
}