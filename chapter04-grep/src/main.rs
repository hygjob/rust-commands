// clap CLI, 정규식·색 하이라이트, 파일/디렉터리 순회
use clap::Parser;
use colored::*;
use regex::{Regex, RegexBuilder};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};

/// Rust로 만든 grep 명령어
#[derive(Parser)]
#[command(name = "grep")]
#[command(version, about = "Rust로 만든 grep 명령어")]
struct Cli {
    /// 대소문자 구분 없이 검색합니다
    #[arg(short = 'i')]
    ignore_case: bool,

    /// 줄 번호를 함께 출력합니다
    #[arg(short = 'n')]
    line_number: bool,

    /// 일치하는 줄 수만 출력합니다
    #[arg(short = 'c')]
    count: bool,

    /// 디렉토리를 재귀적으로 검색합니다
    #[arg(short = 'r')]
    recursive: bool,

    /// 일치한 줄 뒤에 추가로 출력할 줄 수
    #[arg(short = 'A')]
    after_context: Option<usize>,

    /// 일치한 줄 앞에 추가로 출력할 줄 수
    #[arg(short = 'B')]
    before_context: Option<usize>,

    /// 검색할 정규표현식 패턴
    pattern: String,

    /// 검색할 파일 또는 디렉토리
    paths: Vec<String>,
}

/// 한 줄에서 정규식에 일치하는 부분만 빨간 굵게 표시합니다.
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

/// 단일 파일을 검색합니다. `-A`/`-B`가 있으면 전체를 메모리에 올린 뒤 문맥 줄을 함께 출력합니다.
fn grep_file(
    filename: &str,
    re: &Regex,
    cli: &Cli,
) -> io::Result<()> {
    let has_context = cli.before_context.is_some() || cli.after_context.is_some();

    // 문맥 옵션: 파일 전체를 줄 단위로 읽어 인덱스 기준으로 이전/이후 줄 출력
    if has_context {
        let before = cli.before_context.unwrap_or(0);
        let after = cli.after_context.unwrap_or(0);

        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<io::Result<_>>()?;
        let mut match_count = 0;

        for (i, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                match_count += 1;
                if !cli.count {
                    let start = if i >= before { i - before } else { 0 };
                    for j in start..i {
                        println!("{}-{}", filename, lines[j]); // 일치 줄 이전 (컨텍스트)
                    }
                    let highlighted = highlight_matches(line, re);
                    println!("{}:{}", filename, highlighted); // 일치 줄
                    let end = std::cmp::min(i + after + 1, lines.len());
                    for j in (i + 1)..end {
                        println!("{}-{}", filename, lines[j]); // 일치 줄 이후
                    }
                }
            }
        }

        if cli.count {
            println!("{}:{}", filename, match_count);
        }
    } else {
        // 문맥 없음: 한 줄씩 스트리밍으로 검사
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut match_count = 0;

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            if re.is_match(&line) {
                match_count += 1;
                if !cli.count {
                    let highlighted = highlight_matches(&line, re);
                    if cli.line_number {
                        println!("{}:{}:{}", filename, i + 1, highlighted);
                    } else if cli.recursive  {
                        // 여러 파일 구분을 위해 파일명 접두사
                        println!("{}:{}", filename, highlighted);
                    } else {
                        println!("{}", highlighted);
                    }
                }
            }
        }

        if cli.count {
            println!("{}:{}", filename, match_count);
        }
    }

    Ok(())
}

/// 파일이면 `grep_file`, 디렉터리면 `-r`일 때만 하위 항목을 재귀 검색합니다.
fn search_path(path: &str, re: &Regex, cli: &Cli) -> io::Result<()> {
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        if !cli.recursive {
            eprintln!("grep: {}: 디렉토리입니다 (-r 옵션을 사용하세요)", path);
            return Ok(());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                search_path(&entry_path.display().to_string(), re, cli)?; // 하위 디렉터리
            } else {
                grep_file(&entry_path.display().to_string(), re, cli)?;
            }
        }
    } else {
        grep_file(path, re, cli)?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // `-i`면 대소문자 무시하는 정규식 컴파일
    let re = RegexBuilder::new(&cli.pattern)
        .case_insensitive(cli.ignore_case)
        .build()
        .unwrap();

    for path in &cli.paths {
        search_path(path, &re, &cli)?;
    }

    Ok(())
}
