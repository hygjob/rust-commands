// clap CLI, stdin·파일에서 줄 단위로 필드/문자 추출
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};

/// Rust로 만든 cut 명령어
#[derive(Parser)]
#[command(name = "cut")]
#[command(version, about = "Rust로 만든 cut 명령어")]
struct Cli {
    /// 추출할 필드 번호 (쉼표로 구분, 예: 1,3)
    #[arg(short = 'f')]
    fields: Option<String>,

    /// 필드 구분자 (기본값: 탭)
    #[arg(short = 'd', default_value = "\t")]
    delimiter: String,

    /// 추출할 문자 위치 (쉼표로 구분, 예: 1,3,5)
    #[arg(short = 'c')]
    characters: Option<String>,

    /// 입력 파일
    files: Vec<String>,
}

/// `1,3`, `1-3` 형태를 파싱해 1-based 인덱스 목록으로 만듭니다. 범위는 양 끝 포함, 중복 제거 후 정렬.
fn parse_ranges(input: &str) -> Vec<usize> {
    let mut ranges = Vec::new();

    for part in input.split(',') {
        if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() == 2 {
                let start: usize = bounds[0].parse().unwrap_or(1);
                let end: usize = bounds[1].parse().unwrap_or(1);
                for i in start..=end {
                    if !ranges.contains(&i) {
                        ranges.push(i);
                    }
                }
            }
        } else {
            if let Ok(n) = part.parse::<usize>() {
                if !ranges.contains(&n) {
                    ranges.push(n);
                }
            }
        }
    }

    ranges.sort();
    ranges
}

/// `-f`: 구분자로 나눈 뒤 지정 필드만 같은 구분자로 이어 붙입니다.
fn cut_fields(line: &str, delimiter: &str, fields: &[usize]) -> String {
    let parts: Vec<&str> = line.split(delimiter).collect();
    let mut result = Vec::new();

    for &field_num in fields {
        if let Some(part) = parts.get(field_num - 1) {
            result.push(*part);
        }
    }

    result.join(delimiter)
}

/// `-c`: UTF-8 문자 기준으로 `positions`번째 글자만 이어 붙입니다 (1부터).
fn cut_characters(line: &str, positions: &[usize]) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::new();

    for &pos in positions {
        if let Some(&c) = chars.get(pos - 1) {
            result.push(c);
        }
    }

    result
}

/// 필드 목록이 있으면 `-f`, 없고 문자 목록이 있으면 `-c`, 둘 다 없으면 줄 그대로 출력.
fn process_lines(
    reader: &mut dyn BufRead,
    cli: &Cli,
    field_list: &[usize],
    char_list: &[usize],
) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        let output = if !field_list.is_empty() {
            cut_fields(&line, &cli.delimiter, field_list)
        } else if !char_list.is_empty() {
            cut_characters(&line, char_list)
        } else {
            line
        };
        println!("{}", output);
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // `-f` / `-c` 문자열을 숫자·범위 목록으로 변환
    let field_list = cli.fields
        .as_ref()
        .map(|f| parse_ranges(f))
        .unwrap_or_default();

    let char_list = cli.characters
        .as_ref()
        .map(|c| parse_ranges(c))
        .unwrap_or_default();

    // 인자 없으면 stdin, 있으면 파일 순서대로 동일 규칙 적용
    if cli.files.is_empty() {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin.lock());
        process_lines(&mut reader, &cli, &field_list, &char_list)?;
    } else {
        for filename in &cli.files {
            match File::open(filename) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    process_lines(&mut reader, &cli, &field_list, &char_list)?;
                }
                Err(e) => eprintln!("cut: {}: {}", filename, e),
            }
        }
    }

    Ok(())
}
