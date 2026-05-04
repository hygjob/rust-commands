use clap::Parser;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};
use std::path::PathBuf;

/// Rust로 만든 sort 명령어
#[derive(Parser)]
#[command(name = "sort")]
#[command(version, about = "Rust로 만든 sort 명령어")]
struct Cli {
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

    /// 정렬 기준 필드 번호 (1부터 시작)
    #[arg(short = 'k')]
    key: Option<usize>,

    /// 입력 파일
    files: Vec<PathBuf>,
}

fn numeric_compare(a: &str, b: &str) -> Ordering {
    let num_a = a.trim().parse::<f64>();
    let num_b = b.trim().parse::<f64>();

    match (num_a, num_b) {
        (Ok(a), Ok(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

fn get_field(line: &str, field_num: usize) -> String {
    let fields: Vec<&str> = line.split_whitespace().collect();
    fields.get(field_num - 1).unwrap_or(&"").to_string()
}

fn read_lines(paths: &[PathBuf]) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();

    if paths.is_empty() {
        let stdin = stdin();
        let reader = BufReader::new(stdin.lock());
        for line in reader.lines() {
            lines.push(line?);
        }
    } else {
        for path in paths {
            match File::open(path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        lines.push(line?);
                    }
                }
                Err(e) => eprintln!("sort: {}: {}", path.display(), e),
            }
        }
    }

    Ok(lines)
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut lines = read_lines(&cli.files)?;

    lines.sort_by(|a, b| {
        let ord = if let Some(k) = cli.key {
            let key_a = get_field(a, k);
            let key_b = get_field(b, k);
            if cli.numeric {
                numeric_compare(&key_a, &key_b)
            } else {
                key_a.cmp(&key_b)
            }
        } else if cli.numeric {
            numeric_compare(a, b)
        } else if cli.ignore_case {
            a.to_lowercase().cmp(&b.to_lowercase())
        } else {
            a.cmp(b)
        };

        if cli.reverse { ord.reverse() } else { ord }
    });

    if cli.unique {
        lines.dedup();
    }

    for line in &lines {
        println!("{}", line);
    }

    Ok(())
}
