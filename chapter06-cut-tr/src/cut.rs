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

    let field_list = cli.fields
        .as_ref()
        .map(|f| parse_ranges(f))
        .unwrap_or_default();

    let char_list = cli.characters
        .as_ref()
        .map(|c| parse_ranges(c))
        .unwrap_or_default();

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
